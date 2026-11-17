use super::super::tests::Fixture;
use super::*;
use crate::modules::runtime::client_artifact_policy::{
    self as policy, Action, Artifact, Observations, Refusal,
};
use std::time::Duration;
fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
struct Trees {
    fixture: Fixture,
    profile: PinnedDirectory,
    custody: PinnedDirectory,
    trees: [RootTree; 2],
}
impl Trees {
    fn new() -> Self {
        let fixture = Fixture::new();
        let profile = create_private_directory(&fixture.path().join("Profile")).unwrap();
        let codex = profile.path().join(".codex");
        std::fs::create_dir(&codex).unwrap();
        std::fs::write(
            codex.join("config.toml"),
            b"synthetic credential/config unchanged",
        )
        .unwrap();
        let custody = create_private_directory(&profile.path().join("owned-custody")).unwrap();
        let mut trees = [
            RootTree::capture(&codex, end()).unwrap(),
            RootTree::capture(&profile.path().join(".grok"), end()).unwrap(),
        ];
        for (i, tree) in trees.iter_mut().enumerate() {
            tree.move_to_custody(
                custody.path(),
                &profile.path().join(format!("intent-{i}.json")),
                end(),
            )
            .unwrap();
            tree.create_working_copy(end()).unwrap();
        }
        Self {
            fixture,
            profile,
            custody,
            trees,
        }
    }
    fn codex(&self) -> PathBuf {
        self.profile.path().join(".codex")
    }
    fn archive(&mut self, phase: SnapshotPhase) -> WorkingArchive {
        let mut archive = WorkingArchive::create(
            &self.custody,
            phase,
            "00000000-0000-4000-8000-000000000009",
            &"a".repeat(64),
            &Policy::fixed().digest().unwrap(),
            end(),
        )
        .unwrap();
        archive.capture(&mut self.trees, end()).unwrap();
        archive
    }
}
fn context<'a>(
    before: &'a ArtifactFrame,
    ready: &'a ArtifactFrame,
    pre: &'a ArtifactFrame,
    post: &'a ArtifactFrame,
    routes: Option<&'a [String]>,
) -> Observations<'a> {
    Observations {
        before: before.stage(),
        ready: ready.stage(),
        pre_stop: pre.stage(),
        stopped: post.stage(),
        declared_routes: routes,
        retained_pids: &[10, 11],
        audit_home: "synthetic-admitted-H",
        audit_not_before: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:00Z")
            .unwrap()
            .into(),
        audit_not_after: chrono::DateTime::parse_from_rfc3339("2026-11-11T10:00:02Z")
            .unwrap()
            .into(),
    }
}
fn audit(writer: &str, target: &str, intent: &str) -> Vec<u8> {
    let mut raw=serde_json::to_vec(&serde_json::json!({"at":"2026-11-11T10:00:01Z","pid":11,"ppid":10,"command":writer,"opencodexHome":"synthetic-admitted-H","target":target,"outcome":"written","intent":intent,"writer":writer,"configSource":"file","routedBefore":0,"routedAfter":1})).unwrap();
    raw.push(b'\n');
    raw
}
#[test]
fn wl2_r5_artifact_sealed_ready_prestop_and_native_current_feed_shared_policy_only() {
    let mut t = Trees::new();
    let before = t.trees[0].original_artifact_frame(end()).unwrap();
    let ready_catalog =
        serde_json::json!({"models":[{"slug":"P/m","display_name":"Synthetic routed"}]});
    std::fs::write(
        t.codex().join(NAMES[0]),
        serde_json::to_vec(&ready_catalog).unwrap(),
    )
    .unwrap();
    std::fs::write(t.codex().join(NAMES[1]),serde_json::to_vec(&serde_json::json!({"fetched_at":"2000-01-01T00:00:00Z","client_version":"0.0.0","models":ready_catalog["models"]})).unwrap()).unwrap();
    let mut events = audit("startup-cache", "cache", "cache");
    std::fs::write(t.codex().join(NAMES[2]), &events).unwrap();
    let mut ready_archive = t.archive(SnapshotPhase::Ready);
    let ready = ready_archive.artifact_frame(end()).unwrap();
    let mut pre_archive = t.archive(SnapshotPhase::PreStop);
    let pre = pre_archive.artifact_frame(end()).unwrap();
    assert!(
        ready.stage().catalog == pre.stage().catalog,
        "archive-copy identity must not replace the captured live generation"
    );
    std::fs::write(t.codex().join(NAMES[0]), br#"{"models":[]}"#).unwrap();
    events.extend(audit("codex-restore", "catalog", "restore"));
    std::fs::write(t.codex().join(NAMES[2]), &events).unwrap();
    let current = t.trees[0].current_artifact_frame(end()).unwrap();
    t.trees[0].verify_non_artifact_working(end()).unwrap();
    t.trees[1].verify_working(end()).unwrap();
    let routes = vec!["P/m".into()];
    assert_eq!(
        policy::assess(&context(&before, &ready, &pre, &current, Some(&routes))),
        Ok(policy::Assessment([
            (Artifact::Catalog, Action::RestoreOriginal),
            (Artifact::ModelsCache, Action::RestoreOriginal),
            (Artifact::CatalogAudit, Action::RestoreOriginal)
        ]))
    );
    assert_eq!(
        policy::assess(&context(&before, &ready, &pre, &current, None)),
        Err(Refusal::Context)
    );
    let wrong = vec!["other/route".into()];
    assert!(policy::assess(&context(&before, &ready, &pre, &current, Some(&wrong))).is_err());
    assert!(current
        .require_root(&t.profile.path().join("other"))
        .is_err());
    assert!(t.trees[0].verify_working(end()).is_err());
    assert!(t.trees[0].restore_original(end()).is_err());
    assert!(t.custody.path().join(".codex/config.toml").exists());
    ready_archive.recheck(end()).unwrap();
    pre_archive.recheck(end()).unwrap();
    println!("r5 native original/sealed Ready+PreStop/current frames satisfy shared controlled content policy; None/wrong routes refuse; original working-veto still prevents restore, no native exit/lease or Closed manufactured");
    t.fixture.retain();
}
#[test]
fn wl2_r5_artifact_unknown_nonartifact_and_original_config_drift_never_disappear() {
    for variant in 0..3 {
        let mut t = Trees::new();
        match variant {
            0 => std::fs::write(t.codex().join("unknown.new"), b"external").unwrap(),
            1 => std::fs::write(
                t.codex().join("config.toml"),
                b"different credential/config",
            )
            .unwrap(),
            _ => std::fs::create_dir(t.codex().join(NAMES[0])).unwrap(),
        }
        assert!(t.trees[0].verify_non_artifact_working(end()).is_err());
        assert!(t.trees[0].poisoned);
        assert!(t.custody.path().join(".codex/config.toml").exists());
        println!("r5 undeclared node/config drift/allowed-name directory variant={variant} poisons tree; original custody and external bytes retained");
        t.fixture.retain();
    }
}
#[test]
fn wl2_r5_artifact_archive_foreign_node_refuses_data_observation() {
    let mut t = Trees::new();
    let mut ready = t.archive(SnapshotPhase::Ready);
    ready.artifact_frame(end()).unwrap();
    let foreign = t.custody.path().join("ready/.codex/foreign.new");
    std::fs::write(&foreign, b"not an archive entry").unwrap();
    assert!(ready.artifact_frame(end()).is_err());
    assert!(foreign.exists());
    println!("r5 sealed archive unknown entry cannot provide content stage; archive native objects/conflict retained");
    t.fixture.retain();
}
#[test]
fn wl2_r5_artifact_prestop_change_is_not_hidden_by_content_equality() {
    let mut t = Trees::new();
    let before = t.trees[0].original_artifact_frame(end()).unwrap();
    std::fs::write(t.codex().join(NAMES[0]), br#"{"models":[]}"#).unwrap();
    let mut ready = t.archive(SnapshotPhase::Ready);
    let a = ready.artifact_frame(end()).unwrap();
    std::fs::write(
        t.codex().join(NAMES[0]),
        br#"{"models":[],"source":"changed"}"#,
    )
    .unwrap();
    let mut pre = t.archive(SnapshotPhase::PreStop);
    let b = pre.artifact_frame(end()).unwrap();
    let c = t.trees[0].current_artifact_frame(end()).unwrap();
    let routes = Vec::new();
    assert_eq!(
        policy::assess(&context(&before, &a, &b, &c, Some(&routes))),
        Err(Refusal::ChangedSinceReady)
    );
    println!("r5 actual Ready vs PreStop content/generation mismatch refuses shared assessment; no fake equality or skipped historical snapshot");
    t.fixture.retain();
}
