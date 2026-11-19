use super::*;
use std::{
    fs,
    os::unix::fs::{DirBuilderExt, PermissionsExt},
    time::Duration,
};

fn end() -> Instant {
    Instant::now() + Duration::from_secs(10)
}
fn fixture() -> (PathBuf, ProcessContext, Index, Head) {
    let temp = tempfile::Builder::new()
        .prefix("ocxd024-successor-review-")
        .tempdir()
        .unwrap()
        .keep();
    let temp = temp.canonicalize().unwrap();
    println!("SUCCESSOR_CANCEL_REVIEW_FIXTURE_CREATED {}", temp.display());
    let (context, index, closed) =
        crate::infrastructure::macos_client_originals::root_retirement::fixture_closed_generation_at(temp.clone());
    let (index, head) = index
        .create_successor_root(&context, &closed, end())
        .unwrap();
    (temp, context, index, head)
}
fn bytes(root: &Path) -> BTreeMap<PathBuf, Vec<u8>> {
    fn visit(root: &Path, at: &Path, result: &mut BTreeMap<PathBuf, Vec<u8>>) {
        for e in fs::read_dir(at).unwrap() {
            let path = e.unwrap().path();
            if path.is_dir() {
                visit(root, &path, result);
            } else {
                result.insert(
                    path.strip_prefix(root).unwrap().into(),
                    fs::read(path).unwrap(),
                );
            }
        }
    }
    let mut result = BTreeMap::new();
    visit(root, root, &mut result);
    result
}
fn close(temp: PathBuf, context: ProcessContext, index: Index) {
    let path = temp;
    drop(index);
    drop(context);
    remove_fixture(path);
}

fn remove_fixture(path: PathBuf) {
    fs::remove_dir_all(&path).expect("exact owned successor-review fixture cleanup");
    assert!(!path.exists());
    println!("SUCCESSOR_CANCEL_REVIEW_FIXTURE_REMOVED {}", path.display());
}

#[test]
fn successor_preparation_review_audits_history_without_mutation_or_first_root_permission() {
    let (temp, context, index, head) = fixture();
    let before = bytes(&temp);
    let review = index
        .review_successor_root_preparation(&context, &head, end())
        .unwrap();
    assert_eq!(review.fingerprint().len(), 64);
    review.recheck().unwrap();
    assert_eq!(index.head().unwrap(), Some(head.clone()));
    assert_eq!(bytes(&temp), before);
    assert!(index
        .review_first_root_preparation(&context, &head, end())
        .is_err());
    assert!(index.reserve_start().is_err());
    assert!(index.reserve_first_root_capture(&head).is_err());
    assert_eq!(bytes(&temp), before);
    let fingerprint = review.fingerprint().to_owned();
    drop(review);
    drop(index);
    use super::super::super::preview::{self, Status};
    let data = temp.join("desktop");
    let started = Instant::now();
    let preview = preview::inspect_preview(&context, &data);
    let preview_ms = started.elapsed().as_millis();
    assert!(preview_ms < 5000);
    assert_eq!(preview.status, Status::Pending);
    assert_eq!(preview.phase, Some(State::Preparing));
    assert_eq!(preview.attempt_id.as_deref(), Some(head.ticket.as_str()));
    assert_eq!(preview.preparation_kind, Some("successor_root"));
    assert_eq!(
        preview.preparation_digest.as_deref(),
        Some(fingerprint.as_str())
    );
    assert!(preview.can_cancel_preparation);
    assert!(!preview.can_prepare_plan && !preview.can_continue && !preview.can_finalize);
    let expired = preview::inspect_preview_until(&context, &data, Instant::now());
    assert_eq!(expired.status, Status::Unknown);
    assert!(expired.preparation_digest.is_none());
    assert!(!expired.can_cancel_preparation);
    assert_eq!(bytes(&temp), before);
    println!(
        "SUCCESSOR_CANCEL_REVIEW {}",
        serde_json::json!({"readOnly":true,"revision":head.revision,"phase":head.state,"previewElapsedMs":preview_ms,"fingerprintsMatch":true,"noMovementOrStartPermission":true,"releaseAcceptance":false})
    );
    drop(context);
    remove_fixture(temp);
}

