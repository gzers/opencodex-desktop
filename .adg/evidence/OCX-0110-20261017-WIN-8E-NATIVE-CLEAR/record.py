"""Verify the native restart and actual uninstaller cleanup, without replay."""
import hashlib
import json
import re
import sys
from datetime import datetime
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RUN = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-8e-cleanup-20261017T213054")
ADMITTED = ROOT.parent / "OCX-0110-20261017-WIN-8E-CANDIDATE"
SOURCE = "8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b"
EXE = "b948fb7e6601aa606551e72bb3afc15f0cf8cbc02b7c46584e69584f6565c1a8"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def verify():
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert data == Path(row["runtimePath"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    binding = read(ROOT / "candidate-binding.json")
    assert sha((ADMITTED / "manifest.json").read_bytes()) == binding["manifestSha256"]
    candidate = read(ADMITTED / "verification.json")
    assert candidate["source"] == SOURCE and candidate["runId"] == 39033532953
    assert candidate["sameSourceFullCiRunId"] == 39032153138
    descriptor = read(ADMITTED / "original/candidate-descriptor.json")
    assert descriptor["inMemoryNssVariantSha256"] == EXE
    original = ROOT / "original"
    human = read(original / "human-window.json")
    assert human["explicitHumanStart"] and not human["automaticContinuation"]
    started = datetime.fromisoformat(human["authorizedAtUtc"].replace("Z", "+00:00"))
    deadline = datetime.fromisoformat(human["expiresAtUtc"].replace("Z", "+00:00"))
    assert (deadline - started).total_seconds() == 1800
    assert human["userMessage"] == "同意继续"
    install = read(original / "installation.json")
    assert install["source"] == SOURCE and install["exitCode"] == 0
    assert install["exeSha256"] == EXE and not install["autoLaunch"]
    assert install["installerSha256"] == "d7016d62f5e14e2784b01cb5c5494e69fcdb711843f9c03849637173e9088acc"
    contract = read(original / "contract.json")
    assert contract["candidateSource"] == SOURCE and contract["candidateExeSha256"] == EXE
    reuse = read(original / "actual-root-reuse.json")
    assert reuse["actualPriorRoot"] == contract["selectedRoot"] and reuse["testPreparationOnly"]
    assert not reuse["newUiBindingClaimed"]
    exits = []
    for phase in ("custom-first", "custom-restart"):
        launch = read(original / (phase + "-launch.json"))
        exit_record = read(original / (phase + "-exit.json"))
        assert launch["source"] == SOURCE and launch["exeSha256"] == EXE
        assert launch["retainedHandle"] and not launch["sandboxUsed"] and not launch["cdpUsed"]
        token = launch["actualToken"]
        assert not token["elevated"] and token["elevationType"] == 3
        assert token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
        assert exit_record["pid"] == launch["pid"] and exit_record["exitCode"] == 0
        assert exit_record["nativeCount"] == 0 and not exit_record["forcedTermination"]
        assert exit_record["kernelTimes"]["exited"] and exit_record["kernelTimes"]["retainedHandle"]
        assert datetime.fromisoformat(exit_record["atUtc"]) < deadline
        action = read(original / (phase + "-close-action.json"))
        assert "Alt+F4" in action["action"] and not action["forceTerminated"]
        exits.append(exit_record)
    assert int(exits[1]["kernelTimes"]["creationFileTime"]) >= int(exits[0]["kernelTimes"]["exitFileTime"])
    for name in ("custom-first-ui.original.json", "custom-restart-ui.original.json"):
        tree = read(original / name)["accessibility"]["tree"]
        assert "数据目录 · 当前使用" in tree and "OPENCODEX_HOME · 当前使用" in tree
        assert "foreground-99-roots-20261017T172126\\selected-root" in tree
    backup_id = "bk_20261017133409_c058f8a4"
    for name in ("actual-backup-ui.original.json", "backup-after-restart-ui.original.json"):
        ui = read(original / name)["accessibility"]
        assert backup_id in ui["tree"]
    before_restart = read(original / "before-restart.json")
    persistent = read(original / "restart-persistence.json")
    assert persistent["allFilesUnchanged"] and persistent["pointerUnchangedExact"]
    assert persistent["files"] == before_restart["files"]
    assert len(persistent["files"]) == 4 and persistent["backupId"] == backup_id
    before = read(original / "before-clear.json")
    assert before["allCopiedBytesVerified"] and before["normalProductExits"] == 2
    saved = Path(before["preservedCopy"])
    assert len(before["files"]) == 18
    for row in before["files"]:
        data = (saved / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    for row in before_restart["files"]:
        relative = Path(row["path"]).relative_to(Path(before["root"]))
        data = (saved / relative).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    assert any(row["path"] == ".backup-w2.lock" for row in before["files"])
    clear = read(original / "actual-uninstall-clear-result.json")
    assert clear["source"] == SOURCE and clear["userMessage"] == "已卸载"
    assert clear["humanCompletionReported"] and clear["rootClearedByActualUninstaller"]
    assert not clear["uninstallerCheckboxIndependentReadbackProved"]
    assert clear["uninstallerActualExitCode"] is None
    assert not clear["helperDirectlyInvokedByHarness"]
    for key in ("testRootAbsent", "startupPointerAbsent", "mainExecutableAbsent", "uninstallRegistrationAbsent"):
        assert clear[key]
    assert clear["nativeCount"] == clear["uninstallerProcessCount"] == 0
    assert clear["actualDataFilesPreservedInCopy"] == 18 and clear["allSavedCopyBytesVerified"]
    for row in clear["unrelatedFilesPreserved"]:
        assert sha(Path(row["path"]).read_bytes()) == row["sha256"]
    restored = read(original / "profile-restoration-result.json")
    assert restored["fourLogicalPathsRestored"] and restored["nativeDirectoryIdsPreserved"]
    assert restored["physicalDirectoryMoves"] == 2
    final = read(original / "final-protection.json")
    assert final["nativeCount"] == final["uninstallerProcessCount"] == 0
    assert final["originalProfileNativeIdsRestored"] and final["originalPointerRestoredExact"]
    assert final["originalPointerSha256"] == "f62b7543fbb2b258dcaf15324dae91fea96c83d95c969065bfbf10428ac7012c"
    assert final["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert final["taskSha256"] == "bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93"
    assert final["foregroundReleased"] and final["nextUiRequiresNewHumanWindow"]
    assert datetime.fromisoformat(final["atUtc"]) < deadline
    assert not final["originalRealDataDeleted"] and final["testPreCleanupCopyPreserved"]
    return {
        "source": SOURCE, "installedExeSha256": EXE, "ordinaryRestartReadbackPassed": True,
        "normalProductExits": 2, "normalProductExitCodes": [0, 0],
        "metadataPreferencesAndRealBackupFilesUnchangedAcrossRestart": 4,
        "actualUninstallerClearedPreviouslyUsedCustomRoot": True,
        "humanCheckboxAndUninstallCompletionReported": True,
        "independentCheckboxCheckedStateRecorded": False, "uninstallerActualExitCode": None,
        "testFilesPreservedInPrivateCopy": 18, "originalConfigurationRestored": True,
        "finalNativeCount": 0, "foregroundReturnedBeforeDeadline": True,
        "old99UnknownExitRetrospectivelyFilled": False,
        "fullWindowsGatePassed": False, "taskClosed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
else:
    assert not (ROOT / "manifest.json").exists()
    names = [
        "human-window.json", "contract.json", "installation.json", "actual-root-reuse.json",
        "profile-isolation-plan.json", "profile-isolation-result.json",
        "custom-first-launch.json", "custom-first-ui.original.json", "custom-first-close-action.json",
        "custom-first-exit.json", "actual-backup-ui.original.json", "before-restart.json",
        "custom-restart-launch.json", "custom-restart-ui.original.json",
        "backup-after-restart-ui.original.json", "restart-persistence.json",
        "custom-restart-close-action.json", "custom-restart-exit.json", "before-clear.json",
        "uninstaller-launch.json", "uninstaller-before-human.original.json",
        "actual-uninstall-clear-result.json", "new-profile-preservation.json",
        "profile-restoration-result.json", "final-protection.json",
        "run-binding.ps1", "launch-phase.ps1", "retained-process-times.ps1", "restore-after-clear.ps1",
    ]
    copies = []
    for name in names:
        data = (RUN / name).read_bytes()
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
        target = ROOT / "original" / name
        target.parent.mkdir(exist_ok=True)
        with target.open("xb") as stream:
            stream.write(data)
        copies.append({"path": "original/" + name, "runtimePath": str(RUN / name),
                       "bytes": len(data), "sha256": sha(data)})
    write(ROOT / "copies.json", copies)
    write(ROOT / "candidate-binding.json", {"directory": ADMITTED.name,
          "manifestSha256": sha((ADMITTED / "manifest.json").read_bytes())})
    write(ROOT / "verification.json", verify())
    (ROOT / ".gitattributes").write_bytes(b"* -text\n")
    write(ROOT / "manifest.json", {"files": [
        {"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size,
         "sha256": sha(p.read_bytes())}
        for p in sorted(ROOT.rglob("*")) if p.is_file()
    ]})
print(json.dumps(read(ROOT / "verification.json")))
