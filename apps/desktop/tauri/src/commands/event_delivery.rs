//! Owned workers publish terminal facts before returning; observing IPC is not the owner.
use crate::modules::notifications::registry::{
    self, Channel, Delivery, EventIdentity, Evidence, Fact, Job, Trigger,
};
use sha2::{Digest, Sha256};
use std::path::Path;
use tauri::{Emitter, Manager};

/// Transient payloads are sent only after pure catalog validation. In particular,
/// do not call prepare/publish for progress ticks or ordinary UI broadcasts.
pub fn emit_signal<R: tauri::Runtime, S: serde::Serialize + Clone>(
    app: &tauri::AppHandle<R>,
    event: &str,
    job: Job,
    trigger: Trigger,
    channel: Channel,
    payload: S,
) -> crate::errors::AppResult<()> {
    registry::validate_signal(event, job, trigger, channel)
        .map_err(|_| crate::errors::AppError::NotConfigured)?;
    app.emit(event, payload)?;
    Ok(())
}

#[cfg(windows)]
pub fn emit_signal_to<R: tauri::Runtime, S: serde::Serialize + Clone>(
    app: &tauri::AppHandle<R>,
    target: &str,
    event: &str,
    job: Job,
    trigger: Trigger,
    channel: Channel,
    payload: S,
) -> crate::errors::AppResult<()> {
    registry::validate_signal(event, job, trigger, channel)
        .map_err(|_| crate::errors::AppError::NotConfigured)?;
    app.emit_to(target, event, payload)?;
    Ok(())
}

/// Only registered query triggers can cross the IPC boundary.
#[derive(Debug, Clone, Copy, Default, serde::Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum QueryTrigger {
    #[default]
    User,
    Deadline,
    Foreground,
    Online,
}
impl From<QueryTrigger> for Trigger {
    fn from(value: QueryTrigger) -> Self {
        match value {
            QueryTrigger::User => Self::User,
            QueryTrigger::Deadline => Self::Deadline,
            QueryTrigger::Foreground => Self::Foreground,
            QueryTrigger::Online => Self::Online,
        }
    }
}

pub fn prepare(
    root: &Path,
    event: &str,
    channel: Channel,
    candidate: &[u8],
) -> Option<EventIdentity> {
    let result = registry::lookup(event).and_then(|definition| {
        registry::candidate_identity(
            root,
            definition.object,
            definition.action,
            definition.phase,
            channel,
            Sha256::digest(candidate).into(),
        )
    });
    match result {
        Ok(identity) => Some(identity),
        Err(_) => {
            diagnostic(root);
            None
        }
    }
}

/// A failed delivery never converts a committed mutation into a failed mutation.
pub fn publish(
    app: &tauri::AppHandle,
    root: &Path,
    event: &str,
    identity: Option<EventIdentity>,
    trigger: Trigger,
) {
    if publish_checked(app, root, event, identity, trigger).is_err() {
        diagnostic(root);
    }
}

/// An installation receipt is consumed only after persistence really succeeds.
pub fn publish_checked(
    app: &tauri::AppHandle,
    root: &Path,
    event: &str,
    identity: Option<EventIdentity>,
    trigger: Trigger,
) -> crate::errors::AppResult<()> {
    let identity = identity.ok_or(crate::errors::AppError::NotConfigured)?;
    let definition = registry::lookup(event)?;
    let evidence = match definition.fact {
        Fact::Success => Evidence::Success {
            candidate: identity.candidate.clone(),
            verified: true,
        },
        Fact::Failure => Evidence::Failure,
        Fact::Risk => Evidence::Risk,
        Fact::Signal => Evidence::Signal,
    };
    let store = app.state::<crate::state::SharedNotificationStore>();
    let delivery = Delivery {
        event,
        job: definition.job,
        trigger,
        identity,
        evidence,
        occurred_at: chrono::Utc::now(),
    };
    let outcome = crate::commands::notifications::NotificationPublisher {
        store: store.inner(),
        data_root: root,
    }
    .publish_event(&delivery)?;
    if outcome.changed {
        let _ = emit_signal(
            app,
            "notifications-changed",
            Job::NotificationMutation,
            Trigger::Commit,
            Channel::Local,
            (),
        );
    }
    Ok(())
}

fn diagnostic(root: &Path) {
    let _ = crate::infrastructure::runtime_log::RuntimeLog::new(root)
        .append_event("registered event delivery failed");
}

/// Domain callbacks run before releasing backup ownership. Candidate bytes are
/// hashed locally; neither payload contents nor paths enter notification history.
pub struct BackupEvents<'a> {
    pub app: &'a tauri::AppHandle,
    pub root: &'a Path,
    pub trigger: Trigger,
}
impl crate::modules::backup::manager::MaintenanceObserver for BackupEvents<'_> {
    fn backup_result(
        &mut self,
        action: crate::modules::backup::BackupAction,
        payload: &[u8],
        succeeded: bool,
    ) {
        // Manual, upgrade and restore protection are different attempts, even
        // when their captured preferences happen to contain identical bytes.
        let candidate = [action.as_str().as_bytes(), b"\0", payload].concat();
        let identity = prepare(
            self.root,
            "preferences-backup-failed",
            Channel::Local,
            &candidate,
        );
        publish(
            self.app,
            self.root,
            if succeeded {
                "preferences-backup-succeeded"
            } else {
                "preferences-backup-failed"
            },
            identity,
            self.trigger,
        );
    }
    fn cleanup_result(
        &mut self,
        preview: Option<&crate::types::upgrade::BackupCleanupPreviewDto>,
        succeeded: bool,
    ) {
        // An unreadable inventory cannot be resolved by a later, unrelated one.
        let candidate = preview
            .map(|p| p.token.as_bytes())
            .unwrap_or(b"cleanup-inventory-unavailable");
        let identity = prepare(
            self.root,
            "preferences-backup-cleanup-failed",
            Channel::Local,
            candidate,
        );
        publish(
            self.app,
            self.root,
            if succeeded {
                "preferences-backup-cleanup-succeeded"
            } else {
                "preferences-backup-cleanup-failed"
            },
            identity,
            self.trigger,
        );
    }
}
