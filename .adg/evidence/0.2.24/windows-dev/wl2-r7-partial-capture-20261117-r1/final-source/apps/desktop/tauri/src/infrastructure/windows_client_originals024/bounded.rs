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
mod query;
use query::{err, invalid, native_id, projections, time, ProjectionRecord};
pub(super) use query::{native_stamp, required_access, Policy, Projection, QueryError, SnapshotData, REQUIRED};
#[cfg(test)]
use query::{project, query, same_covered_component};

/// Actual retained native bounded proof. Private fields, no serde/Clone, and no
/// constructor accepts snapshots/bytes/bool from a request or persisted record.
pub(super) struct BoundedAccessV1 {
    file: File,
    parent: PinnedDirectory,
    id: FileIdentity,
    projections: Vec<Projection>,
    policy_digest: String,
}
impl BoundedAccessV1 {
    /// Observe actual config security/parent files BEFORE their close transfer.
    /// This exposes only DATA, never a close/restore/Stop permission.
    pub(super) fn config_close_identity(&self, end: Instant) -> io::Result<FileIdentity> {
        self.recheck(end).map_err(io::Error::other)?;
        Ok(self.id)
    }
    pub(super) fn observe_config_close_inventory(
        &self,
        inventory: &mut super::handle_release::ExpectedHandleInventory,
        prefix: &str,
        end: Instant,
    ) -> io::Result<()> {
        self.recheck(end).map_err(io::Error::other)?;
        inventory.observe(
            format!("{prefix}/file"),
            &self.file,
            self.id,
            false,
            false,
            end,
        )?;
        inventory.observe_directory(&format!("{prefix}/parent"), &self.parent, end)
    }
    /// Infallible ownership transfer. Only RoutePin's already-observed FILE
    /// scope uses this method; all original native files have a retained sink.
    pub(super) fn transfer_config_close_files(
        self,
        batch: &mut super::handle_release::NativeHandleBatch,
        prefix: &str,
    ) {
        let Self {
            file, parent, id, ..
        } = self;
        batch.retain(format!("{prefix}/file"), file, id, false, false);
        for (i, file) in parent.into_owned_files().into_iter().enumerate() {
            batch.retain_observed(format!("{prefix}/parent/{i}"), file, true, false);
        }
    }
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
