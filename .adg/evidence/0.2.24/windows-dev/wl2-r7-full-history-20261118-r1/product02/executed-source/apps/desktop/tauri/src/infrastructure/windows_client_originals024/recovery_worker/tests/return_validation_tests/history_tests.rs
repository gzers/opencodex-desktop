use super::*;
#[test]
fn wl2_r7_full_history_native_receiving_control_requires_actual_timely_validation_and_one_read_cutoff(
) {
    for mode in [
        "valid-after-expiry",
        "unvalidated",
        "binding-change",
        "raw5",
        "expired-reader",
        "oversized-reader",
    ] {
        let scene = Scene(Some(tempfile::tempdir().unwrap()));
        let root = scene.0.as_ref().unwrap().path();
        drop(create_private_directory(&root.join(".codex")).unwrap());
        drop(create_private_directory(&root.join("coordination")).unwrap());
        drop(create_private_directory(&root.join("source")).unwrap());
        let paths = pathset(root);
        for path in &paths {
            drop(PrivateRecord::create(path, b"", 16).unwrap());
        }
        let source = root.join("source/original.bin");
        drop(PrivateRecord::create(&source, b"actual historical source only", 128).unwrap());
        let file = OpenOptions::new()
            .read(true)
            .share_mode(1)
            .open(&source)
            .unwrap();
        let id = identity(&file);
        let start = Instant::now();
        let window =
            BeforeResumeRecoveryWindow::begin(start, start + Duration::from_secs(1)).unwrap();
        let old = window.deadline();
        let mut slot = Some(SyntheticOriginal {
            binding: TransferBinding {
                session: uuid::Uuid::new_v4().to_string(),
                action: uuid::Uuid::new_v4().to_string(),
                scope: hash::sha256_hex(b"actual retained original history scope"),
                end: old,
            },
            file,
            id,
            bytes: b"actual historical source only".to_vec(),
            revoked: false,
        });
        let control = WorkerControl::start(&mut slot, window, move |_, local, window| {
            local
                .preparation
                .prepare_fixture_paths(&paths, window.deadline())?;
            local.lease = Some(local.preparation.take_complete()?);
            local.release(window.deadline())
        })
        .unwrap();
        let before = complete_probe(&control);
        assert_eq!(before["phase"], "ReturnedForValidation");
        if mode != "unvalidated" {
            control
                .validate_returned_for_test(old, |actual, released, _| {
                    assert_eq!(identity(&actual.file), id);
                    released.check(old)?;
                    Ok(serde_json::json!({"fixtureOnly":true}))
                })
                .unwrap()
                .unwrap();
            let kept = control.validation.lock().unwrap();
            assert!(kept.completed_at.unwrap() < old);
        }
        if mode == "valid-after-expiry" {
            while Instant::now() < old {
                thread::sleep(Duration::from_millis(1));
            }
        }
        if mode == "binding-change" {
            control
                .validation
                .lock()
                .unwrap()
                .original
                .as_mut()
                .unwrap()
                .original
                .binding
                .scope = hash::sha256_hex(b"substituted scope DATA");
        }
        let read_end = match mode {
            "expired-reader" => Instant::now(),
            "oversized-reader" => Instant::now() + Duration::from_secs(6),
            _ => Instant::now() + Duration::from_secs(5),
        };
        let result = control.read_returned_history_for_test(
            read_end,
            |actual, window, end| {
                if Instant::now() >= end {
                    return Err(io::ErrorKind::TimedOut.into());
                }
                actual.binding(window)
            },
            |actual, released, window, end| {
                assert_eq!(window.deadline(), old);
                assert_eq!(identity(&actual.file), id);
                released.readback_history(end)?;
                if mode == "raw5" {
                    return Err(io::Error::from_raw_os_error(5));
                }
                Ok(serde_json::json!({"fixtureOnly":true,"normalStop":false,"closed":false}))
            },
        );
        if mode == "valid-after-expiry" {
            assert_eq!(result.unwrap()["closed"], false);
        } else if mode == "raw5" {
            assert_eq!(result.unwrap_err().raw_os_error(), Some(5));
        } else {
            assert!(result.is_err(), "{mode}");
        }
        let observed = control
            .observe(Instant::now() + Duration::from_secs(5))
            .unwrap();
        assert_eq!(observed["workerThread"], before["workerThread"]);
        assert_eq!(observed["recoveryRequired"], true);
        assert_eq!(observed["runtimeAuthority"], false);
        if mode == "binding-change" || mode == "raw5" {
            let kept = control.validation.lock().unwrap();
            assert!(kept.original.as_ref().unwrap().original.revoked);
            assert!(!kept.complete);
            assert_eq!(observed["phase"], "HistoryReadFailedRetained");
        }
        println!("r7 fullhistory originalcontrol {mode}: actual retainedsourceFileID={} actualtimelyvalidation required, oneStatus5 rejectsnew6/expiredread, originalwindow unchanged, DATA no fullproductOwnerfactory; state={observed}",id.index);
        let WorkerControl {
            worker,
            validation,
            returned,
            state,
        } = control;
        worker.join().unwrap();
        // Unvalidated originals still live in the actual return channel, not
        // the validation slot. Close that exact own fixture channel explicitly
        // after the released worker is joined and before TempDir closure.
        drop(returned);
        drop(validation);
        drop(state);
        drop(scene);
    }
}
