//! Current TokenUser PATH facts only. Shared terminal_cli owns PATH editing rules,
//! confirmation, journaling and recovery. No setx, HKLM, expansion or normalization.
//! The SID mutex coordinates cooperating callers across sessions, not arbitrary
//! registry writers. A post-write mismatch is an error and is never compensated.
#![cfg(windows)]

use super::windows_official_runtime024::current_identity;
use std::{ffi::c_void, io, mem::size_of, ptr};
use windows_sys::Win32::{
    Foundation::{
        CloseHandle, GetLastError, LocalFree, SetLastError, HANDLE, WAIT_ABANDONED, WAIT_OBJECT_0,
        WAIT_TIMEOUT,
    },
    Security::{GetAce, IsValidAcl, ACCESS_ALLOWED_ACE, ACE_HEADER, ACL, SECURITY_ATTRIBUTES},
    System::{
        Registry::{
            RegCloseKey, RegDeleteValueW, RegOpenCurrentUser, RegOpenKeyExW, RegQueryValueExW,
            RegSetValueExW, HKEY, KEY_QUERY_VALUE, KEY_READ, KEY_SET_VALUE, REG_EXPAND_SZ, REG_SZ,
        },
        Threading::{
            CreateMutexExW, GetCurrentThread, OpenThreadToken, ReleaseMutex, WaitForSingleObject,
        },
    },
    UI::WindowsAndMessaging::{
        SendMessageTimeoutW, HWND_BROADCAST, SMTO_ABORTIFHUNG, SMTO_ERRORONEXIT, WM_SETTINGCHANGE,
    },
};

const MAX_BYTES: usize = 256 * 1024;
const MUTEX_ACCESS: u32 = 0x001f0001;
const MUTEX_WAIT_MS: u32 = 2000;
const ENVIRONMENT: &str = "Environment";