#[test]
fn successor_review_refuses_wrong_head_context_deadline_and_later_capture_phase() {
    let (temp, context, index, head) = fixture();
    let before = bytes(&temp);
    let mut wrong = head.clone();
    wrong.revision += 1;
    assert!(index
        .review_successor_root_preparation(&context, &wrong, end())
        .is_err());
    let other = ProcessContext {
        runtime: context.runtime.clone(),
        working_directory: context.working_directory.clone(),
        opencodex_home: context.opencodex_home.join("foreign"),
    };
    assert!(index
        .review_successor_root_preparation(&other, &head, end())
        .is_err());
    assert!(index
        .review_successor_root_preparation(&context, &head, Instant::now())
        .is_err());
    assert_eq!(bytes(&temp), before);
    let reserved = index.reserve_successor_root_capture(&head).unwrap();
    let reserved_bytes = bytes(&temp);
    assert!(index
        .review_successor_root_preparation(&context, &reserved, end())
        .is_err());
    let pending = Head {
        revision: reserved.revision + 1,
        ticket: reserved.ticket.clone(),
        state: State::RootCancellationPrepared,
        references: None,
        restore_plan: None,
        completion_sha256: None,
        root_retirement: Some(RootRetirement {
            source_revision: reserved.revision,
            plan: RestorePlan {
                id: uuid::Uuid::new_v4().to_string(),
                sha256: "a".repeat(64),
            },
        }),
        root_generation: None,
    };
    validate(&pending).unwrap();
    let error = index
        .write_checked_projected(
            Some(&reserved),
            &pending,
            &|_| Ok(()),
            &|_| panic!("rejected source must not reach admission"),
            &|_, _, _| Ok(()),
            None,
        )
        .unwrap_err();
    assert!(error
        .to_string()
        .contains("unintegrated ordinary lifecycle"));
    let mut history = vault_history::read(index.connection().unwrap(), &reserved).unwrap();
    history.push(pending);
    assert!(root_lineage::parse(&index.binding, &history).is_err());
    assert!(root_cancel_namespace::Namespace::from_history(&history).is_err());
    assert_eq!(index.head().unwrap(), Some(reserved));
    assert_eq!(bytes(&temp), reserved_bytes);
    close(temp, context, index);
}

#[test]
fn successor_review_rejects_new_user_data_and_retains_it_unchanged() {
    let (temp, context, index, head) = fixture();
    let review = index
        .review_successor_root_preparation(&context, &head, end())
        .unwrap();
    let user_file = context.working_directory.join(".codex/new-user-data");
    fs::write(
        &user_file,
        b"explicit owned fixture; must not be touched by review",
    )
    .unwrap();
    let before = bytes(&temp);
    assert!(review.recheck().is_err());
    assert!(index
        .review_successor_root_preparation(&context, &head, end())
        .is_err());
    assert_eq!(bytes(&temp), before);
    assert_eq!(
        fs::read(user_file).unwrap(),
        b"explicit owned fixture; must not be touched by review"
    );
    close(temp, context, index);
}

#[test]
fn successor_review_rechecks_retained_vault_membership_instead_of_ignoring_old_attempts() {
    let (temp, context, index, head) = fixture();
    let review = index
        .review_successor_root_preparation(&context, &head, end())
        .unwrap();
    let extra = index
        .data_root()
        .join("manager-state/client-originals/unindexed-fixture");
    fs::write(&extra, b"unindexed fixture only").unwrap();
    let before = bytes(&temp);
    assert!(review.recheck().is_err());
    assert!(index
        .review_successor_root_preparation(&context, &head, end())
        .is_err());
    assert_eq!(bytes(&temp), before);
    close(temp, context, index);
}

#[test]
fn successor_review_rejects_changed_historical_original_bytes_and_keeps_the_failed_scene() {
    let (temp, context, index, head) = fixture();
    let review = index
        .review_successor_root_preparation(&context, &head, end())
        .unwrap();
    let files = bytes(&temp);
    let original = files
        .keys()
        .find(|p| {
            p.starts_with("desktop/manager-state/client-originals")
                && p.file_name().is_some_and(|n| n == "original.json")
        })
        .expect("actual retained Original file");
    let path = temp.join(original);
    let mode = fs::metadata(&path).unwrap().permissions().mode();
    fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).unwrap();
    fs::write(&path, b"explicit owned old Original drift fixture").unwrap();
    fs::set_permissions(&path, fs::Permissions::from_mode(mode)).unwrap();
    let before = bytes(&temp);
    assert!(review.recheck().is_err());
    assert!(index
        .review_successor_root_preparation(&context, &head, end())
        .is_err());
    assert_eq!(bytes(&temp), before);
    assert_eq!(
        fs::read(path).unwrap(),
        b"explicit owned old Original drift fixture"
    );
    close(temp, context, index);
}

#[test]
fn successor_review_cannot_reuse_old_confirmation_after_grok_changes_or_deadline_expires() {
    let (temp, context, index, head) = fixture();
    let review = index
        .review_successor_root_preparation(&context, &head, end())
        .unwrap();
    let grok = context.working_directory.join(".grok");
    fs::DirBuilder::new().mode(0o700).create(&grok).unwrap();
    fs::write(
        grok.join("config.toml"),
        b"# explicit synthetic newly added Grok settings\n",
    )
    .unwrap();
    fs::set_permissions(grok.join("config.toml"), fs::Permissions::from_mode(0o600)).unwrap();
    let before = bytes(&temp);
    assert!(review.recheck().is_err());
    assert_eq!(bytes(&temp), before);
    let deadline = Instant::now() + Duration::from_secs(3);
    let fresh = index
        .review_successor_root_preparation(&context, &head, deadline)
        .unwrap();
    std::thread::sleep(deadline.saturating_duration_since(Instant::now()));
    assert!(fresh.recheck().is_err());
    assert_eq!(bytes(&temp), before);
    close(temp, context, index);
}
