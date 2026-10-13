"""Preserve completed read-only observation prefix, never overwrite live monitor logs."""
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parents[1]
runtime = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110")
cycle = runtime / "cycles/eae-stable-panel-20261012"
destination = root / "real-cycle-observations-prefix.ndjson"
assert not destination.exists(), "Preserve earlier frozen observation"
raw = (cycle / "observations.ndjson").read_bytes()
last_lf = raw.rfind(b"\n")
assert last_lf >= 0
prefix = raw[:last_lf + 1]
values = [json.loads(line) for line in prefix.splitlines() if line.strip()]
assert len(values) >= 114
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
for value in values:
    assert value["originalLifecycleLive"]
    assert value["native"]["sourceCommit"] == source and value["native"]["applicationPid"] == 37276
    assert value["native"]["executableSha256"] == "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
    assert value["native"]["result"]["preferences"] == {"channel": "stable", "automaticCheck": True}
baseline = json.loads((root / "schedule-before.json").read_text(encoding="utf-8-sig"))
assert all(value["schedule"] == baseline for value in values)
assert all(right["elapsedMonitorMs"] > left["elapsedMonitorMs"] for left, right in zip(values, values[1:]))
assert all(right["native"]["result"]["atMs"] > left["native"]["result"]["atMs"]
           for left, right in zip(values, values[1:]))
state_raw = (cycle / "monitor-state.json").read_bytes()
state = json.loads(state_raw.decode("utf-8-sig"))
assert state["sourceCommit"] == source and state["applicationPid"] == 37276 and state["monitorPid"] == 3920
assert state["managerDeadlineEpoch"] == 1791903339 and state["panelDeadlineEpoch"] == 1791903340
destination.write_bytes(prefix)
(root / "monitor-state-at-prefix.json").write_bytes(state_raw)
summary = {
    "frozenAtUtc": datetime.now(timezone.utc).isoformat(), "sourceCommit": source,
    "applicationPid": 37276, "monitorPid": 3920, "monitorExecSession": 46484,
    "rawPrefixSha256": hashlib.sha256(prefix).hexdigest(), "completeObservations": len(values),
    "firstObservedAtUtc": values[0]["observedAtUtc"], "lastObservedAtUtc": values[-1]["observedAtUtc"],
    "observedMonotonicSpanMs": values[-1]["elapsedMonitorMs"] - values[0]["elapsedMonitorMs"],
    "appWebviewSpanMs": values[-1]["native"]["result"]["atMs"] - values[0]["native"]["result"]["atMs"],
    "allObservedPreferencesStableAndAutomatic": True, "allObservedScheduleEntriesUnchanged": True,
    "foregroundObservations": sum(value["native"]["result"]["foreground"] is True for value in values),
    "notContinuousForegroundRequiredOrClaimed": True,
    "rawNetworkOrNativeQueryCountsInstrumented": False,
    "realCycleAccepted": False, "scope": "completed monitor prefix only; no full24h terminal or request-count acceptance",
}
(root / "cycle-prefix-summary.json").write_bytes((json.dumps(summary, ensure_ascii=False, indent=2) + "\n").encode())
print(json.dumps({"completedPrefixObservations": len(values), "originalCycleRestarted": False,
                  "actualObservedSpanHours": summary["observedMonotonicSpanMs"] / 3_600_000}))