#[link(name = "advapi32")]
unsafe extern "system" {
    fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
        text: *const u16,
        revision: u32,
        sd: *mut *mut c_void,
        length: *mut u32,
    ) -> i32;
    fn ConvertSidToStringSidW(sid: *const c_void, text: *mut *mut u16) -> i32;
    fn GetSecurityInfo(
        handle: HANDLE,
        kind: i32,
        requested: u32,
        owner: *mut *mut c_void,
        group: *mut *mut c_void,
        dacl: *mut *mut ACL,
        sacl: *mut *mut ACL,
        descriptor: *mut *mut c_void,
    ) -> u32;
    fn GetSecurityDescriptorControl(
        sd: *const c_void,
        control: *mut u16,
        revision: *mut u32,
    ) -> i32;
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct RegistryValue {
    pub value_type: u32,
    pub bytes: Vec<u8>,
}
fn invalid(message: &'static str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, message)
}
fn status(code: u32) -> io::Result<()> {
    if code == 0 {
        Ok(())
    } else {
        Err(io::Error::from_raw_os_error(code as i32))
    }
}
fn wide(text: &str) -> Vec<u16> {
    text.encode_utf16().chain(Some(0)).collect()
}
fn validate(value: &RegistryValue) -> io::Result<()> {
    if ![REG_SZ, REG_EXPAND_SZ].contains(&value.value_type)
        || value.bytes.len() < 2
        || value.bytes.len() > MAX_BYTES
        || !value.bytes.len().is_multiple_of(2)
    {
        return Err(invalid("PATH type or byte length is ambiguous"));
    }
    let units: Vec<u16> = value
        .bytes
        .as_chunks::<2>()
        .0
        .iter()
        .map(|b| u16::from_le_bytes([b[0], b[1]]))
        .collect();
    if units.last() != Some(&0)
        || units[..units.len() - 1].contains(&0)
        || String::from_utf16(&units[..units.len() - 1]).is_err()
    {
        return Err(invalid("PATH requires one terminator and valid UTF-16"));
    }
    Ok(())
}
struct Key(HKEY);
impl Drop for Key {
    fn drop(&mut self) {
        unsafe {
            RegCloseKey(self.0);
        }
    }
}
impl Key {
    fn open(subkey: &str, access: u32) -> io::Result<Self> {
        // The mutex SID comes from TokenUser of the process. Refuse thread
        // impersonation rather than reading another effective user's HKCU.
        let mut token = ptr::null_mut();
        if unsafe { OpenThreadToken(GetCurrentThread(), 8, 1, &mut token) } != 0 {
            unsafe {
                CloseHandle(token);
            }
            return Err(io::Error::new(
                io::ErrorKind::PermissionDenied,
                "thread impersonation refused",
            ));
        }
        if unsafe { GetLastError() } != 1008 {
            return Err(io::Error::last_os_error());
        }
        // RegOpenCurrentUser avoids the predefined HKCU cache and binds this
        // operation to the effective current token. No arbitrary production path.
        let mut raw = ptr::null_mut();
        status(unsafe { RegOpenCurrentUser(KEY_READ, &mut raw) })?;
        let user = Self(raw);
        let mut raw = ptr::null_mut();
        status(unsafe { RegOpenKeyExW(user.0, wide(subkey).as_ptr(), 0, access, &mut raw) })?;
        Ok(Self(raw))
    }
    fn read(&self) -> io::Result<Option<RegistryValue>> {
        let name = wide("Path");
        let mut value_type = 0;
        let mut size = 0;
        let code = unsafe {
            RegQueryValueExW(
                self.0,
                name.as_ptr(),
                ptr::null(),
                &mut value_type,
                ptr::null_mut(),
                &mut size,
            )
        };
        if code == 2 {
            return Ok(None);
        }
        status(code)?;
        if size as usize > MAX_BYTES {
            return Err(invalid("PATH exceeds byte bound"));
        }
        let first_type = value_type;
        let mut bytes = vec![0; size as usize];
        let first_size = size;
        status(unsafe {
            RegQueryValueExW(
                self.0,
                name.as_ptr(),
                ptr::null(),
                &mut value_type,
                bytes.as_mut_ptr(),
                &mut size,
            )
        })?;
        if size != first_size || value_type != first_type {
            return Err(invalid("PATH changed during read"));
        }
        let value = RegistryValue { value_type, bytes };
        validate(&value)?;
        Ok(Some(value))
    }
    fn write(&self, value: &Option<RegistryValue>) -> io::Result<()> {
        match value {
            Some(v) => status(unsafe {
                RegSetValueExW(
                    self.0,
                    wide("Path").as_ptr(),
                    0,
                    v.value_type,
                    v.bytes.as_ptr(),
                    v.bytes.len() as u32,
                )
            }),
            None => {
                let code = unsafe { RegDeleteValueW(self.0, wide("Path").as_ptr()) };
                // None -> None is handled without a write by replace_on.
                status(code)
            }
        }
    }
}
struct LocalMemory(*mut c_void);
impl Drop for LocalMemory {
    fn drop(&mut self) {
        unsafe {
            LocalFree(self.0);
        }
    }
}
fn sid_text(sid: *const c_void) -> io::Result<String> {
    if sid.is_null() {
        return Err(invalid("missing mutex owner"));
    }
    let mut text = ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid, &mut text) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let _memory = LocalMemory(text.cast());
    let n = (0..256)
        .find(|&n| unsafe { *text.add(n) == 0 })
        .ok_or_else(|| invalid("SID exceeds bound"))?;
    String::from_utf16(unsafe { std::slice::from_raw_parts(text, n) })
        .map_err(|_| invalid("invalid SID"))
}
fn verify_mutex(handle: HANDLE, sid: &str) -> io::Result<()> {
    let mut owner = ptr::null_mut();
    let mut acl = ptr::null_mut();
    let mut sd = ptr::null_mut();
    status(unsafe {
        GetSecurityInfo(
            handle,
            6,
            5,
            &mut owner,
            ptr::null_mut(),
            &mut acl,
            ptr::null_mut(),
            &mut sd,
        )
    })?;
    let _memory = LocalMemory(sd);
    let mut control = 0;
    let mut revision = 0;
    if sd.is_null()
        || sid_text(owner)? != sid
        || acl.is_null()
        || unsafe { IsValidAcl(acl) } == 0
        || unsafe { GetSecurityDescriptorControl(sd, &mut control, &mut revision) } == 0
        || control & 0x1000 == 0
    {
        return Err(invalid("mutex ownership or protection is unknown"));
    }
    let mut has_user = false;
    let mut has_system = false;
    for i in 0..unsafe { (*acl).AceCount } as u32 {
        let mut raw = ptr::null_mut();
        if unsafe { GetAce(acl, i, &mut raw) } == 0 {
            return Err(io::Error::last_os_error());
        }
        let header = unsafe { &*raw.cast::<ACE_HEADER>() };
        if header.AceType != 0 || header.AceFlags != 0 || header.AceSize < 16 {
            return Err(invalid("unexpected mutex ACE"));
        }
        let ace = unsafe { &*raw.cast::<ACCESS_ALLOWED_ACE>() };
        // The SID must fit wholly inside this ACE before a Win32 SID reader.
        let sid_start = ptr::addr_of!(ace.SidStart).cast::<u8>();
        let sid_size = 8 + 4 * unsafe { *sid_start.add(1) } as usize;
        if 8 + sid_size > header.AceSize as usize || ace.Mask != MUTEX_ACCESS {
            return Err(invalid("invalid mutex ACE rights"));
        }
        match sid_text(sid_start.cast())?.as_str() {
            s if s == sid => has_user = true,
            "S-1-5-18" => has_system = true,
            _ => return Err(invalid("foreign mutex principal")),
        }
    }
    if !has_user || !has_system {
        return Err(invalid("incomplete mutex ACL"));
    }
    Ok(())
}
struct UserMutex {
    handle: HANDLE,
    acquired: bool,
}
impl UserMutex {
    fn lock() -> io::Result<Self> {
        Self::named(&format!(
            "Global\\OpenCodeX.Desktop.024.UserPath.{}",
            current_identity()?.sid
        ))
    }
    fn named(name: &str) -> io::Result<Self> {
        let sid = current_identity()?.sid;
        let text = wide(&format!(
            "O:{sid}D:P(A;;0x001f0001;;;{sid})(A;;0x001f0001;;;SY)"
        ));
        let mut sd = ptr::null_mut();
        if unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                ptr::null_mut(),
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let _memory = LocalMemory(sd);
        let attributes = SECURITY_ATTRIBUTES {
            nLength: size_of::<SECURITY_ATTRIBUTES>() as u32,
            lpSecurityDescriptor: sd,
            bInheritHandle: 0,
        };
        let handle = unsafe { CreateMutexExW(&attributes, wide(name).as_ptr(), 0, 0x00120001) };
        if handle.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut guard = Self {
            handle,
            acquired: false,
        };
        verify_mutex(handle, &sid)?;
        match unsafe { WaitForSingleObject(handle, MUTEX_WAIT_MS) } {
            WAIT_OBJECT_0 => {
                guard.acquired = true;
                Ok(guard)
            }
            WAIT_ABANDONED => {
                guard.acquired = true;
                Err(invalid("abandoned PATH mutex; operation outcome unknown"))
            }
            WAIT_TIMEOUT => Err(io::Error::new(
                io::ErrorKind::TimedOut,
                "PATH mutex deadline",
            )),
            _ => Err(io::Error::last_os_error()),
        }
    }
}
impl Drop for UserMutex {
    fn drop(&mut self) {
        unsafe {
            if self.acquired {
                ReleaseMutex(self.handle);
            }
            CloseHandle(self.handle);
        }
    }
}

