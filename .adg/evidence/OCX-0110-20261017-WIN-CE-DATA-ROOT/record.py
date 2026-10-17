"""Preserve actual default-root acceptance, failed recovery, and the focused fix."""
import hashlib
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUN = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-ce-roots-20261017T130444")
FIX = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop/test/runtime/empty-root-fix-20261017T1320")


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def write(path, value):
    with path.open("x", encoding="utf-8", newline="\n") as stream:
        json.dump(value, stream, ensure_ascii=False, indent=2)
        stream.write("\n")


def verify():
    original = ROOT / "original"
    human = read(original / "human-window.json")
    assert human["authorizedAtUtc"] == "2026-10-17T05:04:44.1341211Z"
    assert human["expiresAtUtc"] == "2026-10-17T05:34:44.1341211Z"
    recovery = read(original / "recovery-process-order.json")
    assert not recovery["processSubsetPassed"] and recovery["newPid"] is None
    assert recovery["observedMaximumNativeCount"] == 1 and recovery["oldExitCode"] == 0
    incomplete = read(original / "recovery-ui-incomplete.json")
    assert not incomplete["selectionCompleted"] and not incomplete["automaticRetry"]
    assert not incomplete["productDefectProven"] and incomplete["selectedRootEmpty"]
    contract = read(original / "contract.json")
    assert contract["candidateSource"] == "ce670ce76bb1c29f70989ee54838d5b31e97059c"
    assert contract["candidateExeSha256"] == "059bc70d158e66863cab0aa0b828c4107806724e3b13e2b2cd337fc33ccf77a5"
    for phase in ("default-root", "default-restart"):
        launch = read(original / (phase + "-launch.json"))
        exit = read(original / (phase + "-exit.json"))
        assert launch["source"] == contract["candidateSource"] and launch["exeSha256"] == contract["candidateExeSha256"]
        assert not launch["actualToken"]["elevated"] and launch["actualToken"]["elevationType"] == 3
        assert exit["exitCode"] == 0 and exit["nativeCount"] == 0 and not exit["forcedTermination"]
        assert exit["kernelTimes"]["retainedHandle"] and exit["kernelTimes"]["exited"]
        ui = read(original / (phase.replace("-root", "-root") + "-ui.json"))
        assert "install with spaces\\data" in ui["accessibility"]["tree"]
        assert "数据目录 · 当前使用" in ui["accessibility"]["tree"]
    assert int(read(original / "default-restart-exit.json")["kernelTimes"]["creationFileTime"]) > int(read(original / "default-root-exit.json")["kernelTimes"]["exitFileTime"])
    assert "目标目录结构不是当前版本" in read(original / "empty-root-initialize-ui.json")["accessibility"]["tree"]
    assert "未能确认面板运行状态" in read(original / "migration-result-ui.json")["accessibility"]["tree"]
    backup = read(original / "actual-backup-ui.json")["accessibility"]["tree"]
    assert "管理器偏好备份已生成" in backup and "bk_20261017051442_80c65217" in backup
    retained = read(original / "actual-default-uninstall-retention.json")
    assert retained["realProductDefaultRootBound"] and retained["uninstallExitCode"] == 0
    assert retained["before"] == retained["after"] and len(retained["after"]) == 4
    for row in retained["after"]:
        path = RUN / "install with spaces" / "data" / row["relative"]
        assert sha(path) == row["sha256"] and path.stat().st_size == row["bytes"]
    final = read(original / "final-protection.json")
    assert final["nativeCount"] == 0 and not final["mainExePresent"]
    assert final["originalPointerRestored"] and final["foregroundReleased"] and not final["dataDeleted"]
    assert final["originalPreferencesSha256"] == "07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d"
    restored = read(original / "profile-restoration-result.json")
    assert restored["fourLogicalPathsRestored"] and restored["nativeDirectoryIdsPreserved"]
    assert read(original / "acl-restored-before-default-case.json")["aclRestoredExact"]
    assert read(ROOT / "fix" / "before-result.json")["expectedFailureObserved"]
    assert read(ROOT / "fix" / "test-result.json")["exitCode"] == 0
    build = read(ROOT / "fix" / "build-result.json")
    assert build["typecheckExitCode"] == build["buildExitCode"] == 0
    tests = (ROOT / "fix" / "tests.log").read_text(encoding="utf-8-sig")
    assert "100 passed (100)" in tests and "542 passed (542)" in tests
    assert read(ROOT / "fix" / "ci-handle.json")["source"] == "edead4c5c508afa8c2560abc6eaf00f0b2d7b243"
    for row in read(ROOT / "copies.json"):
        assert sha(ROOT / row["path"]) == row["sha256"]
        assert (ROOT / row["path"]).read_bytes() == Path(row["runtimePath"]).read_bytes()
    return {
        "status": "default-root-restart-real-backup-uninstall-retention-passed-recovery-incomplete",
        "nativeCandidate": contract["candidateSource"], "normalProductExits": 2,
        "realDefaultRootFilesRetained": 4, "configurationRestored": True,
        "recoverySelectionAndAutomaticRestartPassed": False, "customRootBindingPassed": False,
        "explicitPurgeTested": False, "migrationBlockedForUnverifiedRuntime": True,
        "emptyRootFixSource": "edead4c5c508afa8c2560abc6eaf00f0b2d7b243",
        "frontendTests": 542, "frontendFiles": 100, "typecheckAndBuildPassed": True,
        "fullCiAdmittedAtCheckpoint": False, "fixedNativeCandidateAdmitted": False,
        "fullStorageOrInstallerGatePassed": False, "releaseAccepted": False,
    }


