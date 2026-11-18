//! Dedicated test-image admission and create-only preparation for the r7 original worker.
use super::{
    bounded::BoundedAccessV1,
    coordination,
    existing_qualification::{install_fixed_bundle, CONFIG},
};
use crate::{
    errors::{AppError, AppResult},
    infrastructure::{
        hash, platform, runtime_executable::SharedRuntimeExecutable,
        windows_official_runtime024 as native, windows_owner_host024::empty_host::EmptyBootstrap,
    },
    modules::{
        process::{LifecycleAction, ProcessCommand},
        runtime::{
            official_lifecycle024::ActionWindow,
            windows_lifecycle024_contract::ANCHORED_CODEX_STABLE_FILES,
        },
    },
};
use serde_json::{json, Value};
use sha2::{Digest, Sha256};
use std::{
    ffi::c_void,
    fs::{self, File, OpenOptions},
    io::{self, Read, Write},
    os::windows::fs::OpenOptionsExt,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

pub(crate) const SCOPE: &str = "isolated-r7-partial-capture-original-worker";
pub(crate) const PROFILE: &str = r"C:\Users\ocxr7owner";
pub(crate) const BASE: &str = r"C:\Users\ocxr7owner\ocx024-r7-partial-owner-r1";
const BUNDLE: &str =
    r"C:\Users\Public\ocx-r7-partial-owner-r1\input\runtime-bundle-0.2.24-win32-x64.tar.gz";

fn invalid(reason: &str) -> io::Error {
    io::Error::new(io::ErrorKind::InvalidData, reason)
}

fn app(error: io::Error) -> AppError {
    AppError::RuntimeManaged {
        code: "windows024_partial_capture_fixture_refused".into(),
        detail: error.to_string(),
    }
}

fn absent(path: &Path) -> io::Result<()> {
    match fs::symlink_metadata(path) {
        Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
        Err(error) => Err(error),
        Ok(_) => Err(io::Error::new(
            io::ErrorKind::AlreadyExists,
            format!("fixture path already present: {}", path.display()),
        )),
    }
}

fn write_new(path: &Path, bytes: &[u8]) -> io::Result<()> {
    let mut file = platform::create_new_private_file(path, 0o600)?;
    file.write_all(bytes)?;
    file.sync_all()
}

pub(crate) fn report(path: &Path, value: &Value) -> io::Result<()> {
    write_new(
        path,
        &serde_json::to_vec_pretty(value).map_err(io::Error::other)?,
    )
}

fn desktop() -> io::Result<()> {
    #[link(name = "user32")]
    unsafe extern "system" {
        fn GetProcessWindowStation() -> *mut c_void;
        fn GetThreadDesktop(thread: u32) -> *mut c_void;
        fn GetUserObjectInformationW(
            object: *mut c_void,
            index: i32,
            data: *mut c_void,
            bytes: u32,
            needed: *mut u32,
        ) -> i32;
    }
    fn name(object: *mut c_void) -> io::Result<String> {
        if object.is_null() {
            return Err(io::Error::last_os_error());
        }
        let mut value = [0u16; 256];
        let mut bytes = 0;
        if unsafe {
            GetUserObjectInformationW(
                object,
                2,
                value.as_mut_ptr().cast(),
                (value.len() * 2) as u32,
                &mut bytes,
            )
        } == 0
        {
            return Err(io::Error::last_os_error());
        }
        let len = (bytes / 2) as usize;
        if bytes % 2 != 0 || len < 2 || len > value.len() || value[len - 1] != 0 {
            return Err(invalid("invalid desktop object name"));
        }
        String::from_utf16(&value[..len - 1]).map_err(io::Error::other)
    }
    let station = name(unsafe { GetProcessWindowStation() })?;
    let thread = unsafe { windows_sys::Win32::System::Threading::GetCurrentThreadId() };
    let desk = name(unsafe { GetThreadDesktop(thread) })?;
    if !station.eq_ignore_ascii_case("WinSta0") || !desk.eq_ignore_ascii_case("Default") {
        return Err(invalid("exact interactive WinSta0/Default required"));
    }
    Ok(())
}

fn bootstrap_matches(c: &EmptyBootstrap, token: &native::TokenIdentity, profile: &Path) -> bool {
    let base = Path::new(BASE);
    c.schema == 1
        && c.scope == SCOPE
        && c.profile == Path::new(PROFILE)
        && c.profile == profile
        && token.sid == c.expected_sid
        && token.session_id != 0
        && token.session_id == c.expected_session
        && !token.elevated
        && token.elevation_type == 1
        && c.installation == base.join("I")
        && c.data == base.join("D")
        && c.home == base.join("H")
        && c.broker_sha256 == c.manager_sha256
        && c.broker_sha256.len() == 64
        && c.broker_sha256
            .bytes()
            .all(|byte| byte.is_ascii_hexdigit() && !byte.is_ascii_uppercase())
}

fn local_account_matches(token: &native::TokenIdentity) -> io::Result<()> {
    #[link(name = "kernel32")]
    unsafe extern "system" {
        fn GetComputerNameW(name: *mut u16, chars: *mut u32) -> i32;
    }
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertSidToStringSidW(sid: *const c_void, text: *mut *mut u16) -> i32;
    }
    let mut computer = [0u16; 256];
    let mut chars = computer.len() as u32;
    if unsafe { GetComputerNameW(computer.as_mut_ptr(), &mut chars) } == 0 {
        return Err(io::Error::last_os_error());
    }
    let computer = computer
        .get(..chars as usize)
        .ok_or_else(|| invalid("computer name bounds"))?;
    if computer.is_empty() {
        return Err(invalid("computer name missing"));
    }
    let account = computer
        .iter()
        .copied()
        .chain("\\ocxr7owner\0".encode_utf16())
        .collect::<Vec<_>>();
    let mut sid = [0usize; 16];
    let mut sid_bytes = std::mem::size_of_val(&sid) as u32;
    let mut domain = [0u16; 256];
    let mut domain_chars = domain.len() as u32;
    let mut usage = 0;
    if unsafe {
        windows_sys::Win32::Security::LookupAccountNameW(
            std::ptr::null(),
            account.as_ptr(),
            sid.as_mut_ptr().cast(),
            &mut sid_bytes,
            domain.as_mut_ptr(),
            &mut domain_chars,
            &mut usage,
        )
    } == 0
    {
        return Err(io::Error::last_os_error());
    }
    if usage != windows_sys::Win32::Security::SidTypeUser
        || domain.get(..computer.len()) != Some(computer)
        || domain.get(computer.len()) != Some(&0)
    {
        return Err(invalid("exact local fixture user required"));
    }
    let mut text = std::ptr::null_mut();
    if unsafe { ConvertSidToStringSidW(sid.as_ptr().cast(), &mut text) } == 0 {
        return Err(io::Error::last_os_error());
    }
    struct Local(*mut u16);
    impl Drop for Local {
        fn drop(&mut self) {
            unsafe {
                windows_sys::Win32::Foundation::LocalFree(self.0.cast());
            }
        }
    }
    let text = Local(text);
    let expected = token.sid.encode_utf16().chain(std::iter::once(0));
    if text.0.is_null()
        || !expected
            .enumerate()
            .all(|(index, word)| unsafe { *text.0.add(index) } == word)
    {
        return Err(invalid("local fixture user SID differs from process token"));
    }
    Ok(())
}

