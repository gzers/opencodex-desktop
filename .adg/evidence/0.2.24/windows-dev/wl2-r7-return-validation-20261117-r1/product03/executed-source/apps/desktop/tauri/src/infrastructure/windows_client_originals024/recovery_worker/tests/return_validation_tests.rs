use super::*;
struct Scene(Option<tempfile::TempDir>);
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(root) = self.0.take() {
            if thread::panicking() {
                println!(
                    "r7-return-validation-worker-failed-root={}",
                    root.keep().display()
                );
            } else {
                let path = root.path().to_path_buf();
                root.close().unwrap();
                assert!(!path.exists());
                println!(
                    "r7-return-validation-worker-cleanup-absent={}",
                    path.display()
                );
            }
        }
    }
}
#[test]
fn wl2_r7_return_validation_native_worker_retains_original_before_failure_and_preserves_worker_identity(
) {
    for mode in ["valid", "callback-raw5", "wrong-cutoff", "expired"] {
        let scene = Scene(Some(tempfile::tempdir().unwrap()));
        let root = scene.0.as_ref().unwrap().path();
        drop(create_private_directory(&root.join(".codex")).unwrap());
        drop(create_private_directory(&root.join("coordination")).unwrap());
        let paths = pathset(root);
        for path in &paths {
            drop(PrivateRecord::create(path, b"", 16).unwrap());
        }
        drop(create_private_directory(&root.join("source")).unwrap());
        let source = root.join("source/source.bin");
        drop(PrivateRecord::create(&source, b"same actual original source", 128).unwrap());
        let file = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&source)
            .unwrap();
        let id = identity(&file);
        let start = Instant::now();
        let parent = start
            + if mode == "expired" {
                Duration::from_millis(500)
            } else {
                Duration::from_secs(10)
            };
        let window = BeforeResumeRecoveryWindow::begin(start, parent).unwrap();
        let end = window.deadline();
        let mut original = Some(SyntheticOriginal {
            binding: TransferBinding {
                session: uuid::Uuid::new_v4().to_string(),
                action: uuid::Uuid::new_v4().to_string(),
                scope: hash::sha256_hex(b"returned own original source"),
                end,
            },
            file,
            id,
            bytes: b"same actual original source".to_vec(),
            revoked: false,
        });
        let control = WorkerControl::start(&mut original, window, move |_, local, window| {
            local
                .preparation
                .prepare_fixture_paths(&paths, window.deadline())?;
            local.lease = Some(local.preparation.take_complete()?);
            local.release(window.deadline())
        })
        .unwrap();
        assert!(original.is_none());
        let before = complete_probe(&control);
        assert_eq!(before["phase"], "ReturnedForValidation");
        if mode == "expired" {
            while Instant::now() < end {
                thread::sleep(Duration::from_millis(1));
            }
        }
        let cutoff = if mode == "wrong-cutoff" {
            end + Duration::from_nanos(1)
        } else {
            end
        };
        let result = control.validate_returned_for_test(cutoff, |actual, released, window| {
            assert_eq!(identity(&actual.file), id);
            assert_eq!(window.deadline(), end);
            released.check(end)?;
            if mode == "callback-raw5" {
                return Err(io::Error::from_raw_os_error(5));
            }
            Ok(serde_json::json!({"onlyNativeFixture":true,"normalStop":false,"closed":false}))
        });
        if mode == "valid" {
            assert_eq!(result.unwrap().unwrap()["normalStop"], false);
        } else if mode == "callback-raw5" {
            assert_eq!(result.unwrap_err().raw_os_error(), Some(5));
        } else {
            assert!(result.is_err());
        }
        {
            let kept = control.validation.lock().unwrap();
            assert!(kept.attempted);
            assert_eq!(kept.complete, mode == "valid");
            let actual = kept.original.as_ref().unwrap();
            assert_eq!(identity(&actual.original.file), id);
            assert_eq!(actual.window.deadline(), end);
            assert_eq!(actual.original.revoked, mode != "valid");
        }
        assert!(control
            .validate_returned_for_test(end, |_, _, _| panic!("must not retry"))
            .is_err());
        let after = control
            .observe(Instant::now() + Duration::from_secs(5))
            .unwrap();
        assert_eq!(after["workerThread"], before["workerThread"]);
        assert_eq!(
            after["returnValidatorThread"],
            format!("{:?}", thread::current().id())
        );
        assert_ne!(after["workerThread"], after["returnValidatorThread"]);
        assert_eq!(
            after["phase"],
            if mode == "valid" {
                "ReturnedValidated"
            } else {
                "ReturnValidationFailedRetained"
            }
        );
        assert_eq!(after["recoveryRequired"], true);
        assert_eq!(after["closed"], false);
        assert_eq!(after["runtimeAuthority"], false);
        if mode == "callback-raw5" {
            assert_eq!(after["rawError"], 5);
        }
        println!("r7-return-validation actualworker {mode}: originalfileID={} received before checks, fixedwindow retained, originalworker identity unchanged, no retry/normalStop/Closed; state={after}",id.index);
        let WorkerControl {
            worker, validation, ..
        } = control;
        worker.join().unwrap();
        drop(validation);
        drop(scene);
    }
}