if "--verify" in sys.argv:
    assert verify() == read(ROOT / "verification.json")
    for row in read(ROOT / "manifest.json")["files"]:
        assert sha(ROOT / row["path"]) == row["sha256"]
    print(json.dumps(read(ROOT / "verification.json")))
    sys.exit(0)

assert not (ROOT / "manifest.json").exists()
copies = []
names = [
    "human-window.json", "contract.json", "install.json", "recovery-prepared.json",
    "recovery-launch.json", "recovery-process-order.json", "recovery-census.original.json",
    "recovery-ui-incomplete.json", "acl-restored-before-default-case.json",
    "default-root-launch.json", "default-root-ui.json", "default-root-exit.json",
    "default-root-close-action.json", "default-restart-launch.json", "default-restart-ui.json",
    "default-restart-exit.json", "default-restart-close-action.json", "actual-backup-ui.json",
    "empty-root-initialize-ui.json", "migration-result-ui.json", "before-default-close.json",
    "actual-default-uninstall-retention.json", "before-original-restore.json",
    "profile-isolation-plan.json", "profile-isolation-result.json",
    "profile-restoration-result.json", "final-protection.json",
    "run-binding.ps1", "prepare-recovery.ps1", "launch-recovery.ps1", "launch-phase.ps1",
]
for source, prefix, selected in [
    (RUN, "original", names),
    (FIX, "fix", ["before-result.json", "before-test.log", "test-result.json", "tests.log",
                  "build-result.json", "typecheck.log", "build.log", "ci-handle.json", "ci-discovery.original.json"]),
]:
    for name in selected:
        target = ROOT / prefix / name
        data = (source / name).read_bytes()
        assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as stream:
            stream.write(data)
        copies.append({"path": prefix + "/" + name, "runtimePath": str(source / name),
                       "bytes": len(data), "sha256": sha(target)})
write(ROOT / "copies.json", copies)
write(ROOT / "verification.json", verify())
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
write(ROOT / "manifest.json", {"files": [{
    "path": p.relative_to(ROOT).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p),
} for p in sorted(ROOT.rglob("*")) if p.is_file()]})
print(json.dumps(read(ROOT / "verification.json")))
