"""Independently verify identity, failed recovery, GPU rows, and safe exit."""
import hashlib
import json
import re
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]


def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def filetime(value):
    fraction = re.search(r"\.(\d+)(?:Z|[+-]\d{2}:\d{2})$", value)
    digits = fraction.group(1) if fraction else ""
    assert len(digits) <= 7
    instant = datetime.fromisoformat(value.replace("Z", "+00:00")).astimezone(timezone.utc)
    delta = instant.replace(microsecond=0) - datetime(1601, 1, 1, tzinfo=timezone.utc)
    return (delta.days * 86400 + delta.seconds) * 10_000_000 + int(digits.ljust(7, "0"))


binding, notes = read("code-binding.json"), read("execution-notes.json")
human, launch = read("run/human-window.json"), read("run/recovery-launch.json")
order, census = read("run/recovery-process-order.json"), read("run/recovery-census.original.json")
gpu = read("run/gpu-snapshot.original.json")
exit_observer, stop = read("run/normal-exit-observer-ready.json"), read("run/normal-exit.original.json")
restore, protect = read("run/recovery-restored-guarded.json"), read("run/final-protection-readback.json")
ui = read("run/ui-install-root-final-readback.json")
pointer = read("run/restore-guard-current-pointer.original.json")
start, end = filetime(human["authorizedAtUtc"]), filetime(human["expiresAtUtc"])
assert human["explicitHumanStart"] and not human["automaticContinuation"]
assert human["userMessage"] == "允许继续前台测试"
assert end - start == 30 * 60 * 10_000_000
for stamp in [launch["atUtc"], order["atUtc"], gpu["startedAtUtc"], gpu["finishedAtUtc"],
              exit_observer["atUtc"], stop["atUtc"]]:
    assert start <= filetime(stamp) <= end
product_sha = "823a8a0615153068594daef530c2e80f1d95fc29"
image_sha = "2614489ff3a7c62110fb74fd288432b37c3abeaa5cdaa3119551d66c97640e53"
assert binding["productSource"] == launch["source"] == order["source"] == product_sha
assert launch["exeSha256"] == gpu["exeSha256"] == image_sha
assert launch["pid"] == order["oldPid"] == gpu["rootPid"] == stop["kernelTimes"]["pid"]
assert launch["creationTimeUtc"] == gpu["rootCreationUtc"]
token = launch["actualToken"]
assert token["processId"] == launch["pid"] and not token["elevated"] and token["elevationType"] == 3
assert token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
assert token["executable"] == launch["executable"] == gpu["actualRootToken"]["executable"]
assert not gpu["actualRootToken"]["elevated"] and gpu["actualRootToken"]["elevationType"] == 3
assert filetime(token["creationTimeUtc"]) == filetime(launch["creationTimeUtc"])
assert filetime(gpu["actualRootToken"]["creationTimeUtc"]) == filetime(launch["creationTimeUtc"])
assert all(not launch[key] for key in ["sandboxUsed", "cdpUsed", "uiInputInjected"])
assert len(census) == 485 and all(row["pids"] == [launch["pid"]] for row in census)
assert all(not row["oldKernelExited"] for row in census)
assert all(filetime(a["atUtc"]) < filetime(b["atUtc"]) for a, b in zip(census, census[1:]))
assert order["observedMaximumNativeCount"] == 1 and order["strictRawCountLimitPassed"]
assert order["newPid"] is None and order["oldKernelTimes"] is None
assert order["newKernelTimes"] is None and order["replacementToken"] is None
assert order["readFailure"] is not None
assert not order["exactKernelOrderPassed"] and not order["processSubsetPassed"]
assert not order["fullNativeRecoveryGatePassed"] and not order["productForceTerminated"]
anchor = json.loads(pointer["currentPointer"])["anchor"]
assert anchor in ui["accessibility"]["tree"]
assert "LocalCache\\Roaming\\com.gzers.opencodex.desktop" in anchor
assert pointer["nativeCount"] == 0
assert gpu["desktopRootVerified"] and not gpu["controlledSnapshotOnly"]
assert gpu["retainedHandlesUsed"] and gpu["snapshotOnly"] and gpu["gpuCounterReadStatus"] == "available"
assert all(not gpu[key] for key in ["formalIdleSpanMeasured", "individualGlBuffersAttributed",
    "fullPerformanceGatePassed", "rawCommandLinesSaved", "unrelatedProcessRowsSerialized",
    "uiInputInjected", "processesStartedOrStopped"])
