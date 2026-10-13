"""Synthetic tool-only refusal cases; never inject these records into the product."""
import copy
import json
import sys
from pathlib import Path

sys.dont_write_bytecode = True
from transition_contract import CANDIDATE, OWNER_TERMINAL, SOURCE, evaluate_transition

root = Path(__file__).resolve().parent
base = {
    "initial": {"result": {"originalChannel": "stable", "originalAutomaticCheck": False}},
    "cycle_audit": {"sourceCommit": SOURCE, "status": "recorded_next_successes_require_terminal_and_scope_review"},
    "monitor": {
        "sourceCommit": SOURCE, "applicationPid": 37276, "monitorPid": 3920,
        "status": OWNER_TERMINAL, "finishedAtUtc": "synthetic-tool-only-terminal",
        "originalUpdatePreferencesRestored": True, "failedObservations": 0,
    },
    "cleanup": {"sourceCommit": SOURCE, "applicationPid": 37276, "result": {
        "channel": "stable", "automaticCheck": False, "originalUpdatePreferencesRestored": True,
    }},
    "session": {
        "sourceCommit": SOURCE, "applicationPid": 37276, "supervisorPid": 24300,
        "status": "stop-requested", "finishedAtUtc": "synthetic-tool-only-terminal",
    },
    "processes": [{"pid": pid, "actualLiveHandle": False, "queryError": 87} for pid in [37276, 24300, 3920]],
    "desktops": {"count": 0}, "preferences": {"channel": "stable", "automaticCheck": False},
}
results = []
valid = evaluate_transition(**base)
assert valid["status"] == "read_only_prerequisites_met_requires_operator_review"
assert valid["candidateSource"] == CANDIDATE and not valid["automaticProcessTransitionAllowed"]
assert not valid["foregroundAuthorityGranted"] and not valid["realFullCycleAccepted"]
results.append({"name": "synthetic-complete-prerequisites-remain-review-only", "passed": True, "nativeEvidence": False})
def refuse(name, change, raises=False):
    case = copy.deepcopy(base)
    change(case)
    try:
        value = evaluate_transition(**case)
    except ValueError:
        assert raises
    else:
        assert not raises and value["blockingConditions"]
        assert value["status"] == "pending_original_owners_or_zero_instance"
    results.append({"name": name, "passed": True, "nativeEvidence": False})
refuse("running-monitor", lambda case: case["monitor"].update(status="observing-real-deadlines", finishedAtUtc=None))
refuse("interrupted-monitor", lambda case: case["monitor"].update(status="interrupted-original-lifecycle-ended"))
refuse("incomplete-successes", lambda case: case["cycle_audit"].update(status="pending_original_deadlines"))
refuse("cleanup-absent", lambda case: case.update(cleanup=None))
refuse("cleanup-boolean-is-not-enough", lambda case: case["cleanup"]["result"].update(automaticCheck=True))
refuse("disk-not-restored", lambda case: case["preferences"].update(automaticCheck=True))
refuse("supervisor-still-running", lambda case: case["session"].update(status="running"))
refuse("original-handle-still-live", lambda case: case["processes"][0].update(actualLiveHandle=True, exactCreationMatched=True))
refuse("query-denial-is-not-absent-process", lambda case: case["processes"][0].update(queryError=5))
refuse("another-desktop-running", lambda case: case["desktops"].update(count=1))
refuse("wrong-native-source", lambda case: case["session"].update(sourceCommit=CANDIDATE), raises=True)
refuse("wrong-cleanup-owner", lambda case: case["cleanup"].update(applicationPid=999), raises=True)
refuse("monitor-gap-needs-review", lambda case: case["monitor"].update(failedObservations=1))
record = {
    "scope": "in-memory transition-contract tests only; no native state, time, response or process injection",
    "result": "passed", "cases": results, "nativeCycleAccepted": False,
    "foregroundAuthorityGranted": False, "processTransitionExecuted": False,
}
(root / "transition-contract-self-test.json").write_text(json.dumps(record, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"toolOnlyTests": len(results), "result": "passed", "nativeCycleAccepted": False}))