fn security_snapshot(path: &Path, directory: bool, end: Instant) -> io::Result<Value> {
    let proof = BoundedAccessV1::capture(path, directory, end).map_err(io::Error::other)?;
    let data = proof.snapshot_data()?;
    proof.recheck(end).map_err(io::Error::other)?;
    serde_json::to_value(data).map_err(io::Error::other)
}

fn existing_file_snapshot(path: &Path, expected: &[u8], end: Instant) -> io::Result<Value> {
    if Instant::now() >= end {
        return Err(io::ErrorKind::TimedOut.into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take(expected.len() as u64 + 1)
        .read_to_end(&mut bytes)?;
    if bytes != expected {
        return Err(invalid("synthetic file readback differs"));
    }
    Ok(
        json!({"path":path,"bytes":bytes.len(),"sha256":hash::sha256_hex(&bytes),"security":security_snapshot(path, false, end)?}),
    )
}

pub(crate) struct Fixture {
    peer: native::RetainedPeer,
    pins: [native::PinnedDirectory; 5],
    _image: File,
}

#[test]
fn wl2_r7_partial_owner_bootstrap_rejects_wrong_locator_token_or_image() {
    let profile = Path::new(PROFILE);
    let base = Path::new(BASE);
    let token = native::TokenIdentity {
        sid: "S-1-5-21-1-2-3-1001".into(),
        session_id: 8,
        elevated: false,
        elevation_type: 1,
    };
    let valid = || EmptyBootstrap {
        schema: 1,
        scope: SCOPE.into(),
        expected_sid: token.sid.clone(),
        expected_session: token.session_id,
        profile: profile.into(),
        installation: base.join("I"),
        data: base.join("D"),
        home: base.join("H"),
        broker_sha256: "a".repeat(64),
        manager_sha256: "a".repeat(64),
    };
    assert!(bootstrap_matches(&valid(), &token, profile));
    assert!(!bootstrap_matches(
        &valid(),
        &token,
        Path::new(r"C:\Users\marenben")
    ));
    let mut wrong = valid();
    wrong.data = base.join("H");
    assert!(!bootstrap_matches(&wrong, &token, profile));
    wrong = valid();
    wrong.expected_session += 1;
    assert!(!bootstrap_matches(&wrong, &token, profile));
    wrong = valid();
    wrong.manager_sha256 = "b".repeat(64);
    assert!(!bootstrap_matches(&wrong, &token, profile));
    let mut elevated = token.clone();
    elevated.elevated = true;
    assert!(!bootstrap_matches(&valid(), &elevated, profile));
    assert_eq!(
        existing_file_snapshot(&base.join("H/config.json"), CONFIG, Instant::now())
            .unwrap_err()
            .kind(),
        io::ErrorKind::TimedOut
    );
}

impl Fixture {
    pub(crate) fn admit(c: &EmptyBootstrap) -> io::Result<Self> {
        Self::admit_inner(c, None)
    }

    pub(crate) fn admit_client(c: &EmptyBootstrap, end: Instant) -> io::Result<Self> {
        Self::admit_inner(c, Some(end))
    }

    fn admit_inner(c: &EmptyBootstrap, end: Option<Instant>) -> io::Result<Self> {
        let peer = native::RetainedPeer::current()?;
        let profile = crate::infrastructure::windows_startup_root::current_user_profile()?;
        if !bootstrap_matches(c, &peer.identity().token, &profile) {
            return Err(invalid("isolated r7 native bootstrap refused"));
        }
        local_account_matches(&peer.identity().token)?;
        desktop()?;
        peer.no_outer_job()?;
        let pins = [
            native::PinnedDirectory::open(&profile)?,
            native::PinnedDirectory::open_private(Path::new(BASE))?,
            native::PinnedDirectory::open_private(&c.installation)?,
            native::PinnedDirectory::open_private(&c.data)?,
            native::PinnedDirectory::open_private(&c.home)?,
        ];
        let image_path = c.installation.join("ocxd.exe");
        peer.verify_image(&image_path)?;
        let mut image = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .custom_flags(0x200000)
            .open(&image_path)?;
        let mut hasher = Sha256::new();
        let mut count = 0;
        let mut buffer = [0u8; 65536];
        loop {
            if end.is_some_and(|end| Instant::now() >= end) {
                return Err(io::ErrorKind::TimedOut.into());
            }
            let read = image.read(&mut buffer)?;
            if read == 0 {
                break;
            }
            count += read;
            if count > 512 * 1024 * 1024 {
                return Err(invalid("fixture test image exceeds byte bound"));
            }
            hasher.update(&buffer[..read]);
        }
        if format!("{:x}", hasher.finalize()) != c.broker_sha256 {
            return Err(invalid("exact test-only same image hash differs"));
        }
        peer.verify_image(&image_path)?;
        if end.is_some_and(|end| Instant::now() >= end) {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(Self {
            peer,
            pins,
            _image: image,
        })
    }

    pub(crate) fn prepare(&self, c: &EmptyBootstrap) -> io::Result<()> {
        let base = Path::new(BASE);
        let codex = c.profile.join(".codex");
        let grok = c.profile.join(".grok");
        let config = c.home.join("config.json");
        let ledger = base.join("preparation");
        let token = &self.peer.identity().token;
        let root = coordination::official_runtime_root(token)?;
        let app_data = c.profile.join("AppData/Local");
        if root
            != app_data
                .join("OpenCodex/Runtime/v1")
                .join(token.sid.to_uppercase())
        {
            return Err(invalid(
                "coordinator not under isolated native LocalAppData",
            ));
        }
        let app_data_pin = native::PinnedDirectory::open(&app_data)?;
        let targets = [
            codex.clone(),
            grok.clone(),
            app_data.join("OpenCodex"),
            c.installation.join("runtime"),
            config.clone(),
            ledger.clone(),
        ];
        let observations = targets
            .iter()
            .map(|path| (path, absent(path)))
            .collect::<Vec<_>>();
        report(
            &c.data.join("fixture-preflight.json"),
            &json!({"sid":token.sid,"session":token.session_id,
            "elevationType":token.elevation_type,"elevated":token.elevated,"outerJobAbsent":true,"desktop":"WinSta0/Default",
            "profile":c.profile,"pid":self.peer.identity().pid,"creationFiletime":self.peer.identity().creation_filetime.to_string(),
            "imageSha256":c.broker_sha256,"imageId":self.peer.identity().executable_id.index.to_string(),
            "targets":observations.iter().map(|(path,result)| json!({"path":path,"absent":result.is_ok(),"raw":result.as_ref().err().and_then(io::Error::raw_os_error),"error":result.as_ref().err().map(ToString::to_string)})).collect::<Vec<_>>() }),
        )?;
        for (_, result) in observations {
            result?;
        }
        self.recheck(c)?;
        app_data_pin.recheck()?;
        let ledger = native::create_private_directory(&ledger)?;
        let image = install_fixed_bundle(base, ledger.path(), Path::new(BUNDLE))?;
        let codex_pin = native::create_private_directory(&codex)?;
        let grok_pin = native::create_private_directory(&grok)?;
        let canonical = fs::canonicalize(&codex)?;
        let canonical = dunce::simplified(&canonical);
        if canonical != codex {
            return Err(invalid("synthetic Codex canonical path differs"));
        }
        let digest = hash::sha256_hex(
            canonical
                .to_str()
                .ok_or_else(|| invalid("Codex path UTF-8"))?
                .as_bytes(),
        );
        let mut rows = Vec::new();
        for pin in [&codex_pin, &grok_pin] {
            rows.push(json!({"path":pin.path(),"security":security_snapshot(pin.path(), true, Instant::now() + Duration::from_secs(5))?}));
        }
        let mut files: Vec<(PathBuf, &[u8])> = ANCHORED_CODEX_STABLE_FILES
            .iter()
            .map(|name| (codex.join(name), &b""[..]))
            .collect();
        files.extend([
            (codex.join("config.toml"), &b"# synthetic r7 fixture\n"[..]),
            (codex.join("auth.json"), &b"{}\n"[..]),
            (grok.join("config.json"), &b"{}\n"[..]),
            (config, CONFIG),
        ]);
        for dir in [
            app_data.join("OpenCodex"),
            app_data.join("OpenCodex/Runtime"),
            app_data.join("OpenCodex/Runtime/v1"),
            root.clone(),
            root.join("native-write-locks"),
            root.join("catalog-write-locks"),
        ] {
            let pin = native::create_private_directory(&dir)?;
            pin.recheck()?;
        }
        files.extend([
            (
                root.join("native-write-locks")
                    .join(format!("{digest}.sqlite")),
                &b""[..],
            ),
            (root.join("client-desktop-lifecycle.sqlite"), &b""[..]),
            (
                root.join("catalog-write-locks")
                    .join(format!("{digest}.sqlite")),
                &b""[..],
            ),
        ]);
        for (path, bytes) in files {
            write_new(&path, bytes)?;
            rows.push(existing_file_snapshot(
                &path,
                bytes,
                Instant::now() + Duration::from_secs(5),
            )?);
        }
        self.recheck(c)?;
        codex_pin.recheck()?;
        grok_pin.recheck()?;
        app_data_pin.recheck()?;
        ledger.recheck()?;
        report(
            &ledger.path().join("prepared.json"),
            &json!({"image":image,"syntheticObjects":rows,"officialInitializationProven":false,"publicStartOpen":false}),
        )
    }

    fn recheck(&self, c: &EmptyBootstrap) -> io::Result<()> {
        self.peer.no_outer_job()?;
        self.peer.verify_image(&c.installation.join("ocxd.exe"))?;
        for pin in &self.pins {
            pin.recheck()?;
        }
        Ok(())
    }

    pub(crate) fn pin_for_start(
        &self,
        runtime: &SharedRuntimeExecutable,
        command: &ProcessCommand,
        data: &Path,
        window: &ActionWindow,
    ) -> AppResult<native::PinnedDirectory> {
        window.remaining()?;
        let [profile, _, installation, original_data, home] = &self.pins;
        self.peer.no_outer_job().map_err(app)?;
        self.peer
            .verify_image(&installation.path().join("ocxd.exe"))
            .map_err(app)?;
        for pin in &self.pins {
            pin.recheck().map_err(app)?;
        }
        if command.action != LifecycleAction::Start
            || runtime.managed_installation_root().as_deref() != Some(installation.path())
            || command.working_directory != profile.path()
            || command.environment.opencodex_home != home.path()
            || data != original_data.path()
        {
            return Err(app(invalid("original Start/managed roots drift")));
        }
        existing_file_snapshot(&home.path().join("config.json"), CONFIG, window.deadline())
            .map_err(app)?;
        let parent = native::PinnedDirectory::open_private(data).map_err(app)?;
        if parent.identity() != original_data.identity() {
            return Err(app(invalid("original D identity drift")));
        }
        window.remaining()?;
        Ok(parent)
    }
}
