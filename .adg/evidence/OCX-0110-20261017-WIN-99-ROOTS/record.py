"""Preserve this native run without claiming an unobserved exit or restart."""
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
RUN = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-99-roots-20261017T172126")
SOURCE = "99a62e4ad35e9404cb08922571ad8b77f89214dd"
EXE = "1ab0d12f21fbb7a46ddb0e9278ecbdc316dd6e7e14748206c02a3684f05cf7e3"


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def verify():
    original = ROOT / "original"
    for row in read(ROOT / "copies.json"):
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
        assert data == Path(row["runtimePath"]).read_bytes()
    contract = read(original / "contract.json")
    assert contract["candidateSource"] == SOURCE and contract["candidateExeSha256"] == EXE
    install = read(original / "installation.json")
    assert install["source"] == SOURCE and install["installedExeSha256"] == EXE
    assert install["exitCode"] == 0 and not install["autoLaunch"]
    human = read(original / "human-window.json")
    assert human["explicitHumanStart"] and not human["automaticContinuation"]
    assert human["expiresAtUtc"] == "2026-10-17T09:51:26.3638044Z"
    order = read(original / "recovery-process-order.json")
    assert order["source"] == SOURCE and order["processSubsetPassed"]
    assert order["oldExitCode"] == 0 and order["observedMaximumNativeCount"] == 1
    assert int(order["newKernelTimes"]["creationFileTime"]) >= int(order["oldKernelTimes"]["exitFileTime"])
    launch = read(original / "recovery-launch.json")
    for token in (launch["actualToken"], order["replacementToken"]):
        assert token["elevated"] is False and token["elevationType"] == 3
        assert token["sidSha256"] == "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
    ui = read(original / "custom-root-ui-readback.original.json")["accessibility"]["tree"]
    assert "selected-root 复制" in ui and "selected-root\\opencodex-home 复制" in ui
    assert "数据目录 · 当前使用" in ui and "OPENCODEX_HOME · 当前使用" in ui
    initialized = read(original / "empty-initialization-result.json")
    assert initialized["source"] == SOURCE and initialized["beforeEntries"] == 0
    assert initialized["metadata"] == {"structure_version": "1"}
    assert initialized["activePointerUnchangedExact"]
    assert set(initialized["createdPartitions"]) == {
        "backups", "cache", "exports", "logs", "manager-state", "opencodex-home", "sync-state",
    }
    empty_ui = read(original / "empty-root-initialized-ui.original.json")["accessibility"]["tree"]
    assert "结构有效" in empty_ui and "已创建" in empty_ui
    close = read(original / "replacement-close-action.json")
    assert close["foregroundExpired"] and close["nativeCount"] == 0
    assert close["actualExitCode"] is None and close["actualExitTimeUtc"] is None
    assert not close["normalExit0Proved"] and not close["additionalProductLaunch"]
    retention = read(original / "actual-custom-uninstall-retention.json")
    assert retention["source"] == SOURCE and retention["uninstallerParentExitCode"] == 0
    assert retention["allActualDataBytesPreserved"] and retention["selectedPointerPreservedExact"]
    assert not retention["mainExePresent"] and not retention["uninstallRegistryPresent"]
    assert not retention["explicitCleanupSelected"] and not retention["newProductStarted"]
    for row in retention["actualDataFiles"]:
        data = (RUN / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    restored = read(original / "profile-restoration-result.json")
    assert restored["fourLogicalPathsRestored"] and restored["nativeDirectoryIdsPreserved"]
    assert restored["physicalDirectoryMoves"] == 2
    final = read(original / "final-protection.json")
    assert final["nativeCount"] == 0 and not final["mainExePresent"]
    assert final["originalPointerRestoredExact"] and final["originalProfileNativeIdsRestored"]
    assert final["originalInstallAclRestoredExact"] and final["testProfilesPreserved"]
    assert final["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    assert not final["restartReadbackPassed"] and not final["replacementExit0Proved"]
    task = ROOT.parents[2] / ".adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json"
    assert sha(task.read_bytes()) == read(original / "saved-originals-protection.json")["taskSha256"]
    return {
        "source": SOURCE, "installedExeSha256": EXE,
        "emptyRootInitializationPassed": True,
        "humanSelectedRecoveryProcessOrderAndNativeRootReadbackPassed": True,
        "maximumObservedNativeCount": 1, "recoveryOldExitCode": 0,
        "replacementExitCode": None, "replacementExitTimeUtc": None,
        "replacementExitObserverTimedOut": True,
        "subsequentOrdinaryRestartReadbackPassed": False,
        "actualCustomRootDefaultUninstallRetentionPassed": True,
        "retainedActualFiles": len(retention["actualDataFiles"]),
        "explicitCleanupTested": False, "originalConfigurationRestored": True,
        "finalNativeCount": 0, "fullNativeRecoveryGatePassed": False,
        "fullStorageOrInstallerGatePassed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        data = (ROOT / row["path"]).read_bytes()
        assert len(data) == row["bytes"] and sha(data) == row["sha256"]
    print(json.dumps(read(ROOT / "verification.json")))
    sys.exit(0)

assert not (ROOT / "manifest.json").exists()
names = [
    "contract.json", "human-window.json", "installation.json",
    "profile-isolation-plan.json", "profile-isolation-result.json",
    "recovery-prepared.json", "recovery-launch.json", "recovery-process-order.json",
    "recovery-census.original.json", "recovery-dialog.original.json",
    "recovery-dialog-readback.original.json", "selected-startup-pointer.original.json",
    "replacement-retained-handle.json", "custom-root-ui-readback.original.json",
    "before-empty-initialization.json", "empty-root-correct-field.original.json",
    "empty-root-initialized-ui.original.json", "empty-initialization-result.json",
    "saved-originals-protection.json", "replacement-close-action.json",
    "acl-restored.json", "before-uninstall-files.json", "actual-custom-uninstall-retention.json",
    "new-profile-preservation.json", "profile-restoration-result.json", "final-protection.json",
    "run-binding.ps1", "prepare-recovery.ps1", "launch-recovery.ps1", "launch-phase.ps1",
    "retained-process-times.ps1", "observe-replacement-exit.ps1", "finish-background.ps1",
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
write(ROOT / "verification.json", verify())
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
write(ROOT / "manifest.json", {"files": [
    {"path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p.read_bytes())}
    for p in sorted(ROOT.rglob("*")) if p.is_file()
]})
print(json.dumps(read(ROOT / "verification.json")))
