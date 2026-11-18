use super::*;
use std::{fs, time::Duration};

struct Fixture(Option<tempfile::TempDir>);
impl Fixture {
    fn new() -> Self {
        Self(Some(tempfile::tempdir().unwrap()))
    }
    fn root(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
    fn bind(&self, origin: [Uuid; 3], capture_session: Uuid, end: Instant) -> PendingAttachment {
        let parent = native::create_private_directory(&self.root().join("D")).unwrap();
        PendingAttachment::bind(parent, origin, capture_session, end).unwrap()
    }
}
impl Drop for Fixture {
    fn drop(&mut self) {
        let temp = self.0.take().unwrap();
        if thread::panicking() {
            println!(
                "r7-pending-attachment-failure-retained={}",
                temp.keep().display()
            );
        } else {
            let path = temp.path().to_owned();
            temp.close().unwrap();
            assert!(!path.exists());
            println!("r7-pending-attachment-cleanup-absent={}", path.display());
        }
    }
}

fn origin() -> [Uuid; 3] {
    [
        Uuid::parse_str("44444444-4444-4444-4444-444444444444").unwrap(),
        Uuid::new_v4(),
        Uuid::new_v4(),
    ]
}
fn chain(origin: [Uuid; 3], capture_session: Uuid, recovery: Uuid) -> [Vec<u8>; 2] {
    let mut data = super::super::tests::preparation(false);
    data["ownerInstanceId"] = serde_json::json!(origin[0]);
    RecordV2::initial_preparation_data(
        [
            capture_session.to_string(),
            origin[2].to_string(),
            recovery.to_string(),
            "e".repeat(64),
        ],
        serde_json::from_value(data).unwrap(),
        ["1000", "11000", "1000"].map(str::to_owned),
    )
    .unwrap()
    .preparation_pending_data(Some(64))
    .unwrap()
}
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}

#[test]
fn wl2_r7_pending_attachment_native_same_thread_exact_readback_and_create_only() {
    for mode in ["success", "collision"] {
        let fixture = Fixture::new();
        let origin = origin();
        let end = end();
        let capture_session = Uuid::new_v4();
        assert_ne!(capture_session, origin[1]);
        let mut attachment = fixture.bind(origin, capture_session, end);
        let data = fixture.root().join("D");
        let recovery = Uuid::new_v4();
        let action = data.join(format!("before-resume-pending-{recovery}"));
        let records = chain(origin, capture_session, recovery);
        if mode == "collision" {
            let existing = native::create_private_directory(&action).unwrap();
            fs::write(action.join("untouched"), b"existing").unwrap();
            let error = attachment
                .persist(&data, origin, recovery, &records, end)
                .unwrap_err();
            assert!(matches!(error.raw_os_error(), Some(80 | 183)), "{error:?}");
            assert_eq!(fs::read(action.join("untouched")).unwrap(), b"existing");
            assert_eq!(fs::read_dir(&action).unwrap().count(), 1);
            println!("r7-pending collision raw={:?}", error.raw_os_error());
            existing.recheck().unwrap();
        } else {
            attachment
                .persist(&data, origin, recovery, &records, end)
                .unwrap();
            assert_eq!(fs::read_dir(&action).unwrap().count(), 2);
            for (index, raw) in records.iter().enumerate() {
                let file = action.join(format!("v2-record-{:06}.json", index + 1));
                assert_eq!(&fs::read(&file).unwrap(), raw);
                assert!(fs::OpenOptions::new().write(true).open(&file).is_err());
            }
            assert!(attachment
                .persist(&data, origin, recovery, &records, end)
                .is_err());
            println!("r7-pending exact two-record native readback, no second admission");
        }
    }
}

#[test]
fn wl2_r7_pending_attachment_native_refuses_identity_action_thread_and_expiry_drift() {
    for mode in [
        "parent-path",
        "parent-object",
        "parent-stream",
        "owner-host",
        "owner-session",
        "capture-session",
        "failed-action",
        "new-action",
        "thread",
        "expired",
    ] {
        let fixture = Fixture::new();
        let origin = origin();
        let mut end = end();
        let capture_session = Uuid::new_v4();
        assert_ne!(capture_session, origin[1]);
        let mut attachment = fixture.bind(origin, capture_session, end);
        let mut data = fixture.root().join("D");
        let recovery = Uuid::new_v4();
        let mut current_origin = origin;
        let mut record_origin = origin;
        let mut record_session = capture_session;
        let mut record_recovery = recovery;
        match mode {
            "parent-path" => data = fixture.root().join("different-D"),
            "parent-object" => {
                let other =
                    native::create_private_directory(&fixture.root().join("other")).unwrap();
                attachment.parent_file = storage::open_directory_data(&other).unwrap();
            }
            "parent-stream" => fs::write(format!("{}:foreign", data.display()), b"drift").unwrap(),
            "owner-host" => record_origin[0] = Uuid::new_v4(),
            "owner-session" => current_origin[1] = Uuid::new_v4(),
            "capture-session" => record_session = origin[1],
            "failed-action" => record_origin[2] = Uuid::new_v4(),
            "new-action" => record_recovery = Uuid::new_v4(),
            "expired" => end = Instant::now(),
            "thread" => {}
            _ => unreachable!(),
        }
        let records = chain(record_origin, record_session, record_recovery);
        let error = if mode == "thread" {
            thread::scope(|scope| {
                scope
                    .spawn(|| attachment.persist(&data, current_origin, recovery, &records, end))
                    .join()
                    .unwrap()
            })
            .unwrap_err()
        } else {
            attachment
                .persist(&data, current_origin, recovery, &records, end)
                .unwrap_err()
        };
        if mode == "expired" {
            assert_eq!(error.kind(), io::ErrorKind::TimedOut);
        }
        assert_eq!(fs::read_dir(fixture.root().join("D")).unwrap().count(), 0);
        println!(
            "r7-pending {mode} refused before creating files raw={:?} kind={:?}",
            error.raw_os_error(),
            error.kind()
        );
    }
}

#[test]
fn wl2_r7_pending_attachment_native_short_write_flush_and_readback_keep_diagnostic_prefix() {
    for (mode, fault) in [("short-write", 3), ("flush", 6), ("readback", 4)] {
        let fixture = Fixture::new();
        let origin = origin();
        let end = end();
        let capture_session = Uuid::new_v4();
        assert_ne!(capture_session, origin[1]);
        let mut attachment = fixture.bind(origin, capture_session, end);
        attachment.storage.inject_pending_fixture_fault(fault);
        let data = fixture.root().join("D");
        let recovery = Uuid::new_v4();
        let records = chain(origin, capture_session, recovery);
        let error = attachment
            .persist(&data, origin, recovery, &records, end)
            .unwrap_err();
        let action = data.join(format!("before-resume-pending-{recovery}"));
        let first = fs::read(action.join("v2-record-000001.json")).unwrap();
        let expected = if mode == "short-write" {
            &records[0][..records[0].len() / 2]
        } else {
            &records[0]
        };
        assert_eq!(&first, expected);
        assert_eq!(fs::read_dir(&action).unwrap().count(), 1);
        if mode == "flush" {
            assert_eq!(error.raw_os_error(), Some(1117));
        }
        validate_chain_v2(&records).unwrap();
        assert!(attachment
            .persist(&data, origin, recovery, &records, end)
            .is_err());
        println!(
            "r7-pending injected {mode}, retained bytes={} raw={:?}",
            first.len(),
            error.raw_os_error()
        );
    }
}
