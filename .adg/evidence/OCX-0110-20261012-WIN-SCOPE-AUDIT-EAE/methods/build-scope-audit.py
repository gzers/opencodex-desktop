"""Bounded source/evidence audit of the original goal; no TASK/RCP reconstruction."""
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[1]
docs = root.parents[2]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
running = "afff490a430a81f8c233d44928aa3681a23e91e0"

def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))

def evidence(name, file="archive-verification.json"):
    return f".adg/evidence/OCX-0110-20261012-{name}/{file}"

def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))

original = read(docs / evidence("WIN-FINAL-CI-AUDIT", "requirement-audit.json"))
assert len(original["requirements"]) == 14
scope_names = [item["requirement"] for item in original["requirements"]]
live = read(root / "current-source-and-lifecycle.json")
assert live["mainSource"] == source and live["retainedApplicationSource"] == running
raw = (code / "test/runtime/windows0110/cycles/stable-panel-20261012/observations.ndjson").read_bytes()
complete = raw[:raw.rfind(b"\n") + 1]
assert complete
(root / "complete-live-observation-prefix.ndjson").write_bytes(complete)
records = [json.loads(line) for line in complete.decode("utf-8-sig").splitlines() if line.strip()]
baseline = read(docs / evidence("WIN-REAL-STABLE-PANEL-CYCLE", "schedule-baseline.json"))
initial = read(docs / evidence("WIN-REAL-STABLE-PANEL-CYCLE", "native-initialization.json"))["result"]
previous = None
for record in records:
    native = record["native"]
    result = native["result"]
    assert native["sourceCommit"] == running and native["applicationPid"] == 38924
    assert record["originalLifecycleLive"]
    assert previous is None or result["atMs"] >= previous
    previous = result["atMs"]
