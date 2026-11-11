//! A registered, numeric-only projection of the completed guarded channel read.
use super::*;
use crate::modules::{
    model_config::{config_file::v2::ReadResponseV2, schema_v2::Result as ModelReadResult},
    notifications::registry::{Delivery, Detail, Evidence, ModelReadCounts},
};

fn counts(result: &ReadResponseV2) -> Option<ModelReadCounts> {
    if result.snapshot.dto_version != 2 || result.results.is_empty() {
        return None;
    }
    let mut summary = ModelReadCounts::default();
    for row in &result.results {
        if row.dto_version != 2
            || OpaqueId::parse(&row.read_id).is_err()
            || row.models.len() as u64 != u64::from(row.counts.total)
        {
            return None;
        }
        let c = &row.counts;
        summary.total = summary.total.checked_add(c.total.into())?;
        summary.matched = summary.matched.checked_add(c.matched.into())?;
        summary.changed = summary.changed.checked_add(c.changed.into())?;
        summary.manual_preserved = summary
            .manual_preserved
            .checked_add(c.manual_preserved.into())?;
        summary.unmatched = summary.unmatched.checked_add(c.unmatched.into())?;
        summary.missing_default = summary
            .missing_default
            .checked_add(c.missing_default.into())?;
        summary.missing_dependency = summary
            .missing_dependency
            .checked_add(c.missing_dependency.into())?;
        summary.conflict = summary.conflict.checked_add(c.conflict.into())?;
        summary.protocol_pending = summary
            .protocol_pending
            .checked_add(c.protocol_pending.into())?;
    }
    Some(summary)
}

fn delivery<'a>(id: &str, result: &ModelReadResult<ReadResponseV2>) -> Option<Delivery<'a>> {
    let object_id = OpaqueId::parse(id).ok()?;
    let identity = EventIdentity {
        object: ObjectKind::Configuration,
        object_id: object_id.clone(),
        action: Action::Read,
        phase: Phase::Execution,
        channel: Channel::Local,
        candidate: object_id.clone(),
    };
    let (event, evidence, detail) = match result {
        Ok(response) => (
            "model-channels-read-succeeded",
            Evidence::Success {
                candidate: object_id,
                verified: true,
            },
            Some(Detail::ModelReadCounts(counts(response)?)),
        ),
        Err(_) => ("model-channels-read-failed", Evidence::Failure, None),
    };
    Some(Delivery {
        event,
        job: Job::ModelConfig,
        trigger: Trigger::User,
        identity,
        evidence,
        occurred_at: chrono::Utc::now(),
        detail,
    })
}

/// Called inside file_work's admitted worker, after the actual draft readback;
/// closing its IPC observer cannot abandon the terminal notification. Delivery
/// failure is logged separately and never changes a committed draft result.
pub(crate) fn channel_read_terminal(
    app: &tauri::AppHandle,
    root: &Path,
    result: &ModelReadResult<ReadResponseV2>,
) {
    use tauri::Manager;
    let Some(delivery) = delivery(&uuid::Uuid::new_v4().to_string(), result) else {
        diagnostic(root);
        return;
    };
    let store = app.state::<crate::state::SharedNotificationStore>();
    let published = crate::commands::notifications::NotificationPublisher {
        store: store.inner(),
        data_root: root,
    }
    .publish_event(&delivery);
    match published {
        Ok(outcome) if outcome.changed => {
            if crate::commands::event_delivery::emit_signal(
                app,
                "notifications-changed",
                Job::NotificationMutation,
                Trigger::Commit,
                Channel::Local,
                (),
            )
            .is_err()
            {
                diagnostic(root);
            }
        }
        Ok(_) => {}
        Err(_) => diagnostic(root),
    }
}

#[cfg(test)]
#[path = "feedback_channel_read_tests.rs"]
mod tests;
