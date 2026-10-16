"""Read-only crosscheck of frozen evidence and retained private MSI originals."""
import hashlib
import json
import re
import sys
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent / "OCX-0110-20261016-WIN-RECOVERY-MSI-RUN"
PROJECT = ROOT.parents[2]
SID = re.compile(r"S-1-\d+(?:-\d+){1,}")


def read(relative):
    return json.loads((ROOT / relative).read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def stamp(value):
    return datetime.fromisoformat(value.replace("Z", "+00:00"))


manifest = read("manifest.json")
files = {row["path"] for row in manifest["files"]}
assert files == {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*")
                 if p.is_file() and p.name != "manifest.json"}
for row in manifest["files"]:
    data = (ROOT / row["path"]).read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"], row["path"]
for row in read("original-copies.json"):
    data = (ROOT / row["path"]).read_bytes()
    assert data == Path(row["sourceRuntimePath"]).read_bytes()
    assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    assert not SID.search(data.decode("utf-8-sig"))
scope = read("scope.json")
source = "823a8a0615153068594daef530c2e80f1d95fc29"
image = "2614489ff3a7c62110fb74fd288432b37c3abeaa5cdaa3119551d66c97640e53"
sid = "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
human = read("recovery/human-window.json")
start, end = map(stamp, scope["actualHumanWindow"])
assert (end - start).total_seconds() == 1800
assert human["explicitHumanStart"] and not human["automaticContinuation"]
assert human["userMessage"] == "允许前台"
assert [human["authorizedAtUtc"], human["expiresAtUtc"]] == scope["actualHumanWindow"]
launch = read("recovery/recovery-launch.json")
assert launch["source"] == source and launch["exeSha256"] == image
token = launch["actualToken"]
assert token["sidSha256"] == sid and not token["elevated"] and token["elevationType"] == 3
assert launch["pid"] == token["processId"] and launch["creationTimeUtc"] == token["creationTimeUtc"]
assert not any(launch[k] for k in ["sandboxUsed", "cdpUsed", "uiInputInjected"])
assert start < stamp(launch["atUtc"]) < end
for name in ["platform-preflight-before-prepare.json", "platform-preflight-before-launch.json"]:
    preflight = read("recovery/" + name)
    assert preflight["fourPlatformRootsAbsent"] and preflight["allSavedNativeDirectoryIdentitiesMatch"]
    assert preflight["logicalPathCount"] == 4 and preflight["physicalDirectoryCount"] == 2
    assert preflight["nativeCount"] == 0 and not preflight["productStarted"]
order = read("recovery/recovery-process-order.json")
census = read("recovery/recovery-census.original.json")
assert len(census) > 400
assert max(len(row["pids"]) for row in census) == order["observedMaximumNativeCount"] == 1
assert all(row["pids"] == [launch["pid"]] and not row["oldKernelExited"] for row in census)
assert all(stamp(a["atUtc"]) < stamp(b["atUtc"]) for a, b in zip(census, census[1:]))
assert 239 <= (stamp(order["atUtc"]) - stamp(launch["atUtc"])).total_seconds() < 245
assert order["source"] == source and order["oldPid"] == launch["pid"]
assert order["newPid"] is None and order["readFailure"] is not None
assert all(not order[k] for k in ["exactKernelOrderPassed", "processSubsetPassed",
                                 "fullNativeRecoveryGatePassed", "productForceTerminated"])
observation = read("recovery/computer-use-observation.json")
assert observation["recoveryPromptObserved"] and observation["chooseDirectoryActionSent"]
assert observation["newModalTitle"] == "选择管理器数据目录"
assert "0x80070057" in observation["captureError"]
assert observation["recoveryActivationError"] == "failed to activate captured window"
assert observation["furtherUiInputStopped"] and observation["userHandoffRequested"]
failure = read("recovery/normal-exit-observer.failure.json")
assert failure["message"] == "Bound normal close not observed in 300 seconds"
exit_readback = read("recovery/post-human-exit-readback.json")
assert exit_readback["nativeCount"] == 0 and exit_readback["userReportedExit"]
assert exit_readback["exitCode"] is None and exit_readback["kernelExitTime"] is None
assert not exit_readback["normalExitProven"] and not exit_readback["forceTerminated"]
restored = read("recovery/recovery-restored.json")
assert restored["aclRestored"] and restored["pointerRestoredExact"]
profile = read("recovery/profile-restoration-result.json")
assert profile["fourLogicalPathsRestored"] and profile["physicalDirectoryMoves"] == 2
assert profile["nativeDirectoryIdsPreserved"] and not profile["dataDeleted"]
protection = read("recovery/final-protection-readback.json")
assert protection["nativeCount"] == 0 and protection["pointerMatchesBeforeIsolation"]
assert len(protection["restoredAliases"]) == 4
assert all(row["originalIdentityMatches"] for row in protection["restoredAliases"])
assert len({row["fileId"] for row in protection["restoredAliases"]}) == 2

plan, result = read("msi/execution-plan.json"), read("msi/execution-result.json")
assert plan["source"] == scope["productSource"] == source
frozen_plan = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-HOST-MSI-PREP/execution-plan.original.json"
assert (ROOT / "msi/execution-plan.json").read_bytes() == frozen_plan.read_bytes()
assert result["result"] == "pass" and not result["productLaunched"]
assert result["planSha256"] == sha(frozen_plan.read_bytes())
admin = result["actualAdministratorToken"]
assert admin["elevated"] and admin["elevationType"] == 2 and admin["sidSha256"] == sid
assert read("msi/execution-attempt.json")["result"] == "fail"
assert read("msi/runner-result.json")["completed"]
mh = read("msi/human-window.json")
assert [mh["authorizedAtUtc"], mh["expiresAtUtc"]] == scope["actualHumanWindow"]
assert mh["scope"] == "one-host-msi-retention-execution"
assert mh["explicitHumanStart"] and not mh["automaticContinuation"]
steps = [step for step in result["steps"] if "exitCode" in step]
assert [step["name"] for step in steps] == plan["steps"]
assert len(steps) == 6 and all(step["exitCode"] == 0 for step in steps)
assert all(start < stamp(step["atUtc"]) < end for step in steps)
assert stamp(exit_readback["atUtc"]) < stamp(result["atUtc"]) < stamp(result["finishedAtUtc"]) < end
images = [step for step in result["steps"] if "version" in step]
assert [(x["version"], x["sha256"]) for x in images] == [
    ("0.1.9", plan["packages"]["baseline"]["installedExeSha256"]),
    ("0.1.10", image), ("0.1.10", image)]
phases = [step for step in result["steps"] if "files" in step]
assert [phase["name"] for phase in phases] == ["before-upgrade", "after-upgrade",
    "after-default-uninstall", "after-reinstall", "after-final-default-uninstall"]
assert len(phases[0]["files"]) == 6 and len({x["path"] for x in phases[0]["files"]}) == 6
for phase in phases:
    assert phase["allSeededFixturesUnchanged"] and phase["files"] == phases[0]["files"]
for fixture in phases[-1]["files"]:
    data = Path(fixture["path"]).read_bytes()
    assert len(data) == fixture["bytes"] and sha(data) == fixture["sha256"]
logs = read("private-log-bindings.json")
assert len(logs) == 6
for log in logs:
    raw = Path(log["originalPrivateRuntimePath"]).read_bytes()
    assert len(raw) == log["originalBytes"] and sha(raw) == log["originalSha256"]
    redacted, count = SID.subn("[REDACTED_WINDOWS_SID]", raw.decode(log["originalEncoding"]))
    derived = (ROOT / log["publicDerivedPath"]).read_bytes()
    assert derived == redacted.encode("utf-8") and sha(derived) == log["publicDerivedSha256"]
    assert count == log["redactedSidOccurrences"] > 0 and not log["publicFileIsOriginal"]
    assert "MainEngineThread is returning 0" in redacted and not SID.search(redacted)
final = read("msi/final-state-readback.json")
assert final["source"] is None and scope["finalStateReadbackSourceWasNull"]
assert final["nativeCount"] == 0 and final["stepCount"] == 6
assert not final["priorExePresent"] and not final["testInstalledExePresent"]
assert all(not row[key] for row in final["registry"]
           for key in ["hklm64Present", "hklm32Present", "hkcuPresent"])
assert len(final["retainedFiles"]) == 6 and all(row["matches"] for row in final["retainedFiles"])
assert scope["hostMsiSeededRetentionSubsetPassed"]
for key in ["productRecoveryPassed", "recoveryDefectEstablished", "fullInstallerGatePassed",
            "explicitPurgeTested", "nsisTested", "administratorProductUiTested",
            "normalProductExitCodeKnown", "formalIdlePairRepeated", "newGpuSnapshotTaken",
            "rawSidOrSddlPublished", "historicalFailuresCleared", "taskClosed", "releaseAccepted"]:
    assert not scope[key], key
task_path = PROJECT / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
assert sha(task_path.read_bytes()) == protection["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
sys.path.insert(0, "C:/Users/15119/.codex/skills/project-governance/scripts/lib")
from runtime.kernel import validate_object
receipt = json.loads((PROJECT / ".adg/work/objects/rcp/RCP-OPENCODEX-DESKTOP-182-WIN-RECOVERY-MSI-20261016.json").read_text(encoding="utf-8"))
validate_object(receipt)
assert receipt["result"] == "failed"
assert receipt["fact_impacts"][0]["after_digest"] == sha((PROJECT / "docs/03-开发实施/IMP-OPENCODEX-DESKTOP-20.md").read_bytes().replace(b"\r\n", b"\n"))
print(json.dumps({"status": "verified", "files": len(files), "censusSamples": len(census),
                  "msiSteps": 6, "retainedFiles": 6, "privateMsiLogs": 6,
                  "hostMsiSubsetPassed": True, "fullInstallerGatePassed": False,
                  "recoveryPassed": False}, ensure_ascii=False))
