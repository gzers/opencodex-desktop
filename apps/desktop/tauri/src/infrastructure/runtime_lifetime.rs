//! Authenticated per-HOME Windows lifetime consumer. No global process scan,
//! forced termination, ACL repair, missing-as-stopped or reusable credentials.
use base64::{engine::general_purpose::URL_SAFE_NO_PAD, Engine};
use hmac::{Hmac, Mac};
use serde::Deserialize;
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};
#[derive(Clone, Debug, Deserialize, serde::Serialize, PartialEq, Eq)]
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
pub(crate) struct SignedChallenge<'a> {
    pub secret: &'a str,
    pub challenge: &'a str,
    pub port: u16,
    pub proof: &'a str,
}
impl Binding {
    pub(crate) fn verify_proof(
        &self,
        home: &Path,
        sid: &str,
        pid: u32,
        signed: &SignedChallenge<'_>,
    ) -> bool {
        self.authenticates(&Expected {
            home,
            sid,
            pid,
            created: &self.created,
            secret: signed.secret,
            challenge: signed.challenge,
            port: signed.port,
            proof: signed.proof,
        })
    }

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
                IsProcessInJob, JobObjectBasicAccountingInformation,
                JobObjectExtendedLimitInformation, OpenJobObjectW, QueryInformationJobObject,
                JOBOBJECT_BASIC_ACCOUNTING_INFORMATION, JOBOBJECT_EXTENDED_LIMIT_INFORMATION,
            },
            Threading::{
                GetExitCodeProcess, GetProcessId, GetProcessTimes, OpenProcess, OpenProcessToken,
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
        io::Error::other("Native lifetime proof unavailable; Stop remains unconfirmed")
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
    fn image_in_installation(process: HANDLE, installation: &Path) -> io::Result<()> {
        let mut buffer = vec![0u16; 32768];
        let mut length = buffer.len() as u32;
        if unsafe { QueryFullProcessImageNameW(process, 0, buffer.as_mut_ptr(), &mut length) } == 0
        {
            return Err(error());
        }
        let actual = PathBuf::from(std::ffi::OsString::from_wide(&buffer[..length as usize]));
        let actual = key(&std::fs::canonicalize(actual)?);
        for relative in [
            "runtime/bun/bun.exe",
            "runtime/opencodex/node_modules/bun/bin/bun.exe",
            "runtime/opencodex/node_modules/@oven/bun-windows-x64/bin/bun.exe",
        ] {
            let expected = installation.join(relative);
            if expected.is_file() && key(&std::fs::canonicalize(&expected)?) == actual {
                crate::modules::backup::safety::check_path(installation, &expected, false)
                    .map_err(|_| error())?;
                let image = crate::modules::backup::safety::open_file(&expected, false)
                    .map_err(|_| error())?;
                crate::modules::backup::safety::require_single_link(&image).map_err(|_| error())?;
                return Ok(());
            }
        }
        Err(error())
    }
    fn wide(value: &std::ffi::OsStr) -> Vec<u16> {
        value.encode_wide().chain(Some(0)).collect()
    }
    fn private_job(job: HANDLE, sid: &str) -> io::Result<()> {
        unsafe {
            let mut limits = JOBOBJECT_EXTENDED_LIMIT_INFORMATION::default();
            let mut returned = 0;
            if QueryInformationJobObject(
                job,
                JobObjectExtendedLimitInformation,
                (&mut limits as *mut JOBOBJECT_EXTENDED_LIMIT_INFORMATION).cast(),
                std::mem::size_of_val(&limits) as u32,
                &mut returned,
            ) == 0
                || returned as usize != std::mem::size_of_val(&limits)
                || limits.BasicLimitInformation.LimitFlags != 0
            {
                return Err(error());
            }

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
                if owner.is_null()
                    || EqualSid(owner, expected) == 0
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
        _parent_pins: Vec<File>,
        lease_claimed: bool,
    }
    impl Witness {
        pub(crate) fn live(
            binding: Binding,
            home: &Path,
            sid: &str,
            installation: &Path,
        ) -> io::Result<Self> {
            if key(home) != key(&binding.home) || binding.sid != sid {
                return Err(error());
            }
            let parents = crate::infrastructure::windows_private_create::pin_parents(
                &home.join("runtime-lifetime.lease"),
            )?;
            let process = unsafe {
                OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION | 0x100000, 0, binding.pid)
            };
            if process.is_null() {
                return Err(error());
            }
            let process = unsafe { OwnedHandle::from_raw_handle(process) };
            token_matches(process.as_raw_handle(), sid)?;
            image_in_installation(process.as_raw_handle(), installation)?;
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
                || count(job.as_raw_handle())? < 1
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
            if io::Error::last_os_error().raw_os_error() != Some(33) {
                return Err(error());
            }
            token_matches(process.as_raw_handle(), sid)?;
            image_in_installation(process.as_raw_handle(), installation)?;
            Ok(Self {
                process,
                job,
                lease,
                binding,
                _parent_pins: parents,
                lease_claimed: false,
            })
        }
        pub(crate) fn quiescent(&mut self) -> io::Result<bool> {
            let state = unsafe { WaitForSingleObject(self.process.as_raw_handle(), 0) };
            if state != 0 && state != 0x102 {
                return Err(error());
            }
            if created(self.process.as_raw_handle())? != self.binding.created
                || lease_id(&self.lease)?
                    != (self.binding.lease_volume, self.binding.lease_index.clone())
            {
                return Err(error());
            }
            private_job(self.job.as_raw_handle(), &self.binding.sid)?;
            crate::infrastructure::windows_private_create::verify_runtime_record_handle(
                &self.lease,
            )?;
            let active = count(self.job.as_raw_handle())?;
            if state == 0x102 || active != 0 {
                return Ok(false);
            }
            let mut code = 0;
            if unsafe { GetExitCodeProcess(self.process.as_raw_handle(), &mut code) } == 0
                || code != 0
            {
                return Err(error());
            }
            if !self.lease_claimed {
                let mut overlap = unsafe { std::mem::zeroed() };
                if unsafe { LockFileEx(self.lease.as_raw_handle(), 3, 0, 1, 0, &mut overlap) } == 0
                {
                    return if io::Error::last_os_error().raw_os_error() == Some(33) {
                        Ok(false)
                    } else {
                        Err(error())
                    };
                }
                // Keep exclusive byte ownership until all record/socket/manager
                // readbacks complete. A replacement cannot race an unlock here.
                self.lease_claimed = true;
            }
            Ok(true)
        }
    }
    impl Drop for Witness {
        fn drop(&mut self) {
            if self.lease_claimed {
                let mut overlap = unsafe { std::mem::zeroed() };
                unsafe { UnlockFileEx(self.lease.as_raw_handle(), 0, 1, 0, &mut overlap) };
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> (Binding, String, String) {
        let home = std::env::temp_dir().join("ocxd-lifetime-signature-fixture");
        let sid = "S-1-5-21-100-200-300-1001".to_string();
        let mut hash = Sha256::new();
        hash.update(sid.as_bytes());
        hash.update([0]);
        hash.update(key(&home).as_bytes());
        let value = Binding {
            schema: "ocxd-runtime-lifetime/1".into(),
            home,
            sid,
            pid: 42,
            created: "123456789".into(),
            generation: "6d72c769-ed86-4638-9ce4-6703134f9381".into(),
            job: format!(
                "Global{}opencodex-runtime-{:x}",
                char::from(92),
                hash.finalize()
            ),
            lease_volume: 123,
            lease_index: "456".into(),
        };
        (
            value,
            URL_SAFE_NO_PAD.encode([7u8; 32]),
            URL_SAFE_NO_PAD.encode([1u8; 32]),
        )
    }
    fn sign(value: &Binding, secret: &str, challenge: &str, port: u16) -> String {
        let tuple = serde_json::json!([
            value.schema,
            value.home,
            value.sid,
            value.pid,
            value.created,
            value.generation,
            value.job,
            value.lease_volume,
            value.lease_index
        ]);
        let mut mac = Hmac::<Sha256>::new_from_slice(secret.as_bytes()).unwrap();
        mac.update(
            format!(
                "opencodex-runtime-lifetime-v1\n{challenge}\n{port}\n{}",
                serde_json::to_string(&tuple).unwrap()
            )
            .as_bytes(),
        );
        URL_SAFE_NO_PAD.encode(mac.finalize().into_bytes())
    }
    #[test]
    fn fixed_binding_rejects_every_mutated_field_and_request_context() {
        let (value, secret, challenge) = fixture();
        let proof = sign(&value, &secret, &challenge, 55581);
        let signed = SignedChallenge {
            secret: &secret,
            challenge: &challenge,
            port: 55581,
            proof: &proof,
        };
        assert!(value.verify_proof(&value.home, &value.sid, 42, &signed));
        for (field, bad) in [
            ("schema", serde_json::json!("other")),
            ("home", serde_json::json!(value.home.join("other"))),
            ("sid", serde_json::json!("S-1-5-21-9")),
            ("pid", serde_json::json!(43)),
            ("created", serde_json::json!("987654321")),
            (
                "generation",
                serde_json::json!(uuid::Uuid::new_v4().to_string()),
            ),
            ("job", serde_json::json!("Global\\other")),
            ("leaseVolume", serde_json::json!(124)),
            ("leaseIndex", serde_json::json!("457")),
        ] {
            let mut row = serde_json::to_value(&value).unwrap();
            row[field] = bad;
            let changed: Binding = serde_json::from_value(row).unwrap();
            assert!(
                !changed.verify_proof(&value.home, &value.sid, 42, &signed),
                "{field}"
            );
        }
        assert!(!value.verify_proof(&value.home.join("other"), &value.sid, 42, &signed));
        assert!(!value.verify_proof(&value.home, "S-1-5-21-9", 42, &signed));
        assert!(!value.verify_proof(&value.home, &value.sid, 43, &signed));
        let other = URL_SAFE_NO_PAD.encode([8u8; 32]);
        for changed in [
            SignedChallenge {
                secret: &other,
                ..signed
            },
            SignedChallenge {
                challenge: &other,
                ..signed
            },
            SignedChallenge {
                port: 55582,
                ..signed
            },
            SignedChallenge {
                proof: &other,
                ..signed
            },
        ] {
            assert!(!value.verify_proof(&value.home, &value.sid, 42, &changed));
        }
    }
    #[test]
    fn signed_but_wrong_job_schema_or_malformed_identity_never_gains_authority() {
        let (value, secret, challenge) = fixture();
        for (field, bad) in [
            ("schema", "other"),
            ("job", "Global\\wrong"),
            ("created", "not-a-time"),
            ("leaseIndex", "not-an-id"),
            ("generation", "not-a-uuid"),
        ] {
            let mut row = serde_json::to_value(&value).unwrap();
            row[field] = serde_json::json!(bad);
            let changed: Binding = serde_json::from_value(row).unwrap();
            let proof = sign(&changed, &secret, &challenge, 55581);
            let signed = SignedChallenge {
                secret: &secret,
                challenge: &challenge,
                port: 55581,
                proof: &proof,
            };
            assert!(
                !changed.verify_proof(&value.home, &value.sid, 42, &signed),
                "{field}"
            );
        }
        let mut row = serde_json::to_value(&value).unwrap();
        row["unknownField"] = serde_json::json!(true);
        assert!(serde_json::from_value::<Binding>(row).is_err());
        let mut row = serde_json::to_value(&value).unwrap();
        row.as_object_mut().unwrap().remove("leaseIndex");
        assert!(serde_json::from_value::<Binding>(row).is_err());
    }
}