/// Returns the original type and raw bytes, including the single UTF-16 NUL.
pub fn read_user_path() -> io::Result<Option<RegistryValue>> {
    let _guard = UserMutex::lock()?;
    Key::open(ENVIRONMENT, KEY_QUERY_VALUE)?.read()
}

/// Exact expected comparison under the current SID mutex and retained key handle.
/// No atomic CAS against noncooperating writers is claimed. No rollback on error.
pub fn replace_user_path(
    expected: &Option<RegistryValue>,
    desired: &Option<RegistryValue>,
) -> io::Result<()> {
    let _guard = UserMutex::lock()?;
    let key = Key::open(ENVIRONMENT, KEY_QUERY_VALUE | KEY_SET_VALUE)?;
    replace_on(&key, expected, desired, || {})
}
fn replace_on(
    key: &Key,
    expected: &Option<RegistryValue>,
    desired: &Option<RegistryValue>,
    after_write: impl FnOnce(),
) -> io::Result<()> {
    for v in [expected, desired].into_iter().flatten() {
        validate(v)?;
    }
    if key.read()? != *expected {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "PATH differs from expected; no write",
        ));
    }
    if expected != desired {
        key.write(desired)?;
    }
    after_write();
    if key.read()? != *desired {
        return Err(io::Error::new(
            io::ErrorKind::WouldBlock,
            "PATH postimage differs; no compensation",
        ));
    }
    Ok(())
}

