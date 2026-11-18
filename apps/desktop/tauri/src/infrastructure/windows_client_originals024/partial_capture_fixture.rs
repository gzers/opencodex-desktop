//! Dedicated test-image admission and create-only preparation for the r7 original worker.
use super::{
    bounded::BoundedAccessV1,
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
mod isolation;
pub(crate) use isolation::Isolation;
pub(crate) use isolation::Stage;
use isolation::{canonical_pin, checked_image, Boundary};
use serde_json::{json, Value};
use std::{
    ffi::c_void,
    fs::{self, File},
    io::{self, Read, Write},
    path::{Path, PathBuf},
    sync::Arc,
    time::{Duration, Instant},
};

pub(crate) const SCOPE: &str = "isolated-r7-partial-capture-original-worker";
pub(crate) const PROFILE: &str = r"C:\Users\marenben";
const SID: &str = "S-1-5-21-587408064-856152790-2583328096-1003";
const BASE: &str = r"C:\Users\marenben\ocx024-r7-isolated-owner-nine-r1";
const PUBLIC: &str = r"C:\Users\Public\ocx-r7-isolated-owner-nine-r1";
const BUNDLE: &str =
    r"C:\Users\Public\ocx-r7-isolated-owner-nine-r1\input\runtime-bundle-0.2.24-win32-x64.tar.gz";

const BUNDLE_SHA: &str = "14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7";

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
        Err(error) if error.raw_os_error() == Some(2) => Ok(()),
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
        && token.sid == SID
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
        .chain("\\marenben\0".encode_utf16())
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
    boundary: Boundary,
    isolation: Option<Arc<Isolation>>,
}

