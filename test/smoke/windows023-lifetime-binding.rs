//! Candidate protocol reader, not registered as a production Stop authority.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
fn read_candidate(path: &Path) -> Option<Vec<u8>> {
    use std::io::Read;
    let file = std::fs::File::open(path).ok()?;
    let metadata = file.metadata().ok()?;
    if !metadata.is_file() || metadata.len() > 8192 {
        return None;
    }
    let mut bytes = Vec::new();
    file.take(8193).read_to_end(&mut bytes).ok()?;
    (bytes.len() <= 8192).then_some(bytes)
}

#[derive(Clone, Debug, Deserialize, serde::Serialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub(crate) struct Binding {
    schema: String,
    home: PathBuf,
    sid: String,
    pid: u32,
    created: String,
    generation: String,
    job: String,
    lease_volume: u32,
    lease_index: String,
}
fn key(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/").to_lowercase()
}
#[derive(Clone, Copy)]
struct Expected<'a> {
    home: &'a Path,
    sid: &'a str,
    pid: u32,
    created: &'a str,
    secret: &'a str,
    challenge: &'a str,
    port: u16,
    proof: &'a str,
}
impl Binding {
    fn authenticates(&self, expected: &Expected<'_>) -> bool {
        let Expected {
            home: expected_home,
            sid: expected_sid,
            pid: expected_pid,
            created: expected_created,
            secret,
            challenge,
            port,
            proof,
        } = *expected;
        let valid_key =
            |v: &str| v.len() == 43 && URL_SAFE_NO_PAD.decode(v).is_ok_and(|b| b.len() == 32);
        let home = key(&self.home);
        let mut job_digest = Sha256::new();
        job_digest.update(self.sid.as_bytes());
        job_digest.update([0]);
        job_digest.update(home.as_bytes());
        let expected_job = format!(
            "Global{}opencodex-runtime-{:x}",
            char::from(92),
            job_digest.finalize()
        );
        if self.schema != "ocxd-runtime-lifetime/1"
            || !self.home.is_absolute()
            || home != key(expected_home)
            || self.sid != expected_sid
            || self.pid == 0
            || self.pid != expected_pid
            || self.created != expected_created
            || self.created.parse::<u64>().is_err()
            || self.lease_index.parse::<u64>().is_err()
            || uuid::Uuid::parse_str(&self.generation).is_err()
            || self.job != expected_job
            || port == 0
            || !valid_key(secret)
            || !valid_key(challenge)
        {
            return false;
        }
        let tuple = serde_json::json!([
            self.schema,
            self.home,
            self.sid,
            self.pid,
            self.created,
            self.generation,
            self.job,
            self.lease_volume,
            self.lease_index
        ]);
        let Ok(tuple) = serde_json::to_string(&tuple) else {
            return false;
        };
        let Ok(actual) = URL_SAFE_NO_PAD.decode(proof) else {
            return false;
        };
        let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else {
            return false;
        };
        mac.update(
            format!("opencodex-runtime-lifetime-v1\n{challenge}\n{port}\n{tuple}").as_bytes(),
        );
        mac.verify_slice(&actual).is_ok()
    }
}

