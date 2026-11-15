//! r4 NEW-session native access-security scope. Never a fallback from Full.
//! Full audit SACL/its global control and future information are ExcludedUnknown.
use crate::{
    infrastructure::{
        hash,
        windows_official_runtime024::{current_identity, FileIdentity, PinnedDirectory},
    },
    modules::runtime::windows_lifecycle024_contract::{
        BOUNDED_SECURITY_EXCLUDED_ASSURANCE, BOUNDED_SECURITY_INFORMATION,
        BOUNDED_SECURITY_QUERY_ACCESS, BOUNDED_SECURITY_SCOPE,
    },
};
use serde::{Deserialize, Serialize};
use std::{
    fs::{File, OpenOptions},
    io,
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle},
    path::Path,
    time::Instant,
};
use windows_sys::Win32::{
    Foundation::LocalFree,
    Security::{
        GetSecurityDescriptorControl, GetSecurityDescriptorLength, IsValidSecurityDescriptor,
        MakeSelfRelativeSD,
    },
    Storage::FileSystem::{GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION},
};
pub(super) const REQUIRED: [u32; 8] = [1, 2, 4, 16, 32, 64, 128, 256];
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Policy {
    scope_id: String,
    query_access: u32,
    included_mask: u32,
    required_projections: Vec<u32>,
    excluded_assurance: String,
}
impl Policy {
    pub(super) fn fixed() -> Self {
        Self {
            scope_id: BOUNDED_SECURITY_SCOPE.into(),
            query_access: BOUNDED_SECURITY_QUERY_ACCESS,
            included_mask: BOUNDED_SECURITY_INFORMATION,
            required_projections: REQUIRED.to_vec(),
            excluded_assurance: BOUNDED_SECURITY_EXCLUDED_ASSURANCE.into(),
        }
    }
    pub(super) fn validate(&self) -> io::Result<()> {
        if self != &Self::fixed() {
            return Err(invalid("fixed v2 security policy differs"));
        }
        Ok(())
    }
    pub(super) fn digest(&self) -> io::Result<String> {
        self.validate()?;
        Ok(hash::sha256_hex(
            &serde_json::to_vec(self).map_err(|_| invalid("policy encoding"))?,
        ))
    }
}
#[derive(Debug)]
pub(super) struct QueryError {
    pub(super) api: &'static str,
    pub(super) mask: u32,
    pub(super) raw: Option<i32>,
    pub(super) detail: String,
}
impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "api={} mask=0x{:x} raw={:?} {}",
            self.api, self.mask, self.raw, self.detail
        )
    }
}
impl std::error::Error for QueryError {}
fn invalid(message: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn err(api: &'static str, mask: u32, e: io::Error) -> QueryError {
    QueryError {
        api,
        mask,
        raw: e.raw_os_error(),
        detail: e.to_string(),
    }
}
#[link(name = "advapi32")]
unsafe extern "system" {
    fn QuerySecurityAccessMask(mask: u32, access: *mut u32);
    fn GetSecurityInfo(
        h: *mut std::ffi::c_void,
        kind: i32,
        mask: u32,
        o: *mut *mut std::ffi::c_void,
        g: *mut *mut std::ffi::c_void,
        d: *mut *mut windows_sys::Win32::Security::ACL,
        s: *mut *mut windows_sys::Win32::Security::ACL,
        sd: *mut *mut std::ffi::c_void,
    ) -> u32;
}
pub(super) fn required_access(mask: u32) -> u32 {
    let mut access = 0;
    unsafe {
        QuerySecurityAccessMask(mask, &mut access);
    }
    access
}
fn query(file: &File, mask: u32) -> Result<Vec<u8>, QueryError> {
    let access = required_access(mask);
    if access != BOUNDED_SECURITY_QUERY_ACCESS {
        return Err(QueryError {
            api: "QuerySecurityAccessMask",
            mask,
            raw: None,
            detail: format!("required access0x{access:x}, expected READ_CONTROL exactly"),
        });
    }
    let mut raw = std::ptr::null_mut();
    let code = unsafe {
        GetSecurityInfo(
            file.as_raw_handle(),
            1,
            mask,
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            std::ptr::null_mut(),
            &mut raw,
        )
    };
    if code != 0 {
        return Err(err(
            "GetSecurityInfo(SE_FILE_OBJECT,bounded)",
            mask,
            io::Error::from_raw_os_error(code as i32),
        ));
    }
    if raw.is_null() {
        return Err(err("GetSecurityInfo-null", mask, invalid("no descriptor")));
    }
    struct Local(*mut std::ffi::c_void);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe {
                LocalFree(self.0);
            }
        }
    }
    let _local = Local(raw);
    if unsafe { IsValidSecurityDescriptor(raw) } == 0 {
        return Err(err(
            "IsValidSecurityDescriptor",
            mask,
            invalid("invalid SD"),
        ));
    }
    let len = unsafe { GetSecurityDescriptorLength(raw) };
    if !(20..=262144).contains(&len) {
        return Err(err(
            "GetSecurityDescriptorLength",
            mask,
            invalid("SD length bound"),
        ));
    }
    let (mut control, mut revision) = (0u16, 0u32);
    if unsafe { GetSecurityDescriptorControl(raw, &mut control, &mut revision) } == 0 {
        return Err(err(
            "GetSecurityDescriptorControl",
            mask,
            io::Error::last_os_error(),
        ));
    }
    if revision != 1 {
        return Err(err("SD-revision", mask, invalid("unknown SD revision")));
    }
    if control & 0x8000 != 0 {
        return Ok(unsafe { std::slice::from_raw_parts(raw.cast::<u8>(), len as usize) }.to_vec());
    }
    let mut needed = 0;
    if unsafe { MakeSelfRelativeSD(raw, std::ptr::null_mut(), &mut needed) } != 0
        || io::Error::last_os_error().raw_os_error() != Some(122)
        || !(20..=262144).contains(&needed)
    {
        return Err(err(
            "MakeSelfRelativeSD-size",
            mask,
            invalid("relative buffer bound"),
        ));
    }
    let mut out = vec![0u8; needed as usize];
    if unsafe { MakeSelfRelativeSD(raw, out.as_mut_ptr().cast(), &mut needed) } == 0 {
        return Err(err("MakeSelfRelativeSD", mask, io::Error::last_os_error()));
    }
    if needed as usize != out.len() {
        return Err(err(
            "MakeSelfRelativeSD-drift",
            mask,
            invalid("relative buffer drift"),
        ));
    }
    Ok(out)
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct Projection {
    mask: u32,
    revision: u8,
    control: u16,
    state: String,
    sid: Option<Vec<u8>>,
    acl_revision: Option<u8>,
    aces: Vec<Vec<u8>>,
}
impl Projection {
    fn validate(&self) -> io::Result<()> {
        if !REQUIRED.contains(&self.mask) || self.revision != 1 {
            return Err(invalid("snapshot projection mask/revision"));
        }
        if self.mask == 1 || self.mask == 2 {
            if self.control & !if self.mask == 1 { 1 } else { 2 } != 0
                || self.acl_revision.is_some()
                || !self.aces.is_empty()
            {
                return Err(invalid("SID projection fields"));
            }
            match (&self.sid, self.state.as_str()) {
                (None, "absent") => {}
                (Some(b), "present") if sid_bytes(b, 0)? == *b => {}
                _ => return Err(invalid("SID projection state")),
            }
        } else {
            if self.sid.is_some() || self.control & !if self.mask == 4 { 0x150c } else { 0 } != 0 {
                return Err(invalid("ACL projection control"));
            }
            match self.state.as_str() {
                "absent" | "null" if self.acl_revision.is_none() && self.aces.is_empty() => {}
                "empty" | "present" if self.acl_revision.is_some_and(|r| [2, 4].contains(&r)) => {
                    if (self.state == "empty") != self.aces.is_empty() {
                        return Err(invalid("ACE state mismatch"));
                    }
                    let length = 8 + self.aces.iter().map(Vec::len).sum::<usize>();
                    if length > 65535 || self.aces.len() > 4096 {
                        return Err(invalid("ACL content bound"));
                    }
                    let mut b = vec![self.acl_revision.unwrap(), 0];
                    b.extend_from_slice(&(length as u16).to_le_bytes());
                    b.extend_from_slice(&(self.aces.len() as u16).to_le_bytes());
                    b.extend_from_slice(&[0, 0]);
                    for ace in &self.aces {
                        if ace.len() < 8 || u16at(ace, 2)? as usize != ace.len() {
                            return Err(invalid("ACE exact stored size"));
                        }
                        b.extend_from_slice(ace);
                    }
                    if acl(&b, 0, self.mask)?.1 != self.aces {
                        return Err(invalid("stored ACE projection mismatch"));
                    }
                }
                _ => return Err(invalid("ACL projection state")),
            }
        }
        Ok(())
    }
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct ProjectionRecord {
    requested_mask: u32,
    native_status: u32,
    structure: Projection,
    structure_digest: String,
}
#[derive(Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub(super) struct SnapshotData {
    scope_id: String,
    security_policy_digest: String,
    volume: u32,
    file_id: String,
    projections: Vec<ProjectionRecord>,
    audit_sacl_assurance: String,
    full_security_proven: bool,
}
impl SnapshotData {
    pub(super) fn validate(&self) -> io::Result<()> {
        if self.scope_id != BOUNDED_SECURITY_SCOPE
            || self.security_policy_digest != Policy::fixed().digest()?
            || self
                .file_id
                .parse::<u64>()
                .ok()
                .is_none_or(|id| id == 0 || id.to_string() != self.file_id)
            || self.audit_sacl_assurance != "excluded-unknown"
            || self.full_security_proven
            || self.projections.len() != 8
        {
            return Err(invalid("v2 bounded snapshot fields"));
        }
        for (entry, mask) in self.projections.iter().zip(REQUIRED) {
            if entry.requested_mask != mask
                || entry.structure.mask != mask
                || entry.native_status != 0
            {
                return Err(invalid("v2 required projection/status"));
            }
            entry.structure.validate()?;
            if entry.structure_digest
                != hash::sha256_hex(
                    &serde_json::to_vec(&entry.structure)
                        .map_err(|_| invalid("projection serialize"))?,
                )
            {
                return Err(invalid("v2 projection digest"));
            }
        }
        Ok(())
    }
    pub(super) fn matches_id(&self, volume: u32, id: &str) -> bool {
        self.volume == volume && self.file_id == id
    }
}
fn same_covered_component(a: &Projection, b: &Projection) -> bool {
    if a.mask <= 4 {
        return a == b;
    }
    // Per-projection state is retained from the individual query. The combined
    // descriptor's global SACL present/null/protection/revision is NOT evidence
    // for a different component. Compare only that component's actual ACEs.
    a.mask == b.mask
        && a.revision == b.revision
        && a.control == b.control
        && a.sid == b.sid
        && a.aces == b.aces
        && (a.aces.is_empty() || a.acl_revision == b.acl_revision)
}
fn u16at(b: &[u8], p: usize) -> io::Result<u16> {
    Ok(u16::from_le_bytes(
        b.get(p..p + 2)
            .ok_or_else(|| invalid("truncated u16"))?
            .try_into()
            .unwrap(),
    ))
}
fn u32at(b: &[u8], p: usize) -> io::Result<u32> {
    Ok(u32::from_le_bytes(
        b.get(p..p + 4)
            .ok_or_else(|| invalid("truncated u32"))?
            .try_into()
            .unwrap(),
    ))
}
fn sid_bytes(b: &[u8], p: usize) -> io::Result<Vec<u8>> {
    let head = b.get(p..p + 8).ok_or_else(|| invalid("SID header"))?;
    if head[0] != 1 || head[1] > 15 {
        return Err(invalid("SID revision/count"));
    }
    Ok(b.get(p..p + 8 + head[1] as usize * 4)
        .ok_or_else(|| invalid("truncated SID"))?
        .to_vec())
}
fn acl(b: &[u8], offset: usize, projection: u32) -> io::Result<(u8, Vec<Vec<u8>>)> {
    let header = b
        .get(offset..offset + 8)
        .ok_or_else(|| invalid("ACL header"))?;
    if ![2, 4].contains(&header[0]) {
        return Err(invalid("unknown ACL revision"));
    }
    let length = u16at(b, offset + 2)? as usize;
    let count = u16at(b, offset + 4)? as usize;
    if length < 8 || count > 4096 {
        return Err(invalid("ACL size/count"));
    }
    let bytes = b
        .get(offset..offset + length)
        .ok_or_else(|| invalid("ACL outside SD"))?;
    let mut pos = 8;
    let mut aces = Vec::new();
    for _ in 0..count {
        let n = u16at(bytes, pos + 2)? as usize;
        if n < 8 || n % 4 != 0 {
            return Err(invalid("ACE length/alignment"));
        }
        let ace = bytes
            .get(pos..pos + n)
            .ok_or_else(|| invalid("ACE outside ACL"))?;
        let kind = ace[0];
        if projection == 4 {
            if ![0, 1, 4, 5, 6, 9, 10, 11, 12].contains(&kind) {
                return Err(invalid("unknown covered DACL ACE type"));
            }
        } else if ![17, 18, 19, 20, 21].contains(&kind) {
            return Err(invalid("unknown non-audit policy ACE type"));
        }
        let sid_offset = if [5, 6, 11, 12].contains(&kind) {
            let flags = u32at(ace, 8)?;
            if flags & !3 != 0 {
                return Err(invalid("unknown object ACE flags"));
            }
            12 + if flags & 1 != 0 { 16 } else { 0 } + if flags & 2 != 0 { 16 } else { 0 }
        } else {
            8
        };
        sid_bytes(ace, sid_offset)?;
        let wanted = match projection {
            16 => 17,
            32 => 18,
            64 => 19,
            128 => 20,
            256 => 21,
            _ => kind,
        };
        if projection == 4 || kind == wanted {
            aces.push(ace.to_vec());
        }
        pos += n;
    }
    Ok((header[0], aces))
}
fn project(b: &[u8], mask: u32) -> io::Result<Projection> {
    if b.len() < 20 || b.len() > 262144 || b[0] != 1 || !REQUIRED.contains(&mask) {
        return Err(invalid("projection header/mask"));
    }
    let control = u16at(b, 2)?;
    if control & 0x8000 == 0 {
        return Err(invalid("self-relative descriptor required"));
    }
    let mut out = Projection {
        mask,
        revision: 1,
        control: 0,
        state: "absent".into(),
        sid: None,
        acl_revision: None,
        aces: vec![],
    };
    if mask == 1 || mask == 2 {
        out.control = control & if mask == 1 { 1 } else { 2 };
        let p = u32at(b, if mask == 1 { 4 } else { 8 })? as usize;
        if p != 0 {
            if p < 20 || p % 4 != 0 {
                return Err(invalid("SID relative offset"));
            }
            out.sid = Some(sid_bytes(b, p)?);
            out.state = "present".into();
        }
    } else {
        let dacl = mask == 4;
        let present = control & if dacl { 4 } else { 16 } != 0;
        let p = u32at(b, if dacl { 16 } else { 12 })? as usize;
        if dacl {
            out.control = control & 0x150c;
        }
        if !present {
            if p != 0 {
                return Err(invalid("ACL nonzero but not present"));
            }
        } else if p == 0 {
            out.state = "null".into();
        } else {
            if p < 20 || p % 4 != 0 {
                return Err(invalid("ACL relative offset"));
            }
            let (revision, aces) = acl(b, p, mask)?;
            out.state = if aces.is_empty() { "empty" } else { "present" }.into();
            out.acl_revision = Some(revision);
            out.aces = aces;
        }
    }
    Ok(out)
}
/// Actual retained native bounded proof. Private fields, no serde/Clone, and no
/// constructor accepts snapshots/bytes/bool from a request or persisted record.
pub(super) struct BoundedAccessV1 {
    file: File,
    parent: PinnedDirectory,
    id: FileIdentity,
    projections: Vec<Projection>,
    policy_digest: String,
}
fn native_id(file: &File) -> io::Result<FileIdentity> {
    let mut i: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    if unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut i) } == 0 {
        return Err(io::Error::last_os_error());
    }
    if i.dwFileAttributes & 0x400 != 0 || i.dwFileAttributes & 0x10 == 0 && i.nNumberOfLinks != 1 {
        return Err(invalid("reparse/multilink"));
    }
    Ok(FileIdentity {
        volume: i.dwVolumeSerialNumber,
        index: ((i.nFileIndexHigh as u64) << 32) | i.nFileIndexLow as u64,
    })
}
fn time(deadline: Instant) -> Result<(), QueryError> {
    if Instant::now() >= deadline {
        Err(err(
            "bounded-original-deadline",
            0,
            io::ErrorKind::TimedOut.into(),
        ))
    } else {
        Ok(())
    }
}
fn projections(file: &File, deadline: Instant) -> Result<Vec<Projection>, QueryError> {
    let mut out = Vec::new();
    for mask in REQUIRED {
        time(deadline)?;
        let raw = query(file, mask)?;
        out.push(project(&raw, mask).map_err(|e| err("bounded-projection-structure", mask, e))?);
    }
    let combined = query(file, BOUNDED_SECURITY_INFORMATION)?;
    for (mask, expected) in REQUIRED.into_iter().zip(&out) {
        let actual =
            project(&combined, mask).map_err(|e| err("bounded-combined-structure", mask, e))?;
        if !same_covered_component(&actual, expected) {
            return Err(err(
                "bounded-individual-combined-drift",
                mask,
                invalid("projection differs"),
            ));
        }
    }
    time(deadline)?;
    Ok(out)
}
impl BoundedAccessV1 {
    pub(super) fn snapshot_data(&self) -> io::Result<SnapshotData> {
        let projections = self
            .projections
            .iter()
            .map(|p| {
                let bytes = serde_json::to_vec(p).map_err(|_| invalid("projection serialize"))?;
                Ok(ProjectionRecord {
                    requested_mask: p.mask,
                    native_status: 0,
                    structure: serde_json::from_slice(&bytes)
                        .map_err(|_| invalid("projection serialize"))?,
                    structure_digest: hash::sha256_hex(&bytes),
                })
            })
            .collect::<io::Result<Vec<_>>>()?;
        let data = SnapshotData {
            scope_id: BOUNDED_SECURITY_SCOPE.into(),
            security_policy_digest: self.policy_digest.clone(),
            volume: self.id.volume,
            file_id: self.id.index.to_string(),
            projections,
            audit_sacl_assurance: "excluded-unknown".into(),
            full_security_proven: false,
        };
        data.validate()?;
        Ok(data)
    }
    pub(super) fn capture(
        path: &Path,
        directory: bool,
        deadline: Instant,
    ) -> Result<Self, QueryError> {
        time(deadline)?;
        let token = current_identity().map_err(|e| err("TokenUser", 0, e))?;
        if token.elevated {
            return Err(err(
                "TokenElevation",
                0,
                invalid("ordinary unchanged token required"),
            ));
        }
        for mask in REQUIRED.into_iter().chain([BOUNDED_SECURITY_INFORMATION]) {
            let access = required_access(mask);
            if access != BOUNDED_SECURITY_QUERY_ACCESS {
                return Err(err(
                    "QuerySecurityAccessMask",
                    mask,
                    invalid(&format!("access0x{access:x}")),
                ));
            }
        }
        let parent = PinnedDirectory::open(
            path.parent()
                .ok_or_else(|| err("original-parent", 0, invalid("parent absent")))?,
        )
        .map_err(|e| err("original-parent-pin", 0, e))?;
        let file = OpenOptions::new()
            .access_mode(BOUNDED_SECURITY_QUERY_ACCESS)
            .share_mode(1)
            .custom_flags(0x00200000 | if directory { 0x02000000 } else { 0 })
            .open(path)
            .map_err(|e| err("CreateFileW(READ_CONTROL)", BOUNDED_SECURITY_INFORMATION, e))?;
        let id = native_id(&file).map_err(|e| err("GetFileInformationByHandle", 0, e))?;
        if file
            .metadata()
            .map_err(|e| err("original-kind", 0, e))?
            .is_dir()
            != directory
        {
            return Err(err("original-kind", 0, invalid("file/directory differs")));
        }
        let found = projections(&file, deadline)?;
        if projections(&file, deadline)? != found
            || native_id(&file).map_err(|e| err("identity-recheck", 0, e))? != id
        {
            return Err(err(
                "bounded-original-drift",
                0,
                invalid("object/projections changed"),
            ));
        }
        parent.recheck().map_err(|e| err("parent-recheck", 0, e))?;
        Ok(Self {
            file,
            parent,
            id,
            projections: found,
            policy_digest: Policy::fixed()
                .digest()
                .map_err(|e| err("policyDigest", 0, e))?,
        })
    }
    pub(super) fn recheck(&self, deadline: Instant) -> Result<(), QueryError> {
        self.parent
            .recheck()
            .map_err(|e| err("parent-recheck", 0, e))?;
        if native_id(&self.file).map_err(|e| err("identity-recheck", 0, e))? != self.id
            || projections(&self.file, deadline)? != self.projections
        {
            return Err(err(
                "bounded-original-drift",
                0,
                invalid("covered projection drift"),
            ));
        }
        Ok(())
    }
    pub(super) fn diagnostic(&self) -> serde_json::Value {
        serde_json::json!({"securityScopeId":BOUNDED_SECURITY_SCOPE,"coveredStatus":"verified","auditSaclAssurance":"excluded-unknown","fullSecurityProven":false,"securityPolicyDigest":self.policy_digest,"projections":self.projections.iter().map(|p|serde_json::json!({"requestedMask":p.mask,"nativeStatus":0,"state":p.state,"structureSha256":hash::sha256_hex(&serde_json::to_vec(p).unwrap())})).collect::<Vec<_>>()})
    }
}
#[cfg(test)]
mod tests;
