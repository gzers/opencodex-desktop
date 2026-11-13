use super::*;
use crate::modules::{
    model_config::config_file::v2::{self, ReadRequestV2},
    notifications::{
        persistence::{load_notifications, notifications_path},
        NotificationStore,
    },
};
use std::sync::{Arc, Mutex};
const ID: &str = "00000000-0000-4000-8000-000000000001";
struct Fixture {
    _root: tempfile::TempDir,
    d: std::path::PathBuf,
    h: std::path::PathBuf,
    bytes: Vec<u8>,
    store: crate::state::SharedNotificationStore,
}
impl Fixture {
    fn new() -> Self {
        let root = tempfile::tempdir().unwrap();
        let base = root.path().canonicalize().unwrap();
        let d = base.join("desktop");
        let h = base.join("home");
        std::fs::create_dir_all(d.join("manager-state")).unwrap();
        std::fs::create_dir(&h).unwrap();
        let bytes = serde_json::to_vec(&serde_json::json!({
            "providers": {
                "private-channel-one": {"name":"Private channel one","baseUrl":"https://private.invalid","adapter":"openai-chat","apiKey":"do-not-publish-secret-one","models":["gpt-6-sol","vendor/private-custom"]},
                "private-channel-two": {"name":"Private channel two","baseUrl":"https://private.invalid","adapter":"openai-chat","apiKey":"do-not-publish-secret-two","models":["gpt-6.1-sol"]}
            },"port":41234
        })).unwrap();
        std::fs::write(h.join("config.json"), &bytes).unwrap();
        Self {
            _root: root,
            d,
            h,
            bytes,
            store: Arc::new(Mutex::new(NotificationStore::new())),
        }
    }
    fn read(&self) -> ModelReadResult<ReadResponseV2> {
        let s = v2::snapshot(&self.d, &self.h).unwrap();
        v2::read_existing(
            &self.d,
            &self.h,
            ReadRequestV2 {
                expected_source: s.source_digest,
                expected_library: s.library_digest,
                expected_draft: s.draft.map(|d| d.base),
            },
        )
    }
    fn publish(&self, d: &Delivery<'_>) -> crate::errors::AppResult<registry::DeliveryOutcome> {
        crate::commands::notifications::NotificationPublisher {
            store: &self.store,
            data_root: &self.d,
        }
        .publish_event(d)
    }
}

#[test]
fn admitted_read_persists_numeric_summary_once_without_config_or_credentials_and_reopens() {
    let f = Fixture::new();
    let result = f.read();
    let d = delivery(ID, &result).unwrap();
    assert_eq!(d.event, "model-channels-read-succeeded");
    let Detail::ModelReadCounts(c) = d.detail.unwrap() else {
        panic!("counts required")
    };
    assert_eq!(c.total, 3);
    assert_eq!(
        c.matched
            + c.manual_preserved
            + c.unmatched
            + c.missing_default
            + c.missing_dependency
            + c.conflict,
        3
    );
    f.publish(&d).unwrap();
    f.publish(&d).unwrap();
    let reopened = load_notifications(&notifications_path(&f.d)).unwrap();
    assert_eq!(reopened.all().len(), 1);
    let item = &reopened.all()[0];
    assert!(item.resolved);
    assert!(!item.read);
    assert_eq!(
        item.action_ref,
        Some(crate::modules::notifications::NotificationAction::Models)
    );
    assert!(item.body.contains("已读取 3 个模型"));
    assert!(item.body.contains(&format!("自动匹配 {} 项", c.matched)));
    assert!(item.body.contains("保存配置后才生效"));
    let history = std::fs::read_to_string(notifications_path(&f.d)).unwrap();
    for secret in [
        "private-channel",
        "Private channel",
        "private.invalid",
        "do-not-publish-secret",
        "vendor/private",
    ] {
        assert!(!history.contains(secret));
    }
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), f.bytes);
}

#[test]
fn stale_read_has_fixed_failure_notice_retains_prior_summary_and_never_claims_draft_saved() {
    let f = Fixture::new();
    let result = f.read();
    f.publish(&delivery(ID, &result).unwrap()).unwrap();
    let s = v2::snapshot(&f.d, &f.h).unwrap();
    let failed = v2::read_existing(
        &f.d,
        &f.h,
        ReadRequestV2 {
            expected_source: "stale".into(),
            expected_library: s.library_digest,
            expected_draft: s.draft.map(|d| d.base),
        },
    );
    assert!(failed.is_err());
    let event = delivery("00000000-0000-4000-8000-000000000002", &failed).unwrap();
    assert_eq!(event.event, "model-channels-read-failed");
    assert!(event.detail.is_none());
    f.publish(&event).unwrap();
    let state = load_notifications(&notifications_path(&f.d)).unwrap();
    assert_eq!(state.all().len(), 2);
    assert!(state.all().iter().any(|n| n.title == "现有渠道读取完成"));
    assert!(state
        .all()
        .iter()
        .any(|n| n.title == "现有渠道未读取" && !n.resolved));
    assert_eq!(std::fs::read(f.h.join("config.json")).unwrap(), f.bytes);
}

#[test]
fn incomplete_or_incoherent_read_receipts_and_cross_event_count_details_are_rejected() {
    let f = Fixture::new();
    let mut result = f.read();
    result.as_mut().unwrap().results[0].models.pop();
    assert!(delivery(ID, &result).is_none());
    let mut valid = delivery(ID, &f.read()).unwrap();
    valid.detail = None;
    assert!(f.publish(&valid).is_err());
    valid.detail = Some(Detail::ModelReadCounts(ModelReadCounts {
        total: 1,
        ..Default::default()
    }));
    assert!(f.publish(&valid).is_err());
    valid.detail = Some(Detail::ModelReadCounts(ModelReadCounts::default()));
    valid.event = "model-policy-save-succeeded";
    valid.identity.action = Action::Save;
    assert!(f.publish(&valid).is_err());
    assert!(f.store.lock().unwrap().all().is_empty());
    assert!(delivery("not-an-operation-id", &f.read()).is_none());
}
