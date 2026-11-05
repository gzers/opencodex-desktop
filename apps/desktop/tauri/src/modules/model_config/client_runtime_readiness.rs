//! A cold official settings snapshot defers CLI version validation. Wait only
//! for that explicit pending shape; the caller still validates path/owner and
//! current config before any client edit or restart.
use super::super::management::Management;
use super::super::schema::{invalid, ModelResult};
use serde_json::Value;
use std::{future::Future, time::Duration};

pub(super) async fn read_settings(management: &Management) -> ModelResult<Value> {
    await_version(
        || management.public_settings(),
        Duration::from_secs(5),
        Duration::from_millis(100),
    )
    .await
}

fn pending() -> super::super::schema::ModelError {
    invalid(
        "rename_client_runtime_pending",
        "官方客户端运行信息仍在校验中；未更新当前选择，请稍后重试",
    )
}

async fn await_version<F, Fut>(
    mut read: F,
    budget: Duration,
    interval: Duration,
) -> ModelResult<Value>
where
    F: FnMut() -> Fut,
    Fut: Future<Output = ModelResult<Value>>,
{
    let deadline = tokio::time::Instant::now() + budget;
    loop {
        let settings = tokio::time::timeout_at(deadline, read())
            .await
            .map_err(|_| pending())??;
        let deferred = settings.get("codexRuntime").is_some_and(|runtime| {
            runtime.get("version").is_some_and(Value::is_null)
                && runtime["path"]
                    .as_str()
                    .is_some_and(|path| !path.is_empty())
                && matches!(
                    runtime["source"].as_str(),
                    Some("environment" | "configured" | "shim" | "installed" | "path" | "fallback")
                )
        });
        if !deferred {
            return Ok(settings);
        }
        let next = tokio::time::Instant::now() + interval;
        if next >= deadline {
            return Err(pending());
        }
        tokio::time::sleep_until(next).await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn settings(version: Value) -> Value {
        json!({"codexRuntime":{"path":"C:\\Bound\\codex.exe","source":"environment","version":version}})
    }

    #[tokio::test]
    async fn cold_snapshot_settles_before_identity_validation() {
        let calls = AtomicUsize::new(0);
        let value = await_version(
            || {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                async move {
                    Ok(settings(if call == 0 {
                        Value::Null
                    } else {
                        json!("0.166.0-alpha.6")
                    }))
                }
            },
            Duration::from_secs(1),
            Duration::from_millis(1),
        )
        .await
        .unwrap();
        assert_eq!(value["codexRuntime"]["version"], "0.166.0-alpha.6");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn ready_or_malformed_snapshots_are_not_retried_or_rewritten() {
        for value in [
            settings(json!("0.166.0-alpha.6")),
            json!({}),
            settings(json!(42)),
            settings(json!("")),
            json!({"codexRuntime":{"path":"C:\\Bound\\codex.exe","source":"environment"}}),
            json!({"codexRuntime":{"path":"C:\\Bound\\codex.exe","source":"unsupported","version":null}}),
        ] {
            let calls = AtomicUsize::new(0);
            let readback = await_version(
                || {
                    calls.fetch_add(1, Ordering::SeqCst);
                    std::future::ready(Ok(value.clone()))
                },
                Duration::from_secs(1),
                Duration::from_millis(1),
            )
            .await
            .unwrap();
            assert_eq!(readback, value);
            assert_eq!(calls.load(Ordering::SeqCst), 1);
        }
    }

    #[tokio::test]
    async fn binding_failure_propagates_without_a_second_read() {
        let calls = AtomicUsize::new(0);
        let result = await_version(
            || {
                calls.fetch_add(1, Ordering::SeqCst);
                std::future::ready(Err(invalid("stale_runtime", "changed")))
            },
            Duration::from_secs(1),
            Duration::from_millis(1),
        )
        .await;
        assert_eq!(result.unwrap_err().code, "stale_runtime");
        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }

    #[tokio::test]
    async fn a_binding_change_during_the_pending_window_is_never_retried() {
        let calls = AtomicUsize::new(0);
        let result = await_version(
            || {
                let call = calls.fetch_add(1, Ordering::SeqCst);
                std::future::ready(if call == 0 {
                    Ok(settings(Value::Null))
                } else {
                    Err(invalid("stale_runtime", "changed"))
                })
            },
            Duration::from_secs(1),
            Duration::from_millis(1),
        )
        .await;
        assert_eq!(result.unwrap_err().code, "stale_runtime");
        assert_eq!(calls.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn still_pending_or_a_hung_read_is_bounded_and_remains_unconfirmed() {
        for hang in [false, true] {
            let result = tokio::time::timeout(
                Duration::from_millis(300),
                await_version(
                    || async move {
                        if hang {
                            std::future::pending::<()>().await;
                        }
                        Ok(settings(Value::Null))
                    },
                    Duration::from_millis(20),
                    Duration::from_millis(2),
                ),
            )
            .await
            .expect("readiness must enforce its own short budget");
            assert_eq!(result.unwrap_err().code, "rename_client_runtime_pending");
        }
    }
}
