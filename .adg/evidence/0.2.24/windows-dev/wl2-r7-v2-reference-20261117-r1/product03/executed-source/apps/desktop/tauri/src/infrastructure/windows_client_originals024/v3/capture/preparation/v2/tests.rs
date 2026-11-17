use super::*;
use crate::infrastructure::windows_client_originals024::{
    coordination::NativeLease, tree::tests::Fixture,
};
use crate::infrastructure::windows_official_runtime024::{self as native, PrivateRecord};
use std::time::Duration;
struct Scene(Option<Fixture>);
impl Scene {
    fn new() -> Self {
        Self(Some(Fixture::new()))
    }
    fn path(&self) -> &Path {
        self.0.as_ref().unwrap().path()
    }
}
impl Drop for Scene {
    fn drop(&mut self) {
        if let Some(f) = self.0.take() {
            let path = f.path().to_path_buf();
            drop(f);
            if !std::thread::panicking() {
                assert!(!path.exists());
                println!("r7-v2-reference-cleanup-absent={}", path.display());
            }
        }
    }
}
#[test]
fn wl2_r7_v2_reference_native_original_slot_session_host_and_actual_directory_membership_stay_pending(
) {
    let scene = Scene::new();
    let session = uuid::Uuid::new_v4();
    let host =
        crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process();
    assert_eq!(
        host,
        crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process()
    );
    let mut prefix = PartialCaptureAnchor::new(
        crate::infrastructure::hash::sha256_hex(
            b"component scope from native fixture; not productOwner",
        ),
        session,
        host,
    );
    let end = Instant::now() + Duration::from_secs(10);
    let (initial, details) = prefix.v2_reference_data(end).unwrap();
    assert!(initial.pending());
    assert_eq!(details["sessionId"], session.to_string());
    let path = scene.path().join("actual-custody");
    prefix.intend(path.clone()).unwrap();
    let pin = prefix.create_directory(&path, end).unwrap();
    let id = pin.identity();
    prefix.custody[0] = Some(pin);
    let (first, details) = prefix.v2_reference_data(end).unwrap();
    let (second, _) = prefix.v2_reference_data(end).unwrap();
    assert_eq!(
        serde_json::to_value(&first).unwrap(),
        serde_json::to_value(&second).unwrap()
    );
    assert!(first.pending());
    assert_eq!(prefix.session_id, session);
    assert_eq!(prefix.host_instance_id, host);
    assert_eq!(prefix.custody[0].as_ref().unwrap().identity(), id);
    let v = serde_json::to_value(&first).unwrap();
    assert_eq!(v["kind"], "preparation");
    assert_eq!(v["source"]["state"], "unobserved");
    assert!(v.get("manifestDigest").is_none() && v.get("storagePlanDigest").is_none());
    assert!(v["components"][1]["objects"].as_array().unwrap().len() >= 2);
    assert!(v["components"][1]["objects"][0]["identityDigest"].is_string());
    assert_eq!(
        v["components"][1]["objects"][0]["securityDigest"],
        serde_json::Value::Null
    );
    assert!(!serde_json::to_string(&first)
        .unwrap()
        .contains(&path.to_string_lossy().to_string()));
    assert_eq!(details["runtimeAuthority"], false);
    assert!(std::fs::rename(&path, scene.path().join("replacement")).is_err());
    assert!(prefix.v2_reference_data(Instant::now()).is_err());
    println!("r7 V2 actual original directory creator/pin membership retains session/nativeIDs; noSource/SD/content staysPending, no fake manifest, stable objectIDs, no native recovery/Closed grant");
    drop(prefix);
    drop(scene);
}
#[test]
fn wl2_r7_v2_reference_native_invalid_coordinator_retains_all_known_and_pending_node_membership() {
    let scene = Scene::new();
    let profile = native::create_private_directory(&scene.path().join("Profile")).unwrap();
    let root = profile.path().join(".codex");
    drop(native::create_private_directory(&root).unwrap());
    for name in crate::modules::runtime::windows_lifecycle024_contract::ANCHORED_CODEX_STABLE_FILES
    {
        drop(PrivateRecord::create(&root.join(name), b"", 32).unwrap());
    }
    let bad = root.join(".opencodex-native-main.owner.sqlite");
    std::fs::write(&bad, b"unknown SQLite bytes originalfixture").unwrap();
    let lockdir = native::create_private_directory(&scene.path().join("locks")).unwrap();
    let paths = std::array::from_fn(|i| {
        if i == 0 {
            root.join(".opencodex-native-main.claim.sqlite")
        } else {
            lockdir.path().join(format!("{i}.sqlite"))
        }
    });
    for p in paths.iter().skip(1) {
        drop(PrivateRecord::create(p, b"", 16).unwrap());
    }
    let custody = native::create_private_directory(&profile.path().join("custody")).unwrap();
    let end = Instant::now() + Duration::from_secs(10);
    let mut lease = NativeLease::acquire_synthetic(&paths, end).unwrap();
    let mut claim = lease.anchored_claim(&root, end).unwrap();
    let mut prefix = PartialCaptureAnchor::new(
        crate::infrastructure::hash::sha256_hex(b"component scope only"),
        uuid::Uuid::new_v4(),
        crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process(),
    );
    assert!(AnchoredTree::capture_retained(
        &root,
        custody.path(),
        &mut claim,
        &root,
        end,
        &mut prefix.trees[0]
    )
    .is_err());
    let (reference, details) = prefix.v2_reference_data(end).unwrap();
    let v = serde_json::to_value(&reference).unwrap();
    assert!(reference.pending());
    assert_eq!(v["components"][2]["state"], "retained");
    let objects = v["components"][2]["objects"].as_array().unwrap();
    assert!(objects.len() >= 4);
    assert!(objects.iter().any(|o| o["identityDigest"].is_null()));
    assert!(objects
        .iter()
        .all(|o| o["securityDigest"].is_null() && o["contentDigest"].is_null()));
    assert_eq!(details["sessionId"], prefix.session_id.to_string());
    assert!(std::fs::remove_file(&bad).is_err());
    assert_eq!(
        std::fs::read(&bad).unwrap(),
        b"unknown SQLite bytes originalfixture"
    );
    println!("r7 V2 actual failed original node collection observes known/pendinghandle/parent/custody membership, no inferred Source/Absent/manifest/restore terminal");
    drop(prefix);
    drop(claim);
    lease.release(end).unwrap();
    drop(custody);
    drop(lockdir);
    drop(profile);
    drop(scene);
}
#[test]
fn wl2_r7_v2_reference_native_expired_creation_is_intent_not_created_or_absent() {
    let scene = Scene::new();
    let path = scene.path().join("never-created");
    let mut prefix = PartialCaptureAnchor::new(
        crate::infrastructure::hash::sha256_hex(b"synthetic originalscope only"),
        uuid::Uuid::new_v4(),
        crate::infrastructure::windows_official_lifecycle024::original_host_instance_for_process(),
    );
    prefix.intend(path.clone()).unwrap();
    assert!(prefix.create_directory(&path, Instant::now()).is_err());
    assert!(!path.exists());
    let (reference, _) = prefix
        .v2_reference_data(Instant::now() + Duration::from_secs(10))
        .unwrap();
    assert!(reference.pending());
    let v = serde_json::to_value(reference).unwrap();
    assert_eq!(v["lastStage"], "intentPublished");
    assert_eq!(v["components"][1]["state"], "retained");
    let object = &v["components"][1]["objects"][0];
    assert_eq!(object["originalStage"], "intentPublished");
    assert!(object["identityDigest"].is_null());
    assert_ne!(v["components"][1]["state"], "verifiedAbsent");
    println!("r7 V2 actual expired directory preparation retained intent only; no created/Absent fact inferred from failing call, no original nativepermission");
    drop(prefix);
    drop(scene);
}