pub(crate) fn verify_candidate_file(path: &Path, home: &Path, sid: &str) -> bool {
    let Some(bytes) = read_candidate(path) else {
        return false;
    };
    if bytes.len() > 8192 {
        return false;
    }
    let Ok(value) = serde_json::from_slice::<serde_json::Value>(&bytes) else {
        return false;
    };
    let Some(pid) = value["pid"].as_u64().and_then(|p| u32::try_from(p).ok()) else {
        return false;
    };
    let Some(created) = value["created"].as_str() else {
        return false;
    };
    let Some(proof) = value["candidateProof"].as_str() else {
        return false;
    };
    let Ok(binding) = serde_json::from_value::<Binding>(value["candidateBinding"].clone()) else {
        return false;
    };
    let secret = "A".repeat(43);
    let challenge = URL_SAFE_NO_PAD.encode([1u8; 32]);
    let expected = Expected {
        home,
        sid,
        pid,
        created,
        secret: &secret,
        challenge: &challenge,
        port: 1,
        proof,
    };
    if !binding.authenticates(&expected) {
        return false;
    }
    for (field, bad) in [
        ("home", serde_json::json!(home.join("other"))),
        ("sid", serde_json::json!(format!("{sid}-other"))),
        ("pid", serde_json::json!(pid.wrapping_add(1))),
        ("created", serde_json::json!("0")),
        (
            "generation",
            serde_json::json!(uuid::Uuid::new_v4().to_string()),
        ),
        ("job", serde_json::json!("Global\\other-job")),
        (
            "leaseVolume",
            serde_json::json!(binding.lease_volume.wrapping_add(1)),
        ),
        ("leaseIndex", serde_json::json!("0")),
    ] {
        let mut bad_value = value["candidateBinding"].clone();
        bad_value[field] = bad;
        let Ok(changed) = serde_json::from_value::<Binding>(bad_value) else {
            return false;
        };
        if changed.authenticates(&expected) {
            return false;
        }
    }
    !binding.authenticates(&Expected {
        secret: &"C".repeat(43),
        ..expected
    }) && !binding.authenticates(&Expected {
        challenge: &"D".repeat(43),
        ..expected
    }) && !binding.authenticates(&Expected {
        port: 2,
        ..expected
    })
}