/// Bounded per-recipient OS-session broadcast; existing terminals keep their
/// environment. Does not expose PATH or mutate any registry/environment value.
pub fn notify_environment_changed() -> io::Result<()> {
    notify(HWND_BROADCAST)
}
fn notify(window: windows_sys::Win32::Foundation::HWND) -> io::Result<()> {
    let name = wide(ENVIRONMENT);
    unsafe {
        SetLastError(0);
    }
    if unsafe {
        SendMessageTimeoutW(
            window,
            WM_SETTINGCHANGE,
            0,
            name.as_ptr() as isize,
            SMTO_ABORTIFHUNG | SMTO_ERRORONEXIT,
            1000,
            ptr::null_mut(),
        )
    } == 0
    {
        let code = unsafe { GetLastError() };
        return Err(if code == 0 {
            io::Error::new(
                io::ErrorKind::TimedOut,
                "environment broadcast outcome unknown",
            )
        } else {
            io::Error::from_raw_os_error(code as i32)
        });
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        process::Command,
        sync::atomic::{AtomicU32, Ordering},
        time::{Duration, Instant},
    };
    use windows_sys::Win32::{
        System::Registry::{RegCreateKeyExW, RegDeleteKeyW, KEY_ALL_ACCESS, REG_BINARY, REG_DWORD},
        UI::WindowsAndMessaging::{CreateWindowExW, DestroyWindow, HWND_MESSAGE},
    };
    static NEXT: AtomicU32 = AtomicU32::new(0);
    struct Fixture {
        path: String,
        key: Option<Key>,
    }
    impl Fixture {
        fn new() -> Self {
            let path = format!(
                "Software\\OpenCodeX.Native024.Tests.{}.{}",
                std::process::id(),
                NEXT.fetch_add(1, Ordering::Relaxed)
            );
            let mut raw = ptr::null_mut();
            status(unsafe { RegOpenCurrentUser(KEY_ALL_ACCESS, &mut raw) }).unwrap();
            let user = Key(raw);
            let mut raw = ptr::null_mut();
            let mut disposition = 0;
            status(unsafe {
                RegCreateKeyExW(
                    user.0,
                    wide(&path).as_ptr(),
                    0,
                    ptr::null(),
                    0,
                    KEY_ALL_ACCESS,
                    ptr::null(),
                    &mut raw,
                    &mut disposition,
                )
            })
            .unwrap();
            assert_eq!(disposition, 1, "fresh private fixture key only");
            Self {
                path,
                key: Some(Key(raw)),
            }
        }
        fn key(&self) -> &Key {
            self.key.as_ref().unwrap()
        }
    }
    impl Drop for Fixture {
        fn drop(&mut self) {
            self.key.take();
            let mut raw = ptr::null_mut();
            status(unsafe { RegOpenCurrentUser(KEY_ALL_ACCESS, &mut raw) }).unwrap();
            let user = Key(raw);
            status(unsafe { RegDeleteKeyW(user.0, wide(&self.path).as_ptr()) }).unwrap();
            assert!(Key::open(&self.path, KEY_QUERY_VALUE).is_err());
            println!("private fixture key deleted and absence read back");
        }
    }
    fn value(kind: u32, text: &str) -> Option<RegistryValue> {
        Some(RegistryValue {
            value_type: kind,
            bytes: wide(text).iter().flat_map(|x| x.to_le_bytes()).collect(),
        })
    }
    #[test]
    fn exact_raw_types_missing_restore_and_wrong_expected() {
        let f = Fixture::new();
        let _guard = UserMutex::lock().unwrap();
        assert_eq!(f.key().read().unwrap(), None);
        let a = value(
            REG_EXPAND_SZ,
            "%USERPROFILE%\\工具;C:\\MixedCase\\;C:\\MixedCase\\",
        );
        replace_on(f.key(), &None, &a, || {}).unwrap();
        assert_eq!(f.key().read().unwrap(), a);
        let b = value(REG_SZ, "");
        assert_eq!(
            replace_on(f.key(), &b, &None, || {}).unwrap_err().kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(f.key().read().unwrap(), a);
        replace_on(f.key(), &a, &b, || {}).unwrap();
        assert_eq!(f.key().read().unwrap(), b);
        replace_on(f.key(), &b, &None, || {}).unwrap();
        replace_on(f.key(), &None, &None, || {}).unwrap();
        assert_eq!(f.key().read().unwrap(), None);
        println!("native REG_EXPAND_SZ/REG_SZ and byte-for-byte expected/absence restore passed; no PATH logged");
    }
    #[test]
    fn malformed_and_oversize_values_refused_without_normalizing() {
        let f = Fixture::new();
        let _guard = UserMutex::lock().unwrap();
        let cases = [
            RegistryValue {
                value_type: REG_BINARY,
                bytes: vec![0, 0],
            },
            RegistryValue {
                value_type: REG_DWORD,
                bytes: vec![0; 4],
            },
            RegistryValue {
                value_type: REG_SZ,
                bytes: vec![0],
            },
            RegistryValue {
                value_type: REG_SZ,
                bytes: vec![],
            },
            RegistryValue {
                value_type: REG_SZ,
                bytes: vec![65, 0],
            },
            RegistryValue {
                value_type: REG_SZ,
                bytes: vec![0, 0, 0, 0],
            },
            RegistryValue {
                value_type: REG_SZ,
                bytes: vec![0, 0xd8, 0, 0],
            },
            RegistryValue {
                value_type: REG_SZ,
                bytes: vec![0; MAX_BYTES + 2],
            },
        ];
        for bad in cases {
            assert!(replace_on(f.key(), &None, &Some(bad.clone()), || {}).is_err());
            assert_eq!(f.key().read().unwrap(), None);
            // Deliberately malformed raw fixture input via the real registry API.
            f.key().write(&Some(bad.clone())).unwrap();
            assert!(f.key().read().is_err());
            assert!(replace_on(f.key(), &None, &None, || {}).is_err());
            let mut size = 0;
            let mut kind = 0;
            status(unsafe {
                RegQueryValueExW(
                    f.key().0,
                    wide("Path").as_ptr(),
                    ptr::null(),
                    &mut kind,
                    ptr::null_mut(),
                    &mut size,
                )
            })
            .unwrap();
            assert_eq!((kind, size as usize), (bad.value_type, bad.bytes.len()));
            f.key().write(&None).unwrap();
        }
    }
    #[test]
    fn permission_unknown_and_external_postwrite_change_are_not_success() {
        let f = Fixture::new();
        let readonly = Key::open(&f.path, KEY_QUERY_VALUE).unwrap();
        let a = value(REG_SZ, "fixture-a");
        assert_eq!(
            replace_on(&readonly, &None, &a, || {})
                .unwrap_err()
                .raw_os_error(),
            Some(5)
        );
        assert_eq!(f.key().read().unwrap(), None);
        let external = value(REG_EXPAND_SZ, "external-writer");
        assert_eq!(
            replace_on(f.key(), &None, &a, || {
                f.key().write(&external).unwrap();
            })
            .unwrap_err()
            .kind(),
            io::ErrorKind::WouldBlock
        );
        assert_eq!(
            f.key().read().unwrap(),
            external,
            "external postimage must not be rolled back"
        );
        replace_on(f.key(), &external, &None, || {}).unwrap();
    }
    #[test]
    fn child_replace() {
        let Some(path) = std::env::var_os("OCXD024_TEST_SUBKEY") else {
            return;
        };
        let path = path.to_str().unwrap();
        assert!(path.starts_with("Software\\OpenCodeX.Native024.Tests."));
        let _guard = UserMutex::lock().unwrap();
        let key = Key::open(path, KEY_QUERY_VALUE | KEY_SET_VALUE).unwrap();
        match replace_on(&key, &None, &value(REG_SZ, "child"), || {}) {
            Ok(()) => println!("WRITER_CHANGED"),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => println!("WRITER_CONFLICT"),
            Err(e) => panic!("unexpected native writer error: {e}"),
        }
    }
    #[test]
    fn concurrent_same_sid_processes_get_one_expected_conflict() {
        let f = Fixture::new();
        let spawn = || {
            Command::new(std::env::current_exe().unwrap())
                .args([
                    "--exact",
                    "windows_cli024::tests::child_replace",
                    "--nocapture",
                ])
                .env("OCXD024_TEST_SUBKEY", &f.path)
                .stdout(std::process::Stdio::piped())
                .stderr(std::process::Stdio::piped())
                .spawn()
                .unwrap()
        };
        let a = spawn();
        let b = spawn();
        let outputs = [a.wait_with_output().unwrap(), b.wait_with_output().unwrap()];
        let mut changed = 0;
        let mut conflict = 0;
        for output in outputs {
            assert!(
                output.status.success(),
                "{}",
                String::from_utf8_lossy(&output.stderr)
            );
            let text = String::from_utf8_lossy(&output.stdout);
            changed += usize::from(text.contains("WRITER_CHANGED"));
            conflict += usize::from(text.contains("WRITER_CONFLICT"));
        }
        assert_eq!((changed, conflict), (1, 1));
        let current = f.key().read().unwrap();
        replace_on(f.key(), &current, &None, || {}).unwrap();
        println!("two actual same-SID processes: one write, one expected conflict");
    }
    #[test]
    fn unreadable_or_abandoned_mutex_is_unknown() {
        let name = format!(
            "Local\\OpenCodeX.Native024.MutexTest.{}",
            std::process::id()
        );
        // A mutex with the default, unverified descriptor is never adopted.
        let raw = unsafe { CreateMutexExW(ptr::null(), wide(&name).as_ptr(), 0, MUTEX_ACCESS) };
        assert!(!raw.is_null());
        assert!(UserMutex::named(&name).is_err());
        unsafe {
            CloseHandle(raw);
        }
        let keep = UserMutex::named(&name).unwrap();
        // Retain the actual acquired handle without normal release to exercise
        // WAIT_ABANDONED, then close it explicitly after the observer rejects.
        let name2 = name.clone();
        let abandoned = std::thread::spawn(move || {
            let guard = UserMutex::named(&name2).unwrap();
            let raw = guard.handle as usize;
            std::mem::forget(guard);
            raw
        });
        // Release parent ownership before the worker can acquire it.
        drop(keep);
        let raw = abandoned.join().unwrap();
        assert!(UserMutex::named(&name).is_err());
        unsafe {
            CloseHandle(raw as HANDLE);
        }
    }
    #[test]
    fn bounded_notification_and_invalid_window_failure() {
        let before = read_user_path().unwrap();
        let start = Instant::now();
        // Real message-only window belongs to this test, not the user's apps.
        let window = unsafe {
            CreateWindowExW(
                0,
                wide("STATIC").as_ptr(),
                wide("native024-notify").as_ptr(),
                0,
                0,
                0,
                0,
                0,
                HWND_MESSAGE,
                ptr::null_mut(),
                ptr::null_mut(),
                ptr::null(),
            )
        };
        assert!(!window.is_null());
        notify(window).unwrap();
        unsafe {
            DestroyWindow(window);
        }
        assert!(notify(window).is_err());
        assert!(start.elapsed() < Duration::from_secs(3));
        assert_eq!(before, read_user_path().unwrap());
        println!(
            "real WM_SETTINGCHANGE to own window and invalid-window refusal; daily PATH unchanged"
        );
    }

    #[test]
    fn hung_recipient_times_out_and_preserves_daily_path() {
        let before = read_user_path().unwrap();
        let (send, receive) = std::sync::mpsc::channel();
        let (done, wait) = std::sync::mpsc::channel();
        let worker = std::thread::spawn(move || {
            let window = unsafe {
                CreateWindowExW(
                    0,
                    wide("STATIC").as_ptr(),
                    wide("native024-hung").as_ptr(),
                    0,
                    0,
                    0,
                    0,
                    0,
                    HWND_MESSAGE,
                    ptr::null_mut(),
                    ptr::null_mut(),
                    ptr::null(),
                )
            };
            assert!(!window.is_null());
            send.send(window as usize).unwrap();
            // Deliberately no message loop; only this test-owned window hangs.
            wait.recv_timeout(Duration::from_secs(5)).unwrap();
            unsafe {
                DestroyWindow(window);
            }
        });
        let window = receive.recv_timeout(Duration::from_secs(3)).unwrap() as _;
        let start = Instant::now();
        let outcome = notify(window);
        let elapsed = start.elapsed();
        done.send(()).unwrap();
        worker.join().unwrap();
        assert!(outcome.is_err());
        assert!(elapsed < Duration::from_secs(3));
        assert_eq!(before, read_user_path().unwrap());
        println!(
            "hung test window refused after {}ms; daily PATH unchanged",
            elapsed.as_millis()
        );
    }
}
