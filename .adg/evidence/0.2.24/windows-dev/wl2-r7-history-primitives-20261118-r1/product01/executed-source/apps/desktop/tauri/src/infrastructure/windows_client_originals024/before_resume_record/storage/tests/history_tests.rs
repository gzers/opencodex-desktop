use super::*;
#[test]
fn wl2_r7_history_primitives_native_v1attachment_same_files_after_write_cutoff_no_new_permission() {
    for mode in [
        "valid",
        "member",
        "file-security",
        "expired-read",
        "pending-write",
        "missing-completion",
    ] {
        let scene = Scene::new();
        let original_end = Instant::now() + Duration::from_millis(600);
        let mut storage = scene.initialized(original_end);
        let raw = encoded_data(&record());
        if mode == "pending-write" {
            storage.fault = 3;
            assert!(storage.append_data(&raw, original_end).is_err());
        } else if mode != "missing-completion" {
            storage.append_data(&raw, original_end).unwrap();
        }
        let identities = storage.files.iter().map(|r| r.id).collect::<Vec<_>>();
        while Instant::now() < original_end {
            std::thread::sleep(Duration::from_millis(1));
        }
        match mode {
            "member" => std::fs::write(
                scene.path().join("records/foreign-after-cutoff.json"),
                b"preserve",
            )
            .unwrap(),
            "file-security" => change_dacl(&storage.files[0].path),
            _ => {}
        }
        let read_end = if mode == "expired-read" {
            Instant::now()
        } else {
            Instant::now() + Duration::from_secs(5)
        };
        let result = storage.read_history_data(read_end);
        if mode == "valid" {
            assert_eq!(result.unwrap(), vec![raw.clone()]);
            assert_eq!(
                storage.read_history_data(read_end).unwrap(),
                vec![raw.clone()]
            );
            assert!(!storage.failed);
        } else {
            assert!(result.is_err(), "{mode}");
            assert!(storage.failed);
        }
        assert_eq!(storage.end, Some(original_end));
        assert_eq!(
            storage.files.iter().map(|r| r.id).collect::<Vec<_>>(),
            identities
        );
        assert!(storage.append_data(&raw, original_end).is_err());
        assert!(storage.append_data(&raw, read_end).is_err());
        assert!(!scene.path().join("records/record-000002.json").exists());
        println!("r7 historical v1attachment native {mode}: same original files/native prefix, original write cutoff expired/no newappend or rewindow; DATA only");
        drop(storage);
        drop(scene);
    }
}