now = datetime.now(timezone.utc)
timing = {
    "calculatedAtUtc": now.isoformat(), "sourceCommit": running, "applicationPid": 38924,
    "completeRecords": len(records), "observationPrefixSha256": hashlib.sha256(complete).hexdigest(),
    "monotonicObservationsNondecreasing": True,
    "elapsedFromInitialEnableToLastSampleMs": records[-1]["native"]["result"]["atMs"] - initial["enabledAtMs"],
    "targets": {
        target: {
            "lastSuccessEpoch": baseline["entries"][target]["last_success"],
            "nextDueEpoch": baseline["entries"][target]["next_due"],
            "intervalSeconds": baseline["entries"][target]["next_due"] - baseline["entries"][target]["last_success"],
            "dueUtc": datetime.fromtimestamp(baseline["entries"][target]["next_due"], timezone.utc).isoformat(),
            "secondsUntilDueAtAudit": baseline["entries"][target]["next_due"] - now.timestamp(),
            "observedChangesFromBaseline": sum(
                item["schedule"]["entries"][target] != baseline["entries"][target] for item in records
            ),
        } for target in ["manager-stable", "panel-latest"]
    },
    "rawNetworkOrNativeCommandCounts": "not instrumented",
    "newMainCycleAccepted": False, "realCycleCompleted": False,
}
assert all(value["intervalSeconds"] == 86400 for value in timing["targets"].values())
assert all(value["observedChangesFromBaseline"] == 0 for value in timing["targets"].values())
write("independent-cycle-prefix-recalculation.json", timing)
rows = [
    (
        "proved_current_source",
        "Current main CI and Windows candidate both succeeded on eae8532; raw ZIP, package, CI-installed/readonly-extracted EXE and lock identity verified. Does not prove local installation/native acceptance.",
        [evidence("WIN-VERSION-PROBE-INTEGRATED", "run-38254821701-final.json"),
         evidence("WIN-VERSION-PROBE-INTEGRATED", "candidate-final.json"),
         evidence("WIN-VERSION-PROBE-INTEGRATED", "candidate-identity-verification.json")],
        source,
    ),
    (
        "incomplete_missing_original_authority",
        "IMP-20 is read and remains in_progress. Original TASK-182 object is still absent; its scope_digest and required_checks cannot be recreated from receipts or this audit.",
        ["docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md",
         ".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/current-source-and-lifecycle.json"],
        "document authority plus missing TASK",
    ),
    (
        "observed_invariant",
        "One retained desktop PID38924; independent process handles and creation times verified. Daily executable hash unchanged. All actions used isolated fixtures; no claim of exhaustive real-data inventory.",
        [".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/current-source-and-lifecycle.json",
         ".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/current-native-read.json"],
        running,
    ),
    (
        "partial_old_application_and_current_functions",
        "Actual afff desktop ordinary false/type3 and administrator true/type2 tokens were observed. eae Rust query/probe test processes also prove both tokens and owned-process behavior. The new eae desktop dual-mode/native UI matrix has not run.",
        [evidence("WIN-ADMIN-TOKEN", "administrator-single-session.json"),
         evidence("WIN-ADMIN-TOKEN", "ordinary-pre-transition-session.json"),
         evidence("WIN-VERSION-PROBE-TREE", "probe-atomic-fixed-ordinary-version-result.json"),
         evidence("WIN-VERSION-PROBE-TREE", "probe-atomic-fixed-administrator-version-result.json")],
        f"desktop {running}; Rust functions {source}",
    ),
    (
        "partial_not_complete_dpi_or_native_matrix",
        "Recorded ordinary DPI192/DPR2, Mica, caption max/restore/minimize/move/top-edge resize and close-hide cases exist. Other physical scales, full administrator foreground UI, Snap/tray reopen and normal exit remain unproved. RDP display scale is disabled; tool limitations are not exemptions.",
        [evidence("WIN-WINDOW-GEOMETRY"), evidence("WIN-CLOSE-HIDE"),
         evidence("WIN-ADMIN-TOKEN"), evidence("WIN-ENVIRONMENT-LIMITS", "settings-observation.json")],
        "historical candidate native observations, not final eae desktop",
    ),
    (
        "incomplete_safe_installer_and_root_admission",
        "Current packages share daily UpgradeCode; only CI fresh-installer smoke and local readonly extraction passed. Actual default install/data, custom-root success, permissions, 0.1.9 upgrade/uninstall retention and independent installer environment remain missing. Earlier custom-root admission was runtime_unverified; not bypassed.",
        [evidence("WIN-VERSION-PROBE-INTEGRATED", "msi-identity.json"),
         evidence("WIN-VERSION-PROBE-INTEGRATED", "ci-installer-verification.json"),
         evidence("WIN-RETENTION", "custom-root-reference-switch.json")],
        "current packages plus older blocked root-admission sample",
    ),
    (
        "incomplete_real_update_and_script_confirmation",
        "Cached shared details and truthful unknown version/disabled actions were checked on afff. No-script 2.82.0 attempts ended scripts_required. No explicit FZ50 second confirmation; runtime currently unresolved. Actual install/update, long notes scroll, trusted source/signature/error/cancel/progress/restart matrix remains open.",
        [evidence("WIN-UPDATE-DETAILS"), evidence("WIN-NPM-CWD-FINAL", "no-script-native-install-observation.json"),
         evidence("WIN-NPM-RETEST", "install-script-confirmation-plan.json"),
         ".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/current-native-read.json"],
        "older production/native observations, not current eae update acceptance",
    ),
    (
        "partial_recorded_restore_retention_pin_cases",
        "Real creation/different-value restore, manual/automatic N10 OR D30 with labelled synthetic historical dates, pin/protection and corrupt payload refusal were observed. Candidate-scoped restore recovery and afff administrator command roundtrip passed. Full safety/failure/protection/UI matrix and previously rejected junction fixture remain open.",
        [evidence("WIN-RETENTION"), evidence("WIN-RESTORE-EVENT-SCOPE"),
         evidence("WIN-ADMIN-TOKEN", "administrator-backup-roundtrip.json")],
        "historical actual native/command fixtures; dates explicitly synthetic",
    ),
    (
        "partial_tree_cases_not_full_matrix",
        "Shared tree code and eight browser geometry cases plus real production rows/keyboard/async-focus cases were verified. All checkbox/mixed/disabled controls, themes/density/quality and physical DPI/admin combinations do not have final eae native evidence.",
        [evidence("WIN-CHILD-APPDATA", "archive-verification.json"), evidence("WIN-TREE-KEYBOARD")],
        "browser fixtures and older native candidate scenes",
    ),
    (
        "partial_scope_specific_events_and_channels",
        "Real restore A/B/rotated-A isolation and continuous same-candidate resolution passed. One actual in-flight panel metadata query survived manager stable-to-beta switch. Panel-check has no persistent policy/resolves; query recovery does not prove persistent notification recovery. All domain/phase/channel/in-flight-install cases remain open.",
        [evidence("WIN-RESTORE-EVENT-SCOPE"), evidence("WIN-INFLIGHT-CHANNEL"),
         evidence("WIN-PANEL-EVENT-RECOVERY", "registry-policy-witness.json")],
        running,
    ),
    (
        "passed_recorded_old_real_timer_scenario",
        "Actual old production error timer first fired after 1,800,001.5ms, with 200 generated wakes and actual minimize/restore; original functions/files restored. Scheduler blobs match current main, but there is no new-main full native proof, raw request/command count trace or real clock-change matrix.",
        [evidence("WIN-REAL-TIMER-BACKOFF", "monitor-state.json"),
         evidence("WIN-REAL-TIMER-BACKOFF", "post-cleanup-native-verification.json"),
         ".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/current-source-and-lifecycle.json"],
        running,
    ),
    (
        "not_measured_original_failure_retained",
        "Official 0.1.9 EXE identity preserved; native API experiment proves Profile remains outside fixture under environment overrides, so baseline has not been launched. No paired cold/hot starts, 10min idle/frame/query resource samples accepted. Exact original thresholds and prior scheduler-performance failure are retained.",
        [evidence("WIN-BASELINE-PREFLIGHT", "official-baseline-safety-and-budget.json"),
         evidence("WIN-BASELINE-KNOWN-FOLDERS", "known-folder-comparison.json"),
         "docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/06-执行基线与契约.md"],
        "official baseline de6862d4 and current host boundary",
    ),
    (
        "waiting_real_time_not_accepted",
        "Original afff stable/panel actual successful baseline +86400 deadlines remain; same retained lifecycle is live and recorded prefix has no schedule transition. Nominal due October13 18:01:07/09 UTC+8. Beta6h initialization failed; last authenticated beta-tag sample was404 on October12 17:59. Sleep/resume/restart and final source acceptance are incomplete.",
        [evidence("WIN-REAL-STABLE-PANEL-CYCLE", "schedule-baseline.json"),
         evidence("WIN-REAL-BETA-CYCLE", "beta-source-api.json"),
         ".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/live-monitor-at-audit.json",
         ".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/independent-cycle-prefix-recalculation.json"],
        running,
    ),
    (
        "recorded_branch_correct_delivery_no_closure",
        "Requested source baselines are ancestors; code main and docs/governance files remain separated, shared five blobs match. Current CI/artifact/function evidence is committed. No release/installer execution or actual TASK/certificate closure occurred; core promotion/final acceptance are still not authorized by partial results.",
        [".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/current-source-and-lifecycle.json",
         evidence("WIN-VERSION-PROBE-INTEGRATED"), "AGENTS.md",
         "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md"],
        "current main plus docs authority and recorded lifecycle",
    ),
]
assert len(rows) == len(scope_names)
requirements = []
references = set()
for name, (status, reason, files, applies) in zip(scope_names, rows):
    assert all((docs / file).is_file() for file in files), (name, files)
    references.update(files)
    requirements.append({
        "requirement": name, "status": status, "reason": reason,
        "evidence": files, "sourceApplicability": applies,
        "completeOriginalRequirement": status in {"proved_current_source", "observed_invariant", "recorded_branch_correct_delivery_no_closure"},
    })
