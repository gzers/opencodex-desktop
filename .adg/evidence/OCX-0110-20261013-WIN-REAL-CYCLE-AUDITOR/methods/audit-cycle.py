"""Read-only snapshot and evidence audit of the existing final-source cycle.
Never activate, query, mutate product files, replace timers or start a cycle.
"""
import argparse
import copy
import ctypes
import ctypes.wintypes as wt
import hashlib
import json
from datetime import datetime, timezone
from pathlib import Path

source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
image = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
evidence = Path(__file__).resolve().parents[1]
baseline_dir = evidence.parent / "OCX-0110-20261012-WIN-EAE-REAL-STABLE-PANEL-CYCLE"
runtime = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\windows0110")
cycle = runtime / "cycles/eae-stable-panel-20261012"
period_seconds = 86400
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
sha = lambda value: hashlib.sha256(value).hexdigest()
def encode(value):
    return (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()
def fail_if(condition, message):
    if condition:
        raise ValueError(message)

def evaluate(initial, baseline, observations):
    """Tool-only pure validator. Its result never owns TASK/gate acceptance."""
    fail_if(initial["sourceCommit"] != source or initial["applicationPid"] != 37276,
            "Initial source/PID differs")
    fail_if(initial["executableSha256"] != image or not observations, "Initial image or observations missing")
    anchors = {"manager-stable": initial["result"]["managerQueryCompletedAtMs"],
               "panel-latest": initial["result"]["panelQueryCompletedAtMs"]}
    for target in anchors:
        original = baseline["entries"][target]
        fail_if(original["failures"] != 0 or original["next_due"] - original["last_success"] != period_seconds,
                "Original full86400-second successful baseline differs")
    previous = None
    results = {target: {"observedNextPersistedSuccess": False, "firstSuccessObservationIndex": None,
                       "strictElapsedAtSuccessObservationMs": None, "candidateNextDeadlineEpoch": None}
               for target in anchors}
    for index, observation in enumerate(observations):
        native = observation["native"]
        value = native["result"]
        fail_if(native["sourceCommit"] != source or native["applicationPid"] != 37276 or
                native["executableSha256"] != image or not observation["originalLifecycleLive"],
                "Observed source/image/original lifecycle differs")
        fail_if(value["preferences"] != {"channel": "stable", "automaticCheck": True},
                "Observed active cycle preference differs")
        if previous is not None:
            fail_if(observation["elapsedMonitorMs"] <= previous["elapsedMonitorMs"] or
                    value["atMs"] <= previous["native"]["result"]["atMs"], "Observed monotonic time went backward")
        fail_if(observation["schedule"]["entries"].get("manager-beta") != baseline["entries"].get("manager-beta"),
                "Unrelated beta entry changed during the stable/panel lifecycle")
        for target, anchor in anchors.items():
            original = baseline["entries"][target]
            current = observation["schedule"]["entries"][target]
            succeeded = current["last_success"]
            fail_if(succeeded is None or succeeded < original["last_success"], "Persisted success went backward")
            if succeeded > original["last_success"]:
                fail_if(succeeded < original["next_due"], "New persisted success precedes original deadline")
                elapsed = value["atMs"] - anchor
                fail_if(elapsed < period_seconds * 1000, "Observation of new success before strict86400000ms")
                if current["failures"] == 0 and current["next_due"] - succeeded == period_seconds:
                    if not results[target]["observedNextPersistedSuccess"]:
                        results[target] = {
                            "observedNextPersistedSuccess": True, "firstSuccessObservationIndex": index,
                            "strictElapsedAtSuccessObservationMs": elapsed,
                            "candidateNextDeadlineEpoch": current["next_due"], "persistedSuccessEpoch": succeeded,
                        }
                    if target == "manager-stable":
                        checked = value["manager"]["lastCheckedAt"]
                        fail_if(value["manager"]["error"] is not None or not checked,
                                "Manager persisted transition lacks successful cached native status")
                        checked_time = datetime.fromisoformat(checked.replace("Z", "+00:00")).timestamp()
                        fail_if(checked_time < original["next_due"], "Manager native readback timestamp precedes original due")
                    else:
                        fail_if(not value["panelCache"] or not value["panelCache"].get("version"),
                                "Panel persisted transition lacks cached native version")
        previous = observation
    last = observations[-1]
    if all(value["observedNextPersistedSuccess"] for value in results.values()):
        status = "recorded_next_successes_require_terminal_and_scope_review"
    elif datetime.fromisoformat(last["observedAtUtc"].replace("Z", "+00:00")).timestamp() < \
            min(baseline["entries"][target]["next_due"] for target in anchors):
        status = "pending_original_deadlines"
    else:
        status = "deadline_elapsed_waiting_for_actual_terminal_or_eligible_foreground"
    return {
        "status": status, "sourceCommit": source, "applicationPid": 37276,
        "observationsAudited": len(observations), "targets": results,
        "observedElapsedSpanMs": last["elapsedMonitorMs"] - observations[0]["elapsedMonitorMs"],
        "lastObservedAtUtc": last["observedAtUtc"], "lastForeground": last["native"]["result"]["foreground"],
        "limits": [
            "Elapsed requirement is checked at the success observation; exact earliest query-start elapsed time is not instrumented.",
            "Persisted success plus cached native state is evidence for a recorded transition, not proof of automatic trigger or raw request counts.",
            "This tool requires separate owner session terminal/cleanup, actual foreground outcome and human scope review.",
            "Whole beta6h, sleep/resume/cross-startup/clock, performance and original TASK required checks remain independent gates.",
        ],
        "rawNetworkOrQueryCountAccepted": False, "earliestQueryStartElapsedProven": False,
        "automaticTriggerProven": False, "originalTaskRequiredChecksRecovered": False,
        "realFullCycleAccepted": False, "pairedPerformanceAccepted": False,
        "fullNativeGateAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
    }

def self_test(initial, baseline, observations):
    tests = []
    for name in ["wrong-source", "backward-monotonic", "early-success"]:
        fixture = copy.deepcopy(observations[:2])
        if name == "wrong-source":
            fixture[0]["native"]["sourceCommit"] = "invalid-fixture-source"
        elif name == "backward-monotonic":
            fixture[1]["native"]["result"]["atMs"] = fixture[0]["native"]["result"]["atMs"]
        else:
            original = baseline["entries"]["manager-stable"]
            fixture[0]["schedule"]["entries"]["manager-stable"] = {
                "last_success": original["next_due"], "next_due": original["next_due"] + period_seconds, "failures": 0}
        try:
            evaluate(initial, baseline, fixture)
        except ValueError as error:
            tests.append({"name": name, "validatorRejected": True, "reason": str(error), "nativeEvidence": False})
        else:
            raise AssertionError("Validator accepted unsafe synthetic tooling fixture: " + name)
    return {"scope": "validator-only in-memory synthetic refusal tests; no product time or state injection",
            "tests": tests, "nativeCycleAccepted": False, "realResponsesOrClockInjected": False}

class FileTime(ctypes.Structure):
    _fields_ = [("low", wt.DWORD), ("high", wt.DWORD)]
    def integer(self):
        return (self.high << 32) | self.low
def expected_creation(value):
    seconds, fraction = value.removesuffix("Z").split(".")
    whole = int(datetime.strptime(seconds, "%Y-%m-%dT%H:%M:%S").replace(tzinfo=timezone.utc).timestamp())
    return whole * 10_000_000 + int(fraction.ljust(7, "0")) + 116444736000000000
def actual_processes():
    kernel = ctypes.WinDLL("kernel32", use_last_error=True)
    kernel.OpenProcess.argtypes = [wt.DWORD, wt.BOOL, wt.DWORD]
    kernel.OpenProcess.restype = wt.HANDLE
    kernel.CloseHandle.argtypes = [wt.HANDLE]
    kernel.GetProcessTimes.argtypes = [wt.HANDLE, ctypes.POINTER(FileTime), ctypes.POINTER(FileTime),
                                      ctypes.POINTER(FileTime), ctypes.POINTER(FileTime)]
    kernel.WaitForSingleObject.argtypes = [wt.HANDLE, wt.DWORD]
    kernel.WaitForSingleObject.restype = wt.DWORD
    values = []
    for pid, created_at in [(37276, "2026-10-12T14:53:12.0088010Z"),
                            (24300, "2026-10-12T14:53:10.4430287Z"),
                            (3920, "2026-10-12T14:55:52.1278297Z")]:
        handle = kernel.OpenProcess(0x101000, False, pid)
        if not handle:
            values.append({"pid": pid, "actualLiveHandle": False, "queryError": ctypes.get_last_error(),
                           "scope": "Current handle unavailable; owner-session terminal result required, not inferred"})
            continue
        try:
            started, ended, cpu_kernel, cpu_user = FileTime(), FileTime(), FileTime(), FileTime()
            if not kernel.GetProcessTimes(handle, ctypes.byref(started), ctypes.byref(ended),
                                          ctypes.byref(cpu_kernel), ctypes.byref(cpu_user)):
                raise ctypes.WinError(ctypes.get_last_error())
            fail_if(started.integer() != expected_creation(created_at), "PID creation identity differs")
            values.append({"pid": pid, "creationFileTime100ns": started.integer(), "exactCreationMatched": True,
                           "actualLiveHandle": kernel.WaitForSingleObject(handle, 0) == 258})
        finally:
            kernel.CloseHandle(handle)
    return values

def main():
    parser = argparse.ArgumentParser()
    parser.add_argument("--output-name", required=True)
    args = parser.parse_args()
    fail_if(not args.output_name.replace("-", "").isalnum(), "Output name must be a single owned directory name")
    output = evidence / args.output_name
    fail_if(output.exists(), "Never overwrite earlier frozen audit")
    for path in [evidence, evidence.parent, cycle, runtime]:
        fail_if(path.is_symlink() or (path.stat().st_file_attributes & 0x400) != 0, "Reparse source/output rejected")
    processes = actual_processes()
    raw = (cycle / "observations.ndjson").read_bytes()
    last_lf = raw.rfind(b"\n")
    fail_if(last_lf < 0, "No complete observation available")
    prefix = raw[:last_lf + 1]
    observations = [json.loads(line) for line in prefix.splitlines() if line.strip()]
    initial_raw = (baseline_dir / "native-initialization.json").read_bytes()
    baseline_raw = (baseline_dir / "schedule-baseline.json").read_bytes()
    initial = json.loads(initial_raw.decode("utf-8-sig"))
    baseline = json.loads(baseline_raw.decode("utf-8-sig"))
    result = evaluate(initial, baseline, observations)
    guarded = self_test(initial, baseline, observations)
    monitor_raw = (cycle / "monitor-state.json").read_bytes()
    monitor = json.loads(monitor_raw.decode("utf-8-sig"))
    fail_if(monitor["sourceCommit"] != source or monitor["applicationPid"] != 37276, "Monitor source differs")
    fail_if(monitor["maximumHours"] != 26, "Original26h limit differs")
    result.update({
        "auditAtUtc": datetime.now(timezone.utc).isoformat(), "currentActualProcesses": processes,
        "allThreeOriginalHandlesCurrentlyLive": all(value["actualLiveHandle"] for value in processes),
        "monitorStatus": monitor["status"], "monitorFinishedAtUtc": monitor.get("finishedAtUtc"),
        "monitorCleanupPreferenceRestored": monitor.get("originalUpdatePreferencesRestored", False),
        "ownerTerminalToolResultVerified": False,
        "rawSnapshotHashes": {
            "observations-prefix.ndjson": sha(prefix), "native-initialization.json": sha(initial_raw),
            "schedule-baseline.json": sha(baseline_raw), "monitor-state.json": sha(monitor_raw)},
    })
    output.mkdir()
    for name, value in [("observations-prefix.ndjson", prefix), ("native-initialization.json", initial_raw),
                        ("schedule-baseline.json", baseline_raw), ("monitor-state.json", monitor_raw),
                        ("audit-result.json", encode(result)), ("validator-self-test.json", encode(guarded))]:
        (output / name).write_bytes(value)
    print(json.dumps({"status": result["status"], "observationsAudited": len(observations),
                      "actualOriginalHandlesLive": result["allThreeOriginalHandlesCurrentlyLive"],
                      "validatorRefusalTests": len(guarded["tests"]), "fullCycleAccepted": False}))

if __name__ == "__main__":
    main()
