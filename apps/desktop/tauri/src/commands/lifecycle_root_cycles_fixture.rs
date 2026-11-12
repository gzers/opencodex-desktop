//! Opt-in official299 extension using real product actions and status source.
use super::*;
use crate::infrastructure::{
    atomic_write::atomic_write,
    hash::sha256_hex,
    macos_client_originals::{
        session_index::{Head, Index, State},
        tree::TreeSnapshot,
        PinnedDirectory,
    },
    process_runner::ControlledProcessRunner,
    runtime_log::RuntimeLog,
};
use crate::modules::process::LifecycleResult;
use crate::modules::status::StatusSource;
use crate::types::runtime_status::StatusSnapshotDto;
use base64::Engine;
use std::{path::Path, time::Instant};
/// Test-host unwind cleanup only. It cannot grant any passing lifecycle result
/// and never retries a normal Stop that this round already attempted.
pub(super) struct StopOnUnwind<'a> {
    context: &'a ProcessContext,
    runner: &'a Mutex<ControlledProcessRunner>,
    log: &'a RuntimeLog,
    evidence: &'a Path,
    attempted: std::cell::Cell<bool>,
}
impl<'a> StopOnUnwind<'a> {
    pub(super) fn new(
        context: &'a ProcessContext,
        runner: &'a Mutex<ControlledProcessRunner>,
        log: &'a RuntimeLog,
        evidence: &'a Path,
    ) -> Self {
        Self {
            context,
            runner,
            log,
            evidence,
            attempted: std::cell::Cell::new(false),
        }
    }
    pub(super) fn stop_attempted(&self) {
        self.attempted.set(true);
    }
}
impl Drop for StopOnUnwind<'_> {
    fn drop(&mut self) {
        if std::env::var_os("OCXD024_ROOT_CYCLES").as_deref() != Some(std::ffi::OsStr::new("2"))
            || !std::thread::panicking()
            || self.attempted.replace(true)
        {
            return;
        }
        let cleanup = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
            process_action_with_runner(
                ProcessActionRequest {
                    action: LifecycleAction::Stop,
                    confirm: true,
                },
                self.runner,
                self.context,
                self.log,
                None,
            )
        }));
        let outcome = match cleanup {
            Ok(Ok(value)) => {
                serde_json::json!({"accepted":false,"scope":"unwind-only normal Stop attempt, not acceptance","result":value})
            }
            _ => {
                serde_json::json!({"accepted":false,"scope":"unwind-only normal Stop refused or panicked; retain scene"})
            }
        };
        let path = self.evidence.join(format!(
            "unwind-stop-{}.json",
            self.log
                .data_root()
                .join("manager-state/client-root-generations")
                .exists()
        ));
        if let Ok(bytes) = serde_json::to_vec_pretty(&outcome) {
            let _ = atomic_write(&path, &bytes, 0o600);
        }
    }
}
pub(super) struct Request<'a> {
    pub context: &'a ProcessContext,
    pub runner: &'a Mutex<ControlledProcessRunner>,
    pub log: &'a RuntimeLog,
    pub evidence: &'a Path,
    pub first_index: Index,
    pub first_head: Head,
    pub first_report: &'a serde_json::Value,
}
fn save(evidence: &Path, report: &serde_json::Value) {
    atomic_write(
        &evidence.join("root-cycles.json"),
        &serde_json::to_vec_pretty(report).unwrap(),
        0o600,
    )
    .unwrap();
}
fn vault(data: &Path, head: &Head) -> std::path::PathBuf {
    data.join("manager-state/client-originals")
        .join(&head.ticket)
}
fn snapshot(path: &Path) -> std::io::Result<TreeSnapshot> {
    TreeSnapshot::read_mode(&PinnedDirectory::open(path)?, false)
}
fn normal_chain(events: &[Vec<u8>]) -> bool {
    let expected = [
        State::Preparing,
        State::CaptureReserved,
        State::Captured,
        State::Ready,
        State::StopPrepared,
        State::Stopped,
        State::RestorePrepared,
        State::RootRetirementPrepared,
        State::RootAbsent,
    ];
    let Ok(heads) = events
        .iter()
        .map(|b| serde_json::from_slice::<Head>(b))
        .collect::<Result<Vec<_>, _>>()
    else {
        return false;
    };
    heads.len() == expected.len()
        && heads
            .iter()
            .zip(expected)
            .all(|(h, state)| h.state == state && h.ticket == heads[0].ticket)
        && heads[3]
            .references
            .as_ref()
            .is_some_and(|r| r.phases.len() == 1 && r.live_sha256.is_some())
        && heads[4]
            .references
            .as_ref()
            .is_some_and(|r| r.phases.len() == 2)
        && heads[5..]
            .iter()
            .all(|h| h.references.as_ref().is_some_and(|r| r.phases.len() == 3))
}
fn documents(index: &Index) -> (Vec<u8>, Vec<Vec<u8>>) {
    let c = index.fixture_existing_connection().unwrap();
    let binding = c
        .query_row("SELECT document FROM binding WHERE singleton=1", [], |r| {
            r.get(0)
        })
        .unwrap();
    let mut statement = c
        .prepare("SELECT document FROM events ORDER BY revision")
        .unwrap();
    let events = statement
        .query_map([], |r| r.get(0))
        .unwrap()
        .collect::<Result<Vec<Vec<u8>>, _>>()
        .unwrap();
    (binding, events)
}
struct Observed {
    snapshot: AppResult<StatusSnapshotDto>,
    official: Option<serde_json::Value>,
}
struct RecordingSource {
    inner: crate::infrastructure::status_source::OfficialStatusSource,
    value: std::sync::Arc<Mutex<Option<serde_json::Value>>>,
}
impl StatusSource for RecordingSource {
    fn manager_paths(&self) -> Option<(std::path::PathBuf, std::path::PathBuf)> {
        self.inner.manager_paths()
    }
    fn client_recovery(&self) -> crate::modules::status::ClientRecoveryState {
        self.inner.client_recovery()
    }
    fn resolved(&self) -> bool {
        self.inner.resolved()
    }
    fn source_unknown(&self) -> bool {
        self.inner.source_unknown()
    }
    fn fetch(&self) -> Result<serde_json::Value, crate::modules::status::CollectError> {
        let value = self.inner.fetch()?;
        *self.value.lock().unwrap() = Some(value.clone());
        Ok(value)
    }
    fn fetch_light(&self) -> Result<serde_json::Value, crate::modules::status::CollectError> {
        let value = self.inner.fetch_light()?;
        *self.value.lock().unwrap() = Some(value.clone());
        Ok(value)
    }
}
fn official_status(context: &ProcessContext, data: &Path) -> Observed {
    let inner = crate::infrastructure::status_source::OfficialStatusSource::new(
        context.runtime.clone(),
        &context.working_directory,
        crate::modules::process::EnvironmentPolicy {
            home: Some(context.working_directory.clone().into_os_string()),
            opencodex_home: context.opencodex_home.clone(),
            ..Default::default()
        },
    )
    .with_client_recovery_root(data.to_path_buf());
    let value = std::sync::Arc::new(Mutex::new(None));
    let source = RecordingSource {
        inner,
        value: value.clone(),
    };
    let snapshot = tauri::async_runtime::block_on(async {
        tokio::task::spawn_blocking(move || {
            let mut collector = crate::modules::status::StatusCollector::new(source);
            status_snapshot_from_collector(&mut collector)
        })
        .await
        .unwrap()
    });
    let official = value.lock().unwrap().take();
    Observed { snapshot, official }
}
fn status_json(value: &Observed) -> serde_json::Value {
    let snapshot = match &value.snapshot {
        Ok(v) => serde_json::to_value(v).unwrap(),
        Err(e) => serde_json::json!({"error":e.to_string()}),
    };
    let official=value.official.as_ref().map(|v|serde_json::json!({
        "status":v["status"],"ready":v["ready"],"pid":v["pid"],"port":v["port"],
        "statusObservation":v["statusObservation"],"officialReadiness":v["officialReadiness"],"opencodexHome":v["opencodexHome"]
    }));
    serde_json::json!({"snapshot":snapshot,"official":official})
}
fn stopped_gate(value: &Observed, context: &ProcessContext, data: &Path) -> bool {
    matches!(&value.snapshot,Ok(v) if v.matrix.runtime==crate::types::status::RuntimeState::Stopped
        && v.pid.is_none() && v.port.is_none() && v.can_start && !v.can_stop && !v.can_restart
        && v.facts.client_recovery==crate::modules::status::ClientRecoveryState::NotRequired
        && v.facts.data_root==data && v.facts.opencodex_home.as_ref()==Some(&context.opencodex_home))
}
fn recovery_gate(context: &ProcessContext, data: &Path, expected: &Head) -> bool {
    use crate::infrastructure::macos_client_originals::session_index::{generation, preview};
    let p = preview::inspect_preview(context, data);
    generation::inspect_successor_eligibility(context, data).unwrap_or(false)
        && p.status == preview::Status::Restored
        && p.phase == Some(State::RootAbsent)
        && p.revision == Some(expected.revision)
        && p.attempt_id.as_ref() == Some(&expected.ticket)
        && p.phase_count == expected.references.as_ref().map_or(0, |r| r.phases.len())
        && p.original_evidence == preview::Evidence::Verified
        && p.plan_evidence == preview::Evidence::Verified
        && p.completion_evidence == preview::Evidence::Verified
        && !p.can_continue
        && !p.can_rollback
        && !p.can_finalize
        && !p.can_rearm
        && !p.can_prepare_plan
        && !p.can_cancel_preparation
        && !p.can_finish_unchanged
}
fn stopped_observation(
    context: &ProcessContext,
    data: &Path,
    expected: &Head,
) -> (Observed, u128, bool, bool) {
    let before = snapshot(&data.join("manager-state")).unwrap();
    let config = std::fs::read(context.opencodex_home.join("config.json")).unwrap();
    let t = Instant::now();
    let observed = official_status(context, data);
    let elapsed = t.elapsed().as_millis();
    let preview_ok = recovery_gate(context, data, expected);
    let readonly = snapshot(&data.join("manager-state")).is_ok_and(|now| now == before)
        && std::fs::read(context.opencodex_home.join("config.json")).is_ok_and(|now| now == config);
    (observed, elapsed, readonly, preview_ok)
}
fn record(context: &ProcessContext) -> Option<serde_json::Value> {
    std::fs::read(context.opencodex_home.join("runtime-port.json"))
        .ok()
        .and_then(|b| serde_json::from_slice(&b).ok())
}
pub(super) fn run(request: Request<'_>) -> serde_json::Value {
    let Request {
        context,
        runner,
        log,
        evidence,
        first_index,
        first_head,
        first_report,
    } = request;
    let cleanup = StopOnUnwind::new(context, runner, log, evidence);
    let data = log.data_root();
    let client = context.working_directory.join(".codex");
    let first_vault = vault(data, &first_head);
    let old_snapshot = snapshot(&first_vault).unwrap();
    let old_documents = documents(&first_index);
    assert_eq!(old_documents.1.len() as u64, first_head.revision);
    assert!(normal_chain(&old_documents.1));
    assert_eq!(
        first_index.closed_root_history(&first_head).unwrap().len(),
        1
    );
    let first_original = std::fs::read(first_vault.join("original.json")).unwrap();
    assert_eq!(
        sha256_hex(&first_original),
        first_head.references.as_ref().unwrap().original_sha256
    );
    let first_identity =
        serde_json::from_slice::<serde_json::Value>(&first_original).unwrap()["clientRoot"].clone();
    atomic_write(
        &evidence.join("first-closed-vault-snapshot.json"),
        &serde_json::to_vec_pretty(&old_snapshot).unwrap(),
        0o600,
    )
    .unwrap();
    atomic_write(&evidence.join("first-closed-index-documents.json"),&serde_json::to_vec_pretty(&serde_json::json!({
        "bindingBase64":base64::engine::general_purpose::STANDARD.encode(&old_documents.0),
        "eventsBase64":old_documents.1.iter().map(|v|base64::engine::general_purpose::STANDARD.encode(v)).collect::<Vec<_>>()
    })).unwrap(),0o600).unwrap();
    drop(first_index);
    let mut report = serde_json::json!({"accepted":false,"passed":false,"scope":"two normal official service root lifecycles, not packaged application acceptance",
        "firstClosed":first_head,"firstRootIdentity":first_identity,"firstCycle":{"startElapsedMs":first_report["startElapsedMs"],"statusElapsedMs":first_report["statusElapsedMs"],"stopElapsedMs":first_report["stopElapsedMs"],"record":first_report["record"]},
        "startBudgetMs":30000,"statusBudgetMs":5000,"stopBudgetMs":10000,"businessRequests":false});
    save(evidence, &report);
    let (first_status, first_status_ms, first_readonly, first_preview) =
        stopped_observation(context, data, &first_head);
    let first_gate = stopped_gate(&first_status, context, data)
        && first_status_ms < 5000
        && !client.exists()
        && first_readonly
        && first_preview;
    report["firstReadonlyStateUnchanged"] = first_readonly.into();
    report["firstPreviewGate"] = first_preview.into();
    report["firstStoppedStatus"] = status_json(&first_status);
    report["firstStoppedStatusElapsedMs"] = serde_json::json!(first_status_ms);
    report["firstStoppedGate"] = first_gate.into();
    save(evidence, &report);
    if !first_gate {
        return report;
    }
    let t = Instant::now();
    let start = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Start,
            confirm: true,
        },
        runner,
        context,
        log,
        None,
    );
    let start_ms = t.elapsed().as_millis();
    report["secondStartElapsedMs"] = serde_json::json!(start_ms);
    report["secondStart"] = match &start {
        Ok(v) => serde_json::to_value(v).unwrap(),
        Err(e) => serde_json::json!({"error":e.to_string()}),
    };
    save(evidence, &report);
    let runtime_record = record(context);
    let second_record = runtime_record
        .as_ref()
        .map(|r| serde_json::json!({"pid":r["pid"],"port":r["port"]}))
        .unwrap_or(serde_json::Value::Null);
    report["secondRecord"] = second_record.clone();
    let started = matches!(&start,Ok(v) if v.result==Some(LifecycleResult::Started)&&v.can_stop&&v.can_restart&&!v.can_start)
        && start_ms < 30000;
    let mut old_during_start = false;
    let ready = Index::open_existing(context, data, &client).and_then(|i| {
        let current = documents(&i);
        old_during_start = current.0 == old_documents.0
            && current.1.starts_with(&old_documents.1)
            && snapshot(&first_vault).is_ok_and(|s| s == old_snapshot);
        i.head()
    });
    report["oldEvidenceUnchangedDuringSecondStart"] = old_during_start.into();
    report["secondReadyIndex"] = match &ready {
        Ok(v) => serde_json::to_value(v).unwrap(),
        Err(e) => serde_json::json!({"error":e.to_string()}),
    };
    save(evidence, &report);
    let t = Instant::now();
    let live_status = official_status(context, data);
    let live_ms = t.elapsed().as_millis();
    let live_gate = matches!(&live_status.snapshot,Ok(v) if v.matrix.runtime==crate::types::status::RuntimeState::Running
        && v.facts.client_recovery==crate::modules::status::ClientRecoveryState::Active
        && v.facts.data_root==data && v.facts.opencodex_home.as_ref()==Some(&context.opencodex_home)
        && v.pid.as_deref().and_then(|v|v.parse::<u64>().ok())==second_record["pid"].as_u64()
        && v.port.map(u64::from)==second_record["port"].as_u64()
        && v.pid.is_some() && v.port.is_some() && v.can_stop && v.can_restart && !v.can_start)
        && live_ms < 5000
        && live_status.official.as_ref().is_some_and(|v| {
            v["status"] == "running"
                && v["ready"] == true
                && v["officialReadiness"] == "ready"
                && v["statusObservation"] == "attested-same-connection"
                && v["opencodexHome"].as_str() == context.opencodex_home.to_str()
                && v["pid"]
                    .as_str()
                    .and_then(|s| s.parse::<u64>().ok())
                    .is_some_and(|pid| Some(pid) == second_record["pid"].as_u64())
                && v["port"]
                    .as_u64()
                    .is_some_and(|port| port > 0 && Some(port) == second_record["port"].as_u64())
        });
    report["secondLiveStatus"] = status_json(&live_status);
    report["secondLiveStatusElapsedMs"] = serde_json::json!(live_ms);
    report["secondLiveGate"] = live_gate.into();
    save(evidence, &report);
    // Ordinary product Stop is attempted even after failed Start confirmation.
    // No alternate signal or force kill obtains a passing outcome.
    let t = Instant::now();
    cleanup.stop_attempted();
    let stop = process_action_with_runner(
        ProcessActionRequest {
            action: LifecycleAction::Stop,
            confirm: true,
        },
        runner,
        context,
        log,
        None,
    );
    let stop_ms = t.elapsed().as_millis();
    report["secondStopElapsedMs"] = serde_json::json!(stop_ms);
    report["secondStop"] = match &stop {
        Ok(v) => serde_json::to_value(v).unwrap(),
        Err(e) => serde_json::json!({"error":e.to_string()}),
    };
    save(evidence, &report);
    let stop_gate = matches!(&stop,Ok(v) if v.result==Some(LifecycleResult::Stopped)&&v.lifecycle_state==ProcessLifecycleState::Stopped&&v.failure.is_none()&&v.can_start&&!v.can_stop&&!v.can_restart&&!v.client_recovery_required)
        && stop_ms < 10000;
    let expected_final = Index::open_existing(context, data, &client)
        .and_then(|i| i.head())
        .ok()
        .flatten();
    let (stopped_status, stopped_ms, second_readonly, second_preview) =
        if let Some(head) = &expected_final {
            stopped_observation(context, data, head)
        } else {
            let t = Instant::now();
            let value = official_status(context, data);
            (value, t.elapsed().as_millis(), false, false)
        };
    let stopped = stopped_gate(&stopped_status, context, data)
        && stopped_ms < 5000
        && !client.exists()
        && second_readonly
        && second_preview;
    report["secondReadonlyStateUnchanged"] = second_readonly.into();
    report["secondPreviewGate"] = second_preview.into();
    report["secondStoppedStatus"] = status_json(&stopped_status);
    report["secondStoppedStatusElapsedMs"] = serde_json::json!(stopped_ms);
    report["secondStoppedGate"] = stopped.into();
    save(evidence, &report);
    let mut history_gate = false;
    let mut distinct = false;
    if let Ok(index) = Index::open_existing(context, data, &client) {
        if let Ok(Some(head)) = index.head() {
            report["secondClosed"] = serde_json::to_value(&head).unwrap();
            let current_documents = documents(&index);
            let unchanged = current_documents.0 == old_documents.0
                && current_documents.1.starts_with(&old_documents.1)
                && snapshot(&first_vault).is_ok_and(|s| s == old_snapshot);
            report["oldBindingEventsAndVaultUnchanged"] = unchanged.into();
            let full_history = index.closed_root_history(&head);
            report["closedGenerationCount"] = full_history
                .as_ref()
                .map(|h| serde_json::json!(h.len()))
                .unwrap_or(serde_json::Value::Null);
            if head.state == State::RootAbsent {
                let raw = std::fs::read(vault(data, &head).join("original.json")).unwrap();
                assert_eq!(
                    sha256_hex(&raw),
                    head.references.as_ref().unwrap().original_sha256
                );
                let identity = serde_json::from_slice::<serde_json::Value>(&raw).unwrap()
                    ["clientRoot"]
                    .clone();
                report["secondRootIdentity"] = identity.clone();
                distinct = head.ticket != first_head.ticket
                    && identity != first_identity
                    && head.root_retirement.as_ref().unwrap().plan.id
                        != first_head.root_retirement.as_ref().unwrap().plan.id;
            }
            history_gate = unchanged
                && old_during_start
                && current_documents.1.len() >= old_documents.1.len()
                && normal_chain(&current_documents.1[old_documents.1.len()..])
                && ready
                    .as_ref()
                    .ok()
                    .and_then(|v| v.as_ref())
                    .is_some_and(|ready| {
                        current_documents.1.iter().any(|raw| {
                            serde_json::from_slice::<Head>(raw).is_ok_and(|h| h == *ready)
                        })
                    })
                && full_history.is_ok_and(|h| h.len() == 2)
                && head.state == State::RootAbsent
                && head.completion_sha256.is_some();
            let raw_events: Vec<serde_json::Value> = current_documents
                .1
                .iter()
                .map(|v| serde_json::from_slice(v).unwrap())
                .collect();
            atomic_write(
                &evidence.join("two-cycle-index-events.json"),
                &serde_json::to_vec_pretty(&raw_events).unwrap(),
                0o600,
            )
            .unwrap();
        }
    }
    let pid_changed = first_report["record"]["pid"].as_u64().is_some()
        && second_record["pid"].as_u64().is_some()
        && first_report["record"]["pid"] != second_record["pid"];
    let ready_gate = matches!(&ready,Ok(Some(h)) if h.state==State::Ready&&h.ticket!=first_head.ticket&&h.references.as_ref().is_some_and(|r|r.phases.len()==1&&r.live_sha256.is_some()));
    let records_absent = !context.opencodex_home.join("runtime-port.json").exists()
        && !context.opencodex_home.join("ocx.pid").exists();
    report["distinctRootAttemptPlanAndPid"] = (distinct && pid_changed).into();
    report["secondReadyGate"] = ready_gate.into();
    report["secondStopGate"] = stop_gate.into();
    report["recordsAbsent"] = records_absent.into();
    report["passed"] = (first_gate
        && started
        && ready_gate
        && live_gate
        && stop_gate
        && stopped
        && history_gate
        && distinct
        && pid_changed
        && records_absent)
        .into();
    save(evidence, &report);
    report
}