original_budget = read(docs / evidence("WIN-BASELINE-PREFLIGHT", "official-baseline-safety-and-budget.json"))["originalPerformanceContract"]
audit = {
    "collectedAtUtc": now.isoformat(), "sourceCommit": source, "runningSourceCommit": running,
    "objectiveScopeUnchanged": True, "originalRequirementCount": 14,
    "auditOnlyNotFormalTaskChecks": True, "requirements": requirements,
    "additionalCurrentSourceFunctionEvidence": [
        {"sourceCommit": source, "requirement": "Windows metadata-query owned tree and true process modes",
         "evidence": [evidence("WIN-QUERY-PROCESS-TREE"), evidence("WIN-QUERY-ADMIN-FUNCTIONS")],
         "result": "recorded function cases passed; current metadata production blob unchanged by version fix"},
        {"sourceCommit": source, "requirement": "Version-probe two-second owned tree in actual ordinary/admin test process tokens",
         "evidence": [evidence("WIN-VERSION-PROBE-TREE")],
         "result": "old0/3; fixed3/3 per mode; metadata4/4 per mode"},
    ],
    "originalPerformanceBudget": original_budget, "performanceFailureCleared": False,
    "overallResult": "not_achieved", "formalTaskReceipt": False, "formalCertificate": False,
    "coreAuthorityPromoted": False, "releaseAccepted": False,
    "genuinePrerequisites": [
        "recover original TASK-182 object from original execution machine, without recreating authority",
        "explicit human FZ50 second confirmation for exact one ordinary2.82.0 script retry",
        "safe same-host official0.1.9 Profile and installer environment; do not alter current profile or daily UpgradeCode registration",
        "eligible native sessions/environment for complete physical DPI and administrator UI matrix",
    ],
    "verifiedLiveWait": {
        "monitorPid": 18944, "monitorExecSession": 93170,
        "specificLiveApplicationPid": 38924, "deadlineUtc": "2026-10-13T10:01:07+00:00",
        "status": "verified live old-source timer cycle; no terminal acceptance yet",
    },
}
write("requirement-audit.json", audit)
support = []
for name in sorted(references):
    if name.startswith(".adg/evidence/OCX-0110-20261012-WIN-SCOPE-AUDIT-EAE/"):
        continue  # Own raw files are covered by this archive's manifest.
    path = docs / name
    if name == "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md":
        continue  # IMP is authoritative and will append this recorded audit.
    support.append({"path": name, "bytes": path.stat().st_size,
                    "sha256": hashlib.sha256(path.read_bytes()).hexdigest()})
write("supporting-evidence-hashes.json", {
    "mode": "raw bytes", "scope": "referenced source-bound artifacts at audit time; not a full acceptance certificate",
    "entries": support,
})
print(json.dumps({"originalRequirements": len(requirements), "overall": audit["overallResult"],
                  "completeRealTimeObservations": len(records)}))