#[cfg(windows)]
pub(crate) mod native {
    use super::*;
    use std::{
        ffi::c_void,
        fs::File,
        io,
        os::windows::{
            ffi::{OsStrExt, OsStringExt},
            fs::OpenOptionsExt,
            io::{AsRawHandle, FromRawHandle, OwnedHandle},
        },
        ptr,
    };
    use windows_sys::Win32::{
        Foundation::{LocalFree, HANDLE},
        Security::{
            EqualSid, GetAce, GetSecurityDescriptorControl, GetTokenInformation, IsValidAcl,
            TokenUser, ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, SE_DACL_PROTECTED,
        },
        Storage::FileSystem::{
            GetFileInformationByHandle, LockFileEx, UnlockFileEx, BY_HANDLE_FILE_INFORMATION,
        },
        System::{
            JobObjects::{
                IsProcessInJob, JobObjectBasicAccountingInformation, OpenJobObjectW,
                QueryInformationJobObject, JOBOBJECT_BASIC_ACCOUNTING_INFORMATION,
            },
            Threading::{
                GetProcessId, GetProcessTimes, OpenProcess, OpenProcessToken,
                QueryFullProcessImageNameW, WaitForSingleObject, PROCESS_QUERY_LIMITED_INFORMATION,
            },
        },
    };
    #[link(name = "advapi32")]
    extern "system" {
        fn GetSecurityInfo(
            handle: HANDLE,
            kind: i32,
            info: u32,
            owner: *mut *mut c_void,
            group: *mut *mut c_void,
            dacl: *mut *mut ACL,
            sacl: *mut *mut ACL,
            descriptor: *mut *mut c_void,
        ) -> u32;
        fn ConvertStringSidToSidW(text: *const u16, sid: *mut *mut c_void) -> i32;
    }
    fn error() -> io::Error {
        io::Error::other("Candidate native lifetime witness failed; no Stop authority granted")
    }
    fn token_matches(process: HANDLE, sid: &str) -> io::Result<()> {
        unsafe {
            let mut raw = ptr::null_mut();
            if OpenProcessToken(process, 8, &mut raw) == 0 {
                return Err(error());
            }
            let token = OwnedHandle::from_raw_handle(raw);
            let mut needed = 0;
            GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                ptr::null_mut(),
                0,
                &mut needed,
            );
            if !(16..=65536).contains(&needed) {
                return Err(error());
            }
            let mut storage =
                vec![0usize; (needed as usize).div_ceil(std::mem::size_of::<usize>())];
            if GetTokenInformation(
                token.as_raw_handle(),
                TokenUser,
                storage.as_mut_ptr().cast(),
                needed,
                &mut needed,
            ) == 0
            {
                return Err(error());
            }
            let base = storage.as_ptr() as usize;
            let actual = *(base as *const *mut c_void);
            let address = actual as usize;
            if address < base
                || address
                    .checked_add(8)
                    .is_none_or(|a| a > base + needed as usize)
            {
                return Err(error());
            }
            let mut expected = ptr::null_mut();
            let name = wide(std::ffi::OsStr::new(sid));
            if ConvertStringSidToSidW(name.as_ptr(), &mut expected) == 0 {
                return Err(error());
            }
            let equal = EqualSid(actual, expected);
            LocalFree(expected);
            if equal == 0 {
                return Err(error());
            }
            Ok(())
        }
    }
    fn image_matches(process: HANDLE, expected: &Path) -> io::Result<()> {
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        if unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) } == 0
        {
            return Err(error());
        }
        let actual = PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length as usize]));
        if key(&std::fs::canonicalize(actual)?) != key(&std::fs::canonicalize(expected)?) {
            return Err(error());
        }
        Ok(())
    }
    fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
        value.encode_wide().chain(Some(0)).collect()
    }
    fn private_job(job: HANDLE, sid: &str) -> io::Result<()> {
        unsafe {
            let (mut owner, mut dacl, mut descriptor) =
                (ptr::null_mut(), ptr::null_mut(), ptr::null_mut());
            let code = GetSecurityInfo(
                job,
                6,
                5,
                &mut owner,
                ptr::null_mut(),
                &mut dacl,
                ptr::null_mut(),
                &mut descriptor,
            );
            if code != 0 {
                return Err(io::Error::from_raw_os_error(code as i32));
            }
            let mut expected = ptr::null_mut();
            let text = wide(std::ffi::OsStr::new(sid));
            if ConvertStringSidToSidW(text.as_ptr(), &mut expected) == 0 {
                LocalFree(descriptor);
                return Err(error());
            }
            let result = (|| {
                let (mut control, mut revision) = (0u16, 0u32);
                if EqualSid(owner, expected) == 0
                    || GetSecurityDescriptorControl(descriptor, &mut control, &mut revision) == 0
                    || control & SE_DACL_PROTECTED == 0
                    || dacl.is_null()
                    || IsValidAcl(dacl) == 0
                    || (*dacl).AceCount != 1
                {
                    return Err(error());
                }
                let mut ace = ptr::null_mut();
                if GetAce(dacl, 0, &mut ace) == 0 {
                    return Err(error());
                }
                let header = &*ace.cast::<ACE_HEADER>();
                if header.AceType != 0 || header.AceFlags != 0 || header.AceSize < 16 {
                    return Err(error());
                }
                let value = &*ace.cast::<ACCESS_ALLOWED_ACE>();
                let principal = ptr::addr_of!(value.SidStart).cast::<c_void>();
                let count = *principal.cast::<u8>().add(1) as u16;
                if count > 15
                    || header.AceSize != 16 + count * 4
                    || ![0x10000000, 0x1f003f].contains(&value.Mask)
                    || EqualSid(principal.cast_mut(), expected) == 0
                {
                    return Err(error());
                }
                Ok(())
            })();
            LocalFree(expected);
            LocalFree(descriptor);
            result
        }
    }
    fn count(job: HANDLE) -> io::Result<u32> {
        unsafe {
            let mut value = JOBOBJECT_BASIC_ACCOUNTING_INFORMATION::default();
            let mut returned = 0;
            if QueryInformationJobObject(
                job,
                JobObjectBasicAccountingInformation,
                (&mut value as *mut JOBOBJECT_BASIC_ACCOUNTING_INFORMATION).cast(),
                std::mem::size_of_val(&value) as u32,
                &mut returned,
            ) == 0
                || returned as usize != std::mem::size_of_val(&value)
            {
                return Err(error());
            }
            Ok(value.ActiveProcesses)
        }
    }
    fn created(process: HANDLE) -> io::Result<String> {
        unsafe {
            let (mut born, mut exit, mut kernel, mut user) = (
                std::mem::zeroed(),
                std::mem::zeroed(),
                std::mem::zeroed(),
                std::mem::zeroed(),
            );
            if GetProcessTimes(process, &mut born, &mut exit, &mut kernel, &mut user) == 0 {
                return Err(error());
            }
            Ok((((born.dwHighDateTime as u64) << 32) | born.dwLowDateTime as u64).to_string())
        }
    }
    fn lease_id(file: &File) -> io::Result<(u32, String)> {
        unsafe {
            let mut info = BY_HANDLE_FILE_INFORMATION::default();
            if GetFileInformationByHandle(file.as_raw_handle(), &mut info) == 0
                || info.dwFileAttributes & 0x410 != 0
                || info.nNumberOfLinks != 1
            {
                return Err(error());
            }
            Ok((
                info.dwVolumeSerialNumber,
                (((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64).to_string(),
            ))
        }
    }
    pub(crate) struct Witness {
        process: OwnedHandle,
        job: OwnedHandle,
        lease: File,
        binding: Binding,
    }
    impl Witness {
        pub(crate) fn live(
            record: &Path,
            home: &Path,
            sid: &str,
            expected_image: &Path,
        ) -> io::Result<Self> {
            if !verify_candidate_file(record, home, sid) {
                return Err(error());
            }
            let raw = read_candidate(record).ok_or_else(error)?;
            let value: serde_json::Value = serde_json::from_slice(&raw).map_err(|_| error())?;
            let binding: Binding =
                serde_json::from_value(value["candidateBinding"].clone()).map_err(|_| error())?;
            let process = unsafe {
                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | 0x100000, 0, binding.pid)
            };
            if process.is_null() {
                return Err(error());
            }
            let process = unsafe { OwnedHandle::from_raw_handle(process) };
            token_matches(process.as_raw_handle(), sid)?;
            image_matches(process.as_raw_handle(), expected_image)?;
            if unsafe { GetProcessId(process.as_raw_handle()) } != binding.pid
                || created(process.as_raw_handle())? != binding.created
                || unsafe { WaitForSingleObject(process.as_raw_handle(), 0) } != 0x102
            {
                return Err(error());
            }
            let name = wide(std::ffi::OsStr::new(&binding.job));
            let job = unsafe { OpenJobObjectW(4 | 0x20000, 0, name.as_ptr()) };
            if job.is_null() {
                return Err(error());
            }
            let job = unsafe { OwnedHandle::from_raw_handle(job) };
            private_job(job.as_raw_handle(), sid)?;
            let mut member = 0;
            if unsafe { IsProcessInJob(process.as_raw_handle(), job.as_raw_handle(), &mut member) }
                == 0
                || member != 1
                || count(job.as_raw_handle())? < 2
            {
                return Err(error());
            }
            let lease = std::fs::OpenOptions::new()
                .read(true)
                .write(true)
                .share_mode(3)
                .custom_flags(0x200000)
                .open(home.join("runtime-lifetime.lease"))?;
            crate::infrastructure::windows_private_create::verify_runtime_record_handle(&lease)?;
            if lease_id(&lease)? != (binding.lease_volume, binding.lease_index.clone()) {
                return Err(error());
            }
            let mut overlap = unsafe { std::mem::zeroed() };
            if unsafe { LockFileEx(lease.as_raw_handle(), 3, 0, 1, 0, &mut overlap) } != 0 {
                unsafe { UnlockFileEx(lease.as_raw_handle(), 0, 1, 0, &mut overlap) };
                return Err(error());
            }
            if io::Error::last_os_error().raw_os_error() != Some(33)
                || read_candidate(record).ok_or_else(error)? != raw
            {
                return Err(error());
            }
            token_matches(process.as_raw_handle(), sid)?;
            image_matches(process.as_raw_handle(), expected_image)?;
            Ok(Self {
                process,
                job,
                lease,
                binding,
            })
        }
        pub(crate) fn exited(&self) -> io::Result<()> {
            if unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) } != 0
                || created(self.process.as_raw_handle())? != self.binding.created
                || count(self.job.as_raw_handle())? != 0
                || lease_id(&self.lease)?
                    != (self.binding.lease_volume, self.binding.lease_index.clone())
            {
                return Err(error());
            }
            private_job(self.job.as_raw_handle(), &self.binding.sid)?;
            crate::infrastructure::windows_private_create::verify_runtime_record_handle(
                &self.lease,
            )?;
            let mut overlap = unsafe { std::mem::zeroed() };
            if unsafe { LockFileEx(self.lease.as_raw_handle(), 3, 0, 1, 0, &mut overlap) } == 0 {
                return Err(error());
            }
            if unsafe { UnlockFileEx(self.lease.as_raw_handle(), 0, 1, 0, &mut overlap) } == 0 {
                return Err(error());
            }
            Ok(())
        }
    }
}
