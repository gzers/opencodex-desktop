import hashlib
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parent

def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))

manifest = read("manifest.json")
actual = {path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*")
          if path.is_file() and path.relative_to(ROOT).as_posix() not in {"manifest.json", "verification.json"}}
assert actual == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
launch = read("state-floor-retry-launch.json")
assert launch["source"] == "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
assert launch["exeSha256"] == "7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9"
assert launch["applicationUser"]["elevated"] is False and launch["applicationUser"]["elevationType"] == 3
assert launch["sandboxUsed"] and launch["cdpUsed"] and launch["realMalformedState"]
method = read("logpoint-method.json")
assert method["expectedDebuggerPauses"] == 0
assert not method["stubbedResult"] and not method["nativeBridgeReplaced"] and not method["clockOrTimerChanged"]
assert method["nowPerf"] < 20000
observation = read("logpoint-observation.json")
result = read("logpoint-result.json")
assert observation["pauses"] == result["pauses"] == 0
plan_events = [event for event in observation["events"] if event["command"] == "update_schedule_plan"]
assert len(plan_events) == 2 and plan_events[0]["status"] == "rejected" and plan_events[1]["status"] == "fulfilled"
assert 44000 <= plan_events[0]["startPerf"] <= 90000
gap = plan_events[1]["startPerf"] - plan_events[0]["endPerf"]
assert gap >= 1800000 and gap == result["realMonotonicRetryGapMs"]
assert result["noEarlyPlannerCalls"] and result["realRetryFulfilled"] and result["repaired"]
assert not result["queryEvents"] and not result["systemClockOrTimersChanged"]
repair = read("real-state-repair.json")
assert 10000 <= repair["elapsedSinceRealFailureMs"] < 60000
assert repair["nextDeadlinesAreFixtureOnly"] and not repair["frontendTimerChanged"]
assert result["syntheticOnlineWakesUsed"] and not result["fullSchedulerGatePassed"]
exit_record = read("state-floor-retry-exit.json")
assert exit_record["pid"] == launch["pid"] and exit_record["creationTimeUtc"] == launch["creationTimeUtc"]
assert exit_record["exitObserved"] and exit_record["exitCode"] == 0 and not exit_record["forcedTermination"]
finish = read("finish-result.json")
assert finish["nativeCount"] == 0 and finish["exitObserved"] and finish["exitCode"] == 0
assert finish["foregroundBeforeDuePerformed"] and not finish["fullVersionGatePassed"]
initial = read("initial-failure.original.json")
assert not initial["repaired"] and "Bridge is not observable" in initial["error"]
network = read("network-failure.original.json")
assert not network["events"] and not network["repaired"]
result_summary = {
    "status": "verified-instrumented-real-state-error-floor-subset",
    "files": len(actual), "realMonotonicRetryGapMs": gap, "minimumMs": 1800000,
    "actualNativeFailureAndSuccessfulRetry": True,
    "debuggerPauses": 0, "originalFailuresRetained": True,
    "normalExit": True, "nativeCount": 0,
    "futureDeadlineSeedIsRealCycleBaseline": False,
    "fullSchedulerGatePassed": False, "fullVersionGatePassed": False,
}
(ROOT / "verification.json").write_text(json.dumps(result_summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result_summary))
