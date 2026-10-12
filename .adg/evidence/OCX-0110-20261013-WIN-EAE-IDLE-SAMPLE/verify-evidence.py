"""Validate real unpaired idle sample and measurement limits, never budget acceptance."""
import hashlib
import json
import math
import statistics
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
source = "eae85322ebca3b755c034db97cbebf2ea5bf71fc"
image = "f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c"
def read(name):
    return json.loads((root / name).read_text(encoding="utf-8-sig"))
def native(name):
    value = read(name)
    assert value["sourceCommit"] == source and value["applicationPid"] == 37276
    assert value["executableSha256"] == image and value["screenshot"] is None
    return value["result"]
contract = read("measurement-contract.json")
assert contract["requiredIdleElapsedSeconds"] == 600 and contract["sampleIntervalSeconds"] == 1
assert contract["sourceCommit"] == source and contract["applicationPid"] == 37276
assert not any(contract[key] for key in [
    "automaticCheckPreferencesChanged", "performanceBudgetChanged", "appOrCycleRestarted",
    "pairedPerformanceAccepted", "baselinePerformanceAccepted", "completeIdleGateAccepted",
    "framesOrMainThreadAccepted", "realCyclesAccepted", "releaseAccepted"])
before, after = native("scene-before.json"), native("scene-after.json")
for scene in [before, after]:
    assert scene["route"] == "#settings?section=backup"
    assert scene["documentVisibility"] == "visible"
    assert scene["busy"] == {"manager": False, "panel": False, "root": False}
    assert scene["preferences"] == {
        "channel": "stable", "automaticCheck": True, "theme": "system", "visualEffects": "high",
        "glowRender": "mesh", "interfaceScale": 100, "autoPanel": False, "launchWithCodex": False}
preflight = read("sampler-preflight.json")
assert preflight["status"] == "real-readonly-apis-preflight"
assert preflight["sourceCommit"] == source and preflight["applicationPid"] == 37276
assert preflight["logicalProcessorCountAllGroups"] == 20
assert len(preflight["attachedActualProcesses"]) == 7
report = read("sampler-state.json")
assert report["status"] == "sampled-real-ten-minute-window"
terminal = read("sampler-terminal.json")
assert terminal["sourceCommit"] == source and terminal["execSession"] == 25560
assert terminal["samplerPid"] == 36408 and terminal["exitCode"] == 0
assert terminal["finalStdout"]["samples"] == report["samples"]
assert terminal["finalStdout"]["actualObservedElapsedSeconds"] == report["actualObservedElapsedSeconds"]
assert not terminal["controllerProcessRestarted"] and not terminal["applicationOrCycleRestarted"]
assert not terminal["official019BaselineStarted"] and not terminal["pairedPerformanceAccepted"]
assert report["sourceCommit"] == source and report["applicationPid"] == 37276
assert report["executableSha256"] == image and report["samplerPid"] == 36408
assert report["requiredElapsedSeconds"] == 600 and report["sampleIntervalSeconds"] == 1
assert report["logicalProcessorCountAllGroups"] == 20
assert not any(report[key] for key in ["allProcessLifetimeCountAccepted", "queryOrNetworkCountsAccepted",
    "pairedPerformanceAccepted", "completeIdleGateAccepted", "fullNativeGateAccepted", "realCyclesAccepted", "releaseAccepted"])
raw = (root / "resource-samples.ndjson").read_bytes()
assert raw.endswith(b"\n")
samples = [json.loads(line) for line in raw.splitlines() if line.strip()]
assert len(samples) == report["samples"] >= 601
assert [item["sampleIndex"] for item in samples] == list(range(len(samples)))
assert all(b["elapsedNs"] > a["elapsedNs"] for a, b in zip(samples, samples[1:]))
assert all(b["familyCpuSinceStart100ns"] >= a["familyCpuSinceStart100ns"]
           for a, b in zip(samples, samples[1:]))
known = {(item["pid"], item["creationUtc"]): item for item in report["attachedActualProcesses"]}
assert report["observedDistinctProcessHandles"] == len(known)
assert (37276, "2026-10-12T14:53:12.008801+00:00") in known
assert all(item["liveHandleAcquired"] for item in known.values())
for item in known.values():
    if item["pid"] != 37276:
        assert item["parentPid"] in {parent["pid"] for parent in known.values()}
for sample in samples:
    active = sample["activeObservedProcesses"]
    exited = sample["exitedObservedProcesses"]
    combined = active + exited
    assert len({(item["pid"], item["creationUtc"]) for item in combined}) == len(combined)
    assert all((item["pid"], item["creationUtc"]) in known for item in combined)
    assert sum(item["cpuSinceSampleStart100ns"] for item in combined) == sample["familyCpuSinceStart100ns"]
    assert sum(item["workingSetBytes"] for item in active) == sample["workingSetFamilySumBytes"]
    root_process = next(item for item in active if item["pid"] == 37276)
    assert root_process["workingSetBytes"] == sample["rootWorkingSetBytes"]
    assert root_process["image"] == "opencodex-desktop.exe"
    assert sample["window"]["hwnd"] == 1051698 and sample["window"]["ownerPid"] == 37276
    assert sample["window"]["dpi"] == 192 and sample["window"]["visible"] and not sample["window"]["minimized"]
    assert all(item["cpuSinceSampleStart100ns"] >= 0 for item in combined)
    assert all(item["workingSetBytes"] > 0 for item in active)
