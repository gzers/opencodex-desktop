use super::*;
mod return_validation_tests;
use crate::infrastructure::{
    hash,
    windows_official_runtime024::{create_private_directory, FileIdentity, PrivateRecord},
};
use std::{
    fs::{File, OpenOptions},
    os::windows::{fs::OpenOptionsExt, io::AsRawHandle, process::CommandExt},
    path::{Path, PathBuf},
    process::{Command, Stdio},
    time::Duration,
};
fn identity(file: &File) -> FileIdentity {
    use windows_sys::Win32::Storage::FileSystem::{
        GetFileInformationByHandle, BY_HANDLE_FILE_INFORMATION,
    };
    let mut info: BY_HANDLE_FILE_INFORMATION = unsafe { std::mem::zeroed() };
    assert_ne!(
        unsafe { GetFileInformationByHandle(file.as_raw_handle(), &mut info) },
        0
    );
    FileIdentity {
        volume: info.dwVolumeSerialNumber,
        index: ((info.nFileIndexHigh as u64) << 32) | info.nFileIndexLow as u64,
    }
}
struct SyntheticOriginal {
    binding: TransferBinding,
    file: File,
    id: FileIdentity,
    bytes: Vec<u8>,
    revoked: bool,
}
impl Original for SyntheticOriginal {
    fn binding(&mut self, w: &BeforeResumeRecoveryWindow) -> io::Result<TransferBinding> {
        if self.revoked || self.binding.end != w.deadline() || identity(&self.file) != self.id {
            return Err(invalid());
        }
        use std::io::{Read, Seek, SeekFrom};
        self.file.seek(SeekFrom::Start(0))?;
        let mut raw = Vec::new();
        self.file.read_to_end(&mut raw)?;
        if raw != self.bytes {
            return Err(invalid());
        }
        Ok(TransferBinding {
            session: self.binding.session.clone(),
            action: self.binding.action.clone(),
            scope: self.binding.scope.clone(),
            end: self.binding.end,
        })
    }
    fn revoke(&mut self) {
        self.revoked = true;
    }
}
fn pathset(root: &Path) -> [PathBuf; 4] {
    [
        root.join(".codex/.opencodex-native-main.claim.sqlite"),
        root.join("coordination/n.sqlite"),
        root.join("coordination/l.sqlite"),
        root.join("coordination/k.sqlite"),
    ]
}
fn locked(path: &Path) -> bool {
    use windows_sys::Win32::{
        Storage::FileSystem::{
            LockFileEx, UnlockFileEx, LOCKFILE_EXCLUSIVE_LOCK, LOCKFILE_FAIL_IMMEDIATELY,
        },
        System::IO::OVERLAPPED,
    };
    let file = OpenOptions::new()
        .read(true)
        .share_mode(3)
        .open(path)
        .unwrap();
    let mut overlap: OVERLAPPED = unsafe { std::mem::zeroed() };
    let offset = if path.file_name().unwrap() == ".opencodex-native-main.claim.sqlite" {
        0x40000000
    } else {
        0x40000001
    };
    overlap.Anonymous.Anonymous.Offset = offset;
    let ok = unsafe {
        LockFileEx(
            file.as_raw_handle(),
            LOCKFILE_EXCLUSIVE_LOCK | LOCKFILE_FAIL_IMMEDIATELY,
            0,
            1,
            0,
            &mut overlap,
        )
    };
    if ok == 0 {
        assert_eq!(io::Error::last_os_error().raw_os_error(), Some(33));
        true
    } else {
        assert_ne!(
            unsafe { UnlockFileEx(file.as_raw_handle(), 0, 1, 0, &mut overlap) },
            0
        );
        false
    }
}
fn save(root: &Path, name: &str, data: &serde_json::Value) {
    let tmp = root.join(format!("{name}.tmp"));
    std::fs::write(&tmp, serde_json::to_vec(data).unwrap()).unwrap();
    std::fs::rename(tmp, root.join(name)).unwrap();
}
fn complete_probe(control: &WorkerControl<SyntheticOriginal>) -> serde_json::Value {
    let end = Instant::now() + Duration::from_secs(5);
    loop {
        let v = control.observe(end).unwrap();
        if v["phase"] != "Registered" && v["phase"] != "Running" {
            return v;
        }
        assert!(Instant::now() < end);
        std::thread::sleep(Duration::from_millis(5));
    }
}

