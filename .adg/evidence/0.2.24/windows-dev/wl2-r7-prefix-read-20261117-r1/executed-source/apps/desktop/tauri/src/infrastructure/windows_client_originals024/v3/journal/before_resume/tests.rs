use super::*;
use crate::infrastructure::windows_client_originals024::{tree::tests::Fixture, v3::tests::plan};
use std::time::Duration;
struct Scene(Option<Fixture>);
impl Scene {
    fn new(window: Duration) -> (Self, Journal) {
        let (fixture, plan, profile, _) = plan();
        let data = native::create_private_directory(&fixture.path().join("D")).unwrap();
        let directory = native::create_private_directory(&data.path().join("session")).unwrap();
        let pins = [
            PinnedDirectory::open_private(&profile).unwrap(),
            data,
            native::create_private_directory(&fixture.path().join("H")).unwrap(),
            native::create_private_directory(&fixture.path().join("I")).unwrap(),
        ];
        let peer = RetainedPeer::current().unwrap();
        let binding = Binding::from_native(
            &peer,
            &pins,
            hash::sha256_hex(b"fixture DATA not official Owner"),
            hash::sha256_hex(b"current DATA"),
        )
        .unwrap();
        let end = Instant::now() + window;
        let journal = Journal::create(
            directory,
            pins,
            peer,
            binding,
            plan,
            &uuid::Uuid::new_v4().to_string(),
            &"c".repeat(64),
            &uuid::Uuid::new_v4().to_string(),
            end,
        )
        .unwrap();
        (Self(Some(fixture)), journal)
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(fixture) = self.0.take() {
            let path = fixture.path().to_path_buf();
            drop(fixture);
            if !std::thread::panicking() {
                assert!(!path.exists());
                println!("r7-canonical-prefix-cleanup-absent={}", path.display());
            }
        }
    }
}
fn event(journal: &Journal) -> Event {
    let node = journal
        .manifest
        .storage_plan
        .roots
        .iter()
        .flat_map(|r| &r.nodes)
        .find(|n| n.kind != Kind::CustodiedFile)
        .unwrap();
    super::super::super::tests::event(
        &journal.manifest.storage_plan,
        &journal.progress,
        node,
        Operation::SnapshotVerified,
        &node.file_id,
    )
}
#[test]
fn wl2_r7_prefix_read_canonical_native_expired_original_complete_chain_reads_without_mutation_renewal(
) {
    let (scene, mut journal) = Scene::new(Duration::from_millis(500));
    let old = journal.deadline;
    let entry = event(&journal);
    journal.append(entry, old).unwrap();
    let manifest = journal.manifest_record.raw.clone();
    let current = journal.current_raw.clone();
    let manifest_id = journal.manifest_record.file.identity();
    let current_id = journal.current_id;
    while Instant::now() < old {
        std::thread::sleep(Duration::from_millis(1));
    }
    assert_eq!(
        journal.check(old).unwrap_err().kind(),
        io::ErrorKind::TimedOut
    );
    assert!(journal.poisoned);
    let fresh = Instant::now() + Duration::from_secs(10);
    let data = journal.read_failed_start_records(fresh).unwrap();
    assert_eq!(data["runtimeAuthority"], false);
    assert_eq!(data["sequence"], 1);
    assert_eq!(data["poisoned"], true);
    assert_eq!(journal.deadline, old);
    assert!(journal.check(fresh).is_err());
    assert!(journal.poisoned);
    assert_eq!(journal.manifest_record.raw, manifest);
    assert_eq!(journal.current_raw, current);
    assert_eq!(journal.manifest_record.file.identity(), manifest_id);
    assert_eq!(identity(&journal.current).unwrap(), current_id);
    assert_eq!(
        std::fs::read(journal.directory.path().join("current.json")).unwrap(),
        current
    );
    assert!(
        journal.source_action.is_none(),
        "component DATA test cannot create the actual production Owner/native read guard"
    );
    println!("r7 actual immutable native records/all8/current/membership and full DATA replay read after original expiry; old deadline/poison unchanged; no Source/Owner/native capability fabricated");
    drop(journal);
    drop(scene);
}
#[test]
fn wl2_r7_prefix_read_canonical_native_event_without_current_and_partial_current_are_not_completed()
{
    for fault in [1, 2] {
        let (scene, mut journal) = Scene::new(Duration::from_secs(10));
        let end = journal.deadline;
        journal.fault = fault;
        let entry = event(&journal);
        assert!(journal.append(entry, end).is_err());
        assert!(journal.poisoned);
        assert_eq!(journal.events.len(), 1);
        assert_eq!(journal.progress.sequence, 0);
        let current = std::fs::read(journal.directory.path().join("current.json")).unwrap();
        assert!(journal.read_failed_start_records(end).is_err());
        assert!(journal.poisoned);
        assert_eq!(
            std::fs::read(journal.directory.path().join("current.json")).unwrap(),
            current
        );
        assert!(journal.directory.path().join("event-000001.json").exists());
        println!("r7 actual failed append stage{fault}: immutable extra event/partial current retained, never inferred complete by recovery read");
        drop(journal);
        drop(scene);
    }
}
fn change_private_dacl(path: &Path) {
    use std::os::windows::ffi::OsStrExt;
    #[link(name = "advapi32")]
    unsafe extern "system" {
        fn ConvertStringSecurityDescriptorToSecurityDescriptorW(
            text: *const u16,
            revision: u32,
            descriptor: *mut *mut std::ffi::c_void,
            size: *mut u32,
        ) -> i32;
        fn SetFileSecurityW(
            path: *const u16,
            information: u32,
            descriptor: *const std::ffi::c_void,
        ) -> i32;
    }
    let text = "D:P(A;;FA;;;OW)(A;;FA;;;SY)"
        .encode_utf16()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let name = path
        .as_os_str()
        .encode_wide()
        .chain(Some(0))
        .collect::<Vec<_>>();
    let mut sd = std::ptr::null_mut();
    assert_ne!(
        unsafe {
            ConvertStringSecurityDescriptorToSecurityDescriptorW(
                text.as_ptr(),
                1,
                &mut sd,
                std::ptr::null_mut(),
            )
        },
        0
    );
    let result = unsafe { SetFileSecurityW(name.as_ptr(), 4, sd) };
    unsafe {
        windows_sys::Win32::Foundation::LocalFree(sd);
    }
    assert_ne!(result, 0, "only new own journal fixture DACL changed");
}
#[test]
fn wl2_r7_prefix_read_canonical_native_membership_security_partial_bytes_and_cutoff_refuse() {
    for change in ["member", "security", "partial-current", "expired"] {
        let (scene, mut journal) = Scene::new(Duration::from_secs(10));
        let end = journal.deadline;
        assert!(journal.check(Instant::now()).is_err());
        assert!(journal.poisoned);
        match change {
            "member" => {
                std::fs::write(journal.directory.path().join("unknown.json"), b"{}").unwrap()
            }
            "security" => change_private_dacl(&journal.manifest_record.path),
            "partial-current" => {
                journal.current.seek(SeekFrom::Start(0)).unwrap();
                journal.current.write_all(b"{").unwrap();
                journal.current.set_len(1).unwrap();
                journal.current.sync_all().unwrap();
            }
            _ => {}
        }
        let read_end = if change == "expired" {
            Instant::now()
        } else {
            end
        };
        assert!(
            journal.read_failed_start_records(read_end).is_err(),
            "{change}"
        );
        assert!(journal.poisoned);
        assert_eq!(journal.deadline, end);
        println!("r7 actual canonical prefix rejects {change}; retains old native objects, deadline and poison");
        drop(journal);
        drop(scene);
    }
}