first, last = samples[0], samples[-1]
seconds = (last["elapsedNs"] - first["elapsedNs"]) / 1_000_000_000
cpu_seconds = (last["familyCpuSinceStart100ns"] - first["familyCpuSinceStart100ns"]) / 10_000_000
assert seconds >= 600 and seconds == report["actualObservedElapsedSeconds"]
assert cpu_seconds == report["observedFamilyCpuSeconds"]
assert math.isclose(cpu_seconds / seconds * 100, report["observedFamilyCpuSingleLogicalProcessorPercent"], rel_tol=1e-12)
assert math.isclose(cpu_seconds / seconds * 100 / 20, report["observedFamilyCpuAllLogicalProcessorsPercent"], rel_tol=1e-12)
assert statistics.fmean(item["workingSetFamilySumBytes"] for item in samples) == report["familySummedWorkingSetBytesMean"]
assert max(item["workingSetFamilySumBytes"] for item in samples) == report["familySummedWorkingSetBytesMax"]
assert statistics.fmean(item["rootWorkingSetBytes"] for item in samples) == report["rootWorkingSetBytesMean"]
assert max(item["rootWorkingSetBytes"] for item in samples) == report["rootWorkingSetBytesMax"]
assert report["stateHashesBefore"] == report["stateHashesAfter"]
assert report["stateHashesBefore"]["preferences.json"] == "674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa"
assert report["stateHashesBefore"]["update-schedule.json"] == "6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20"
assert report["dailyExecutableSha256After"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
window_states = [json.loads(item) for item in sorted({json.dumps(item["window"], sort_keys=True) for item in samples})]
assert report["actualWindowStates"] == window_states
host = read("host-and-webview.json")
assert host["sourceCommit"] == source and host["currentAppPid"] == 37276
assert host["logicalProcessorsSum"] == report["logicalProcessorCountAllGroups"]
assert host["buildNumber"] == "26100" and host["ordinaryActualTokenType"] == 3
assert not host["baselineStarted"] and not host["pairedPerformanceAccepted"]
admission = read("baseline-and-task-admission.json")
assert admission["baselineSourceCommit"] == "de6862d40389d527527b9db8ef52e186a58b625f"
assert admission["baselineExecutableSha256"] == "626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608"
assert not admission["baselineStarted"] and not admission["profileEnvironmentIsolationPreviouslyProven"]
assert not admission["taskLocalExists"] and admission["taskExactRequestedSourceTree"] == []
assert admission["taskExactAllRefsHistory"] == [] and len(admission["trackedWorkPathsAtRequestedSource"]) == 11
assert admission["originalPerformanceStatus"] == "failed"
assert any(item["status"] == "failed" and item["name"] == "review:scheduler-performance-0110"
           for item in admission["originalPerformanceChecks"])
assert not admission["taskReconstructed"] and not admission["priorFailureCleared"]
handle = read("sampler-live-handle-confirmation.json")
assert handle["samplerPid"] == report["samplerPid"] and handle["actualLiveHandle"]
assert handle["exactImageAndArgvMatched"] and handle["execSession"] == 25560
assert handle["actualCreationFileTime100ns"] == 134362979108800824
assert handle["formatterPrecisionDelta100ns"] == 4 and not handle["processOrSamplerRestarted"]
assert handle["firstHandleCheckWrapperError"] == "Sampler handle identity differs"
life = read("scene-lifecycle.json")
assert life["sourceCommit"] == source and life["applicationPid"] == 37276 and life["desktopApplicationCount"] == 1
assert {item["processId"] for item in life["processReadbacks"]} == {37276, 24300, 3920}
assert all(item["liveHandleVerified"] and not item["hasExited"] for item in life["processReadbacks"])
assert life["applicationToken"]["elevated"] is False and life["applicationToken"]["elevationType"] == 3
assert life["executableSha256"] == image and life["dailyExecutableSha256"] == report["dailyExecutableSha256After"]
assert not any(life[key] for key in ["clocksOrTimersChanged", "manualUpdateCheckOrInstallInvoked",
    "pairedPerformanceAccepted", "realCyclesAccepted", "fullNativeGateAccepted", "releaseAccepted"])
files = sorted(p for p in root.rglob("*") if p.is_file() and p.name not in {"manifest.json", "archive-verification.json"})
assert all(p.suffix.lower() not in {".exe", ".dll", ".msi", ".zip", ".pending"} for p in files)
entries = [{"path": p.relative_to(root).as_posix(), "bytes": p.stat().st_size,
            "sha256": hashlib.sha256(p.read_bytes()).hexdigest()} for p in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "sourceCommit": source, "filesVerified": len(entries),
    "scope": "real final ordinary candidate unpaired600-second observed-family CPU and arithmetic working-set sum, native scene and measurement limitations",
    "samples": len(samples), "actualElapsedSeconds": seconds,
    "official019BaselineStarted": False, "pairedIdleBudgetAccepted": False,
    "uniquePhysicalResidentMemoryMeasured": False, "allTransientLifetimeCountsAccepted": False,
    "queryOrNetworkCountsAccepted": False, "framesOrMainThreadAccepted": False,
    "existingPerformanceFailureCleared": False, "fullNativeGateAccepted": False,
    "realCyclesAccepted": False, "formalTaskReceipt": False, "releaseAccepted": False,
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries), "samples": len(samples),
                  "realElapsedSeconds": seconds, "pairedPerformanceAccepted": False}))