#[test]
fn wl2_r7_partial_owner_bootstrap_rejects_wrong_locator_token_or_image() {
    let profile = Path::new(PROFILE);
    let base = Path::new(BASE);
    let token = native::TokenIdentity {
        sid: SID.into(),
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
        Path::new(r"C:\Users\Administrator")
    ));
    let mut wrong = valid();
    wrong.expected_sid = "S-1-5-21-1-2-3-1001".into();
    assert!(!bootstrap_matches(&wrong, &token, profile));
    wrong = valid();
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
        let (peer, boundary) = Boundary::admit(c)?;
        let profile = boundary.profile.path();
        let pins = [
            canonical_pin(profile, false)?,
            canonical_pin(Path::new(BASE), true)?,
            canonical_pin(&c.installation, true)?,
            canonical_pin(&c.data, true)?,
            canonical_pin(&c.home, true)?,
        ];
        let image_path = c.installation.join("ocxd.exe");
        peer.verify_image(&image_path)?;
        let image = checked_image(&image_path, &c.broker_sha256, end, true)?;
        peer.verify_image(&image_path)?;
        if end.is_some_and(|end| Instant::now() >= end) {
            return Err(io::ErrorKind::TimedOut.into());
        }
        Ok(Self {
            peer,
            pins,
            _image: image,
            boundary,
            isolation: None,
        })
    }

    pub(crate) fn prepare(&mut self, c: &EmptyBootstrap) -> io::Result<()> {
        let base = Path::new(BASE);
        let clients = base.join("clients");
        let codex = clients.join(".codex");
        let grok = clients.join(".grok");
        let config = c.home.join("config.json");
        let ledger = base.join("preparation");
        let token = &self.peer.identity().token;
        let root = base.join("coordination");
        let targets = [
            clients.clone(),
            root.clone(),
            c.installation.join("runtime"),
            config.clone(),
            ledger.clone(),
        ];
        let [_, base_pin, installation, data, home] = &self.pins;
        let input = canonical_pin(Path::new(PUBLIC).join("input").as_path(), false)?;
        let _bundle = checked_image(Path::new(BUNDLE), BUNDLE_SHA, None, false)?;
        let observations = targets
            .iter()
            .zip([base_pin, base_pin, installation, home, base_pin])
            .map(|(path, parent)| (path, isolation::fixed_leaf(parent, path)))
            .collect::<Vec<_>>();
        self.recheck(c)?;
        let preflight = json!({"sid":token.sid,"session":token.session_id,
            "elevationType":token.elevation_type,"elevated":token.elevated,"outerJobAbsent":true,"desktop":"WinSta0/Default",
            "profile":c.profile,"pid":self.peer.identity().pid,"creationFiletime":self.peer.identity().creation_filetime.to_string(),
            "imageSha256":c.broker_sha256,"imageId":self.peer.identity().executable_id.index.to_string(),
            "excludedRoots":self.boundary.excluded,
            "targets":observations.iter().map(|(path,result)| json!({"path":path,"absent":result.is_ok(),"raw":result.as_ref().err().and_then(io::Error::raw_os_error),"error":result.as_ref().err().map(ToString::to_string)})).collect::<Vec<_>>() });
        if observations.iter().any(|(_, result)| result.is_err()) {
            eprintln!("r7-fixture-preflight={preflight}");
        }
        for (_, result) in observations {
            result?;
        }
        isolation::fixed_leaf(data, &c.data.join("fixture-preflight.json"))?;
        report(&c.data.join("fixture-preflight.json"), &preflight)?;
        self.recheck(c)?;
        let ledger = isolation::create(base_pin, &ledger)?;
        let clients_pin = isolation::create(base_pin, &clients)?;
        let image = install_fixed_bundle(base, ledger.path(), Path::new(BUNDLE))?;
        input.recheck()?;
        let codex_pin = isolation::create(&clients_pin, &codex)?;
        let grok_pin = isolation::create(&clients_pin, &grok)?;
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
        let coordination_pin = isolation::create(base_pin, &root)?;
        let paths = isolation::coordination_paths(&codex, &root, &digest);
        files.extend(paths.iter().skip(1).map(|path| (path.clone(), &b""[..])));
        for (path, bytes) in files {
            let parent = [&codex_pin, &grok_pin, home, &coordination_pin]
                .into_iter()
                .find(|pin| Some(pin.path()) == path.parent())
                .ok_or_else(|| invalid("fixture file has no pinned parent"))?;
            isolation::fixed_leaf(parent, &path)?;
            let mut file = platform::create_new_private_file(&path, 0o600)?;
            file.write_all(bytes)?;
            file.sync_all()?;
            let created = super::coordination::file_id(&file)?;
            drop(file);
            let observed = isolation::open_file(&path, 3, true)?;
            if created != super::coordination::file_id(&observed)?
                || created.volume != parent.identity().volume
            {
                return Err(invalid("fixture file differs from the create-only object"));
            }
            parent.recheck()?;
            rows.push(existing_file_snapshot(
                &path,
                bytes,
                Instant::now() + Duration::from_secs(5),
            )?);
        }
        self.recheck(c)?;
        codex_pin.recheck()?;
        grok_pin.recheck()?;
        ledger.recheck()?;
        self.isolation = Some(Arc::new(Isolation::bind(
            c,
            [clients_pin, codex_pin, grok_pin, coordination_pin],
            paths,
        )?));
        let prepared = ledger.path().join("prepared.json");
        isolation::fixed_leaf(&ledger, &prepared)?;
        report(
            &prepared,
            &json!({"image":image,"syntheticObjects":rows,"officialInitializationProven":false,"publicStartOpen":false}),
        )
    }

    fn recheck(&self, c: &EmptyBootstrap) -> io::Result<()> {
        self.boundary.recheck()?;
        self.peer.no_outer_job()?;
        self.peer.verify_image(&c.installation.join("ocxd.exe"))?;
        for pin in &self.pins {
            pin.recheck()?;
        }
        Ok(())
    }

    pub(crate) fn isolation_for_start(
        &self,
        runtime: &SharedRuntimeExecutable,
        command: &ProcessCommand,
        data: &Path,
        window: &ActionWindow,
    ) -> AppResult<Arc<Isolation>> {
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
        let isolation = self.isolation.as_ref().ok_or_else(|| {
            app(invalid(
                "test Start requires a prepared native isolation description",
            ))
        })?;
        isolation.recheck(window.deadline()).map_err(app)?;
        window.remaining()?;
        Ok(isolation.clone())
    }

    pub(crate) fn pin_attachment_parent(
        &self,
        data: &Path,
        window: &ActionWindow,
    ) -> AppResult<native::PinnedDirectory> {
        let original_data = &self.pins[3];
        let parent = canonical_pin(data, true).map_err(app)?;
        if parent.identity() != original_data.identity() {
            return Err(app(invalid("original D identity drift")));
        }
        window.remaining()?;
        Ok(parent)
    }
}