pub(in crate::infrastructure::windows_client_originals024) fn assert_pending_attachment_projection(
    records: &[Vec<u8>; 2],
    outcome: io::Result<()>,
) {
    let started = Instant::now();
    let end = started + Duration::from_secs(5);
    let window = BeforeResumeRecoveryWindow::begin(started, end).unwrap();
    let session = uuid::Uuid::new_v4().to_string();
    let scope = hash::sha256_hex(b"synthetic pending attachment scope");
    let (done, parked) = mpsc::channel::<()>();
    let worker = Arc::new(OnceLock::from(thread::spawn(move || {
        let _ = parked.recv();
    })));
    let (control, _) = WorkerControl::register_retained_start(
        session.clone(),
        scope.clone(),
        window,
        worker.clone(),
    );
    let durable = outcome.is_ok();
    let error = outcome.as_ref().err();
    let expected = serde_json::json!({
        "intentDigest": hash::sha256_hex(&records[0]),
        "pendingDigest": hash::sha256_hex(&records[1]),
        "startRawError": 64,
        "attachmentAttempted": true,
        "attachmentRawError": error.and_then(io::Error::raw_os_error),
        "attachmentErrorKind": error.map(|error| format!("{:?}", error.kind())),
        "durable": durable,
    });
    {
        let mut state = control.state.lock().unwrap();
        state.phase = Phase::Running;
        state.preparation_pending = Some(PreparationPending {
            records: records.clone(),
            start_raw: Some(64),
            attachment: Some(outcome),
        });
    }
    assert!(control.observe(end).unwrap()["preparationPending"].is_null());
    for phase in [Phase::FailedRetained, Phase::ExpiredRetained] {
        control.state.lock().unwrap().phase = phase;
        let status = control.observe(end).unwrap();
        assert_eq!(status["phase"], format!("{phase:?}"));
        assert_eq!(status["sessionId"], session);
        assert_eq!(status["scopeDigest"], scope);
        assert_eq!(status["preparationPending"], expected);
        assert_eq!(status["recoveryRequired"], true);
        for flag in ["normalStop", "closed", "runtimeAuthority"] {
            assert_eq!(status[flag], false);
        }
    }
    control.state.lock().unwrap().phase = Phase::LostWorkerUnknown;
    assert!(control.observe(end).unwrap()["preparationPending"].is_null());
    assert_eq!(
        control.observe(started).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    drop(done);
    drop(control);
    Arc::try_unwrap(worker)
        .unwrap()
        .into_inner()
        .unwrap()
        .join()
        .unwrap();
    println!("r7-pending worker Status fixture projection={expected}");
}
#[test]
#[ignore = "explicit subprocess fixture only; parent tests invoke this exact entry"]
fn wl2_r7_worker_subprocess_fixture() {
    let root = PathBuf::from(std::env::var_os("OCX024_WORKER_FIXTURE_ROOT").unwrap());
    let mode = std::env::var("OCX024_WORKER_FIXTURE_MODE").unwrap();
    assert!(root.is_absolute());
    assert!(
        mode == "success"
            || mode == "failure"
            || mode == "expired"
            || mode == "panic"
            || mode == "disconnect"
            || mode == "release-partial"
            || mode == "release-expired"
    );
    drop(create_private_directory(&root.join(".codex")).unwrap());
    drop(create_private_directory(&root.join("coordination")).unwrap());
    let paths = pathset(&root);
    for p in &paths {
        drop(PrivateRecord::create(p, b"", 16).unwrap());
    }
    drop(create_private_directory(&root.join("source")).unwrap());
    let original_path = root.join("source/original-source.bin");
    drop(
        PrivateRecord::create(
            &original_path,
            b"original native worker source, never changed",
            1024,
        )
        .unwrap(),
    );
    let file = OpenOptions::new()
        .read(true)
        .share_mode(1)
        .open(&original_path)
        .unwrap();
    let id = identity(&file);
    let started = Instant::now();
    let parent = started
        + if mode == "expired" || mode == "release-expired" {
            Duration::from_millis(250)
        } else {
            Duration::from_secs(10)
        };
    let window = BeforeResumeRecoveryWindow::begin(started, parent).unwrap();
    let mut slot = Some(SyntheticOriginal {
        binding: TransferBinding {
            session: uuid::Uuid::new_v4().to_string(),
            action: uuid::Uuid::new_v4().to_string(),
            scope: hash::sha256_hex(b"actual synthetic source binding"),
            end: window.deadline(),
        },
        file,
        id,
        bytes: b"original native worker source, never changed".to_vec(),
        revoked: false,
    });
    let parent_thread = format!("{:?}", thread::current().id());
    let worker_root = root.clone();
    let task_mode = mode.clone();
    let control=WorkerControl::start(&mut slot,window,move|original,local,window|{
        assert_ne!(format!("{:?}",thread::current().id()),parent_thread);
        original.binding(window)?;local.preparation.prepare_fixture_paths(&paths,window.deadline())?;local.lease=Some(local.preparation.take_complete()?);
        let actual_thread=format!("{:?}",thread::current().id());
        save(&worker_root,"entered.json",&serde_json::json!({"thread":actual_thread,"originalFileId":original.id.index.to_string(),"ownsActualLease":true}));
        match task_mode.as_str(){
            "success"=>local.release(window.deadline()),
            "release-partial"=>{crate::infrastructure::windows_client_originals024::coordination::fail_recovery_release_after(1);local.release(window.deadline())},
            "release-expired"=>{while Instant::now()<window.deadline(){std::thread::sleep(Duration::from_millis(5));}local.release(window.deadline())},
            "expired"=>{while Instant::now()<window.deadline(){std::thread::sleep(Duration::from_millis(5));}Err(io::ErrorKind::TimedOut.into())},
            "panic"=>panic!("injected worker callback panic; native lease must remain outside unwinding"),
            _=>Err(io::Error::from_raw_os_error(33)),
        }
    }).unwrap();
    assert!(
        slot.is_none(),
        "same original source transferred uniquely before active lease"
    );
    let first = complete_probe(&control);
    assert_eq!(
        control.observe(Instant::now()).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    let entered: serde_json::Value =
        serde_json::from_slice(&std::fs::read(root.join("entered.json")).unwrap()).unwrap();
    assert_eq!(entered["thread"], first["workerThread"]);
    if mode == "success" {
        let returned = control.take_returned().unwrap().unwrap();
        let mut released = returned.released;
        released.readback_history(parent).unwrap();
        assert_eq!(identity(&returned.original.file), id);
        assert!(!returned.original.revoked);
        assert!(!matches!(control.take_returned(), Ok(Some(_))));
        for p in pathset(&root) {
            assert!(!locked(&p));
        }
        save(
            &root,
            "observed.json",
            &serde_json::json!({"state":first,"sameOriginalNativeReturned":true,"nativeExplicitReleaseChecked":true,"noNormalStopOrClosed":true}),
        );
        drop(released);
        drop(returned.original);
        return;
    }
    assert_ne!(first["phase"], "ReturnedForValidation");
    assert_eq!(first["workerFinished"], false);
    let ranges = pathset(&root).iter().map(|p| locked(p)).collect::<Vec<_>>();
    assert_eq!(
        ranges,
        if mode == "release-partial" {
            vec![true, true, true, false]
        } else {
            vec![true; 4]
        }
    );
    if mode == "panic" {
        assert_eq!(first["phase"], "LostWorkerUnknown");
    }
    if mode == "expired" || mode == "release-expired" {
        assert_eq!(first["phase"], "ExpiredRetained");
        assert_eq!(first["originalDeadlineExpired"], true);
    }
    if mode == "disconnect" {
        let observation = control.state.clone();
        drop(control);
        std::thread::sleep(Duration::from_millis(80));
        for p in pathset(&root) {
            assert!(locked(&p));
        }
        let s = observation.try_lock().unwrap();
        assert_eq!(format!("{:?}", s.phase), first["phase"].as_str().unwrap());
        assert_eq!(s.binding.as_ref().unwrap().action, first["actionId"].as_str().unwrap());
        save(
            &root,
            "observed.json",
            &serde_json::json!({"state":first,"observerDropped":true,"rangesStillHeld":true,"sameAction":true}),
        );
    } else {
        std::thread::sleep(Duration::from_millis(50));
        let second = control
            .observe(Instant::now() + Duration::from_secs(5))
            .unwrap();
        assert_eq!(first["actionId"], second["actionId"]);
        assert_eq!(first["workerThread"], second["workerThread"]);
        assert!(control.take_returned().unwrap().is_none());
        save(
            &root,
            "observed.json",
            &serde_json::json!({"state":second,"actualRangesHeld":ranges,"sameAction":true,"newWindowCreated":false}),
        );
    }
    loop {
        thread::park();
    }
}
struct Scene(Option<tempfile::TempDir>);
impl Scene {
    fn new() -> Self {
        Self(Some(
            tempfile::Builder::new()
                .prefix("ocx024-r7-worker-test-")
                .tempdir()
                .unwrap(),
        ))
    }
    fn path(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(temp) = self.0.take() {
            if std::thread::panicking() {
                println!(
                    "r7-worker-failed-fixture-retained={}",
                    temp.keep().display()
                );
            } else {
                let path = temp.path().to_path_buf();
                temp.close().unwrap();
                assert!(!path.exists());
                println!("r7-worker-cleanup-absent={}", path.display());
            }
        }
    }
}
fn run_case(mode: &str) {
    let scene = Scene::new();
    let out = File::create(scene.path().join("child.stdout.log")).unwrap();
    let err = File::create(scene.path().join("child.stderr.log")).unwrap();
    let filter="infrastructure::windows_client_originals024::recovery_worker::tests::wl2_r7_worker_subprocess_fixture";
    let mut child = Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            filter,
            "--ignored",
            "--test-threads=1",
            "--nocapture",
        ])
        .env("OCX024_WORKER_FIXTURE_ROOT", scene.path())
        .env("OCX024_WORKER_FIXTURE_MODE", mode)
        .stdin(Stdio::null())
        .stdout(Stdio::from(out))
        .stderr(Stdio::from(err))
        .creation_flags(0x08000000)
        .spawn()
        .unwrap();
    let wait = Instant::now() + Duration::from_secs(15);
    while !scene.path().join("observed.json").exists() {
        if let Some(code) = child.try_wait().unwrap() {
            panic!(
                "native worker fixture exited before observation mode{mode}: {code}; {}",
                scene.path().display()
            );
        }
        assert!(
            Instant::now() < wait,
            "same child native fixture observation timeout; retain scene and child, do not restart"
        );
        std::thread::sleep(Duration::from_millis(20));
    }
    let observed: serde_json::Value =
        serde_json::from_slice(&std::fs::read(scene.path().join("observed.json")).unwrap())
            .unwrap();
    assert_eq!(
        std::fs::read(scene.path().join("source/original-source.bin")).unwrap(),
        b"original native worker source, never changed"
    );
    let paths = pathset(scene.path());
    for (role, p) in paths.iter().enumerate() {
        assert!(std::fs::read(p).unwrap().is_empty());
        assert_eq!(
            locked(p),
            mode != "success" && !(mode == "release-partial" && role == 3)
        );
    }
    let raw = if mode == "success" {
        let code = child.wait().unwrap();
        assert!(code.success());
        0
    } else {
        assert!(child.try_wait().unwrap().is_none());
        // Exact newly-created own subprocess handle, separate forced fixture
        // disposal only. It never produces worker release/AbortRestored/Closed.
        assert_ne!(
            unsafe {
                windows_sys::Win32::System::Threading::TerminateProcess(child.as_raw_handle(), 242)
            },
            0
        );
        assert_eq!(child.wait().unwrap().code(), Some(242));
        242
    };
    if let Some(evidence) = std::env::var_os("OCX024_WORKER_TEST_EVIDENCE") {
        let dest = PathBuf::from(evidence).join(mode);
        std::fs::create_dir(&dest).unwrap();
        for name in [
            "entered.json",
            "observed.json",
            "child.stdout.log",
            "child.stderr.log",
        ] {
            std::fs::copy(scene.path().join(name), dest.join(name)).unwrap();
        }
        std::fs::write(dest.join("cleanup.json"),serde_json::to_vec_pretty(&serde_json::json!({"ownFixtureRaw":raw,"sourceBytesUnchanged":true,"fourLockFilesUnchanged":true,"forcedDisposalIsNotWorkerRelease":mode!="success","workerState":observed,"currentR2Touched":false})).unwrap()).unwrap();
    }
    println!("r7 actual owning-worker {mode}: native original thread/source and4role ranges observed; ownfixture raw{raw}, no normalStop/Closed/currentR2 actions");
    drop(child);
    drop(scene);
}
#[test]
fn wl2_r7_worker_native_success_returns_same_original_after_explicit_native_release() {
    run_case("success");
}
#[test]
fn wl2_r7_worker_native_failure_expiry_panic_and_disconnect_keep_actual_ranges_on_original_thread()
{
    for mode in [
        "failure",
        "expired",
        "panic",
        "disconnect",
        "release-partial",
        "release-expired",
    ] {
        run_case(mode);
    }
}
#[test]
fn wl2_r7_worker_native_local_stays_non_send_and_control_original_is_send() {
    fn send<T: Send>() {}
    send::<WorkerControl<Owner>>();
    send::<Owner>();
    send::<ReleasedLease>();
    trait Ambiguous<A> {
        fn verify() {}
    }
    impl<T: ?Sized> Ambiguous<()> for T {}
    impl<T: ?Sized + Send> Ambiguous<u8> for T {}
    let _ = <WorkerLocal as Ambiguous<_>>::verify;
    let _ = <NativeLease as Ambiguous<_>>::verify;
}