members = {(m["pid"], filetime(m["creationUtc"])) for m in gpu["members"]}
assert len(members) == len(gpu["members"]) == 7
assert sum(m["pid"] == launch["pid"] and m["role"] == "native-root" for m in gpu["members"]) == 1
gpu_members = [m for m in gpu["members"] if m["role"] == "gpu"]
assert len(gpu_members) == 1
assert all(row["pid"] == gpu_members[0]["pid"] for row in gpu["gpuCounterInstances"])
assert len(gpu["gpuCounterInstances"]) == 2
for row in gpu["gpuCounterInstances"]:
    assert re.match(rf"^pid_{row['pid']}_", row["instance"])
    for name in ["DedicatedUsage", "SharedUsage", "TotalCommitted", "LocalUsage", "NonLocalUsage"]:
        assert isinstance(row[name], str) and 0 <= int(row[name]) <= 2**64 - 1
shared = sum(int(row["SharedUsage"]) for row in gpu["gpuCounterInstances"])
assert shared == 191668224
assert sha(ROOT / "run/recovery-launch.json") == exit_observer["launchSha256"]
assert int(stop["kernelTimes"]["creationFileTime"]) == filetime(launch["creationTimeUtc"])
assert stop["kernelTimes"]["retainedHandle"] and stop["kernelTimes"]["exited"]
assert int(stop["kernelTimes"]["exitFileTime"]) <= filetime(stop["atUtc"])
assert stop["exitCode"] == 0 and stop["nativeCount"] == 0 and not stop["forceTerminated"]
assert restore["aclRestored"] and restore["pointerRestoredExact"] and restore["nativeCount"] == 0
assert restore["guardCurrentReadbackSha256"] == sha(ROOT / "run/restore-guard-current-pointer.original.json")
assert restore["uiReadbackSha256"] == sha(ROOT / "run/ui-install-root-final-readback.json")
assert not restore["rawSddlOrSidPublished"] and not restore["dataMovedOrDeleted"]
assert protect["nativeCount"] == 0 and protect["aclRestoredExact"] and protect["pointerRestoredExact"]
assert protect["dailyExecutableSha256"] == "43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67"
assert protect["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
assert all(p["exists"] for p in protect["savedProfileDirectories"])
for item in binding["originalCopies"]:
    path = ROOT / item["path"]
    assert path.stat().st_size == item["bytes"] and sha(path) == item["sha256"]
for identity, expected in binding["sourceBlobs"].items():
    data = subprocess.check_output(["git", "show", identity], cwd=PROJECT)
    assert hashlib.sha256(data).hexdigest() == expected
for name, expected in binding["frozenControllerSha256"].items():
    assert sha(PROJECT / binding["frozenControllerDirectory"] / name) == expected
assert binding["privateAclAndRawSidOriginalExcluded"] and not notes["automaticRetry"]
assert not notes["productRecoveryDefectEstablished"] and not notes["fullNativeRecoveryGatePassed"]
assert not list(ROOT.rglob("*private-originals*"))
if (ROOT / "manifest.json").exists():
    for item in read("manifest.json")["files"]:
        path = ROOT / item["path"]
        assert path.stat().st_size == item["bytes"] and sha(path) == item["sha256"]
print(json.dumps({
    "status": "verified-one-attempt-and-native-snapshot", "productSource": product_sha,
    "runtimeOriginals": len(binding["originalCopies"]), "censusSamples": len(census),
    "observedMaximumNativeCount": 1, "replacementObserved": False,
    "recoveryResult": "failed-preconditions-not-isolated", "productRecoveryDefectEstablished": False,
    "actualOrdinaryTokenBound": True, "nativeGpuSnapshotVerified": True,
    "gpuCounterInstances": len(gpu["gpuCounterInstances"]), "gpuSharedUsageBytes": str(shared),
    "gpuProcessRssBytes": gpu_members[0]["workingSetBytes"],
    "normalExitCode": 0, "nativeCountAfterExit": 0, "aclAndPointerRestoredExact": True,
    "sourceCiRawJobLogsFullyVerified": False, "fullNativeRecoveryGatePassed": False,
    "fullPerformanceGatePassed": False, "historicalFailuresCleared": False, "releaseAccepted": False,
}, indent=2))
