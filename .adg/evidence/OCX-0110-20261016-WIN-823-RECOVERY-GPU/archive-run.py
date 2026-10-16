"""Archive the one real foreground attempt; exclude private ACL/SID originals."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-823-recovery-20261016T151826")
PRODUCT = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
DIAGNOSTIC = RUNTIME.parents[2]
PRODUCT_SHA = "823a8a0615153068594daef530c2e80f1d95fc29"
TOOL_SHA = "768e231811a2363b32f5fa67c8e74cbc31d751e1"
FILES = [
    "human-window.json", "execution-scope.json",
    "recovery-prepared.json", "recovery-launch.json",
    "recovery-census.original.json", "recovery-process-order.json",
    "collect-gpu.ps1", "gpu-snapshot.original.json",
    "observe-normal-exit.ps1", "normal-exit-observer-ready.json", "normal-exit.original.json",
    "restore-guard-current-pointer.original.json", "recovery-restored-guarded.json",
    "ui-backup-before-readback.json", "ui-install-root-readback.json",
    "ui-install-root-final-readback.json", "final-protection-readback.json",
]


def digest(data):
    return hashlib.sha256(data).hexdigest()


def write_json(path, value):
    assert not path.exists(), str(path)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))


assert not (ROOT / "manifest.json").exists(), "Never extend frozen evidence"
assert "recovery-private-originals.json" not in FILES
copied = []
for name in FILES:
    data = (RUNTIME / name).read_bytes()
    destination = ROOT / "run" / name
    destination.parent.mkdir(exist_ok=True)
    assert not destination.exists(), "Do not overwrite original evidence"
    destination.write_bytes(data)
    assert destination.read_bytes() == data
    copied.append({"path": f"run/{name}", "bytes": len(data), "sha256": digest(data)})

controller_root = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-823-FRONT-PREP"
controllers = {
    name: digest((controller_root / name).read_bytes())
    for name in ["prepare-recovery.ps1", "launch-recovery.ps1", "restore-recovery.ps1",
                 "run-binding.ps1", "retained-process-times.ps1", "contract.json"]
}
source_blobs = {}
for repo, sha, paths in [
    (PRODUCT, PRODUCT_SHA, [
        "apps/desktop/tauri/src/modules/data_root/bootstrap.rs",
        "apps/desktop/tauri/src/infrastructure/windows_startup_root.rs",
        "apps/desktop/tauri/src/lib.rs",
    ]),
    (DIAGNOSTIC, TOOL_SHA, ["test/smoke/windows-gpu-resource.ps1", "test/smoke/windows-process-token.ps1"]),
]:
    assert subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=repo).decode().strip() == sha
    assert not subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=repo)
    for path in paths:
        git_bytes = subprocess.check_output(["git", "show", f"{sha}:{path}"], cwd=repo)
        source_blobs[f"{sha}:{path}"] = digest(git_bytes)
write_json(ROOT / "code-binding.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(), "productSource": PRODUCT_SHA,
    "gpuToolSource": TOOL_SHA, "etwRemoteSourcePreserved": "54927d2cb184a38d5dd2e1f7c00ed4697b2e0a56",
    "productTreeClean": True, "gpuToolTreeClean": True, "sourceBlobs": source_blobs,
    "frozenControllerDirectory": controller_root.relative_to(PROJECT).as_posix(),
    "frozenControllerSha256": controllers,
    "gpuToolCi": {"runId": 38864269467, "status": "completed", "conclusion": "success",
                 "statusReadbackBeforeHumanWindow": True, "rawJobLogsFullyVerified": False},
    "runtime": str(RUNTIME), "originalCopies": copied,
    "privateAclAndRawSidOriginalExcluded": True, "sourceCopiedAsSecondAuthority": False,
    "productRebuilt": False, "formalIdlePairRepeated": False,
})
write_json(ROOT / "execution-notes.json", {
    "scope": "one real recovery attempt and one GPU snapshot; no second launch",
    "humanWindowLocal": ["2026-10-16T15:18:26+08:00", "2026-10-16T15:48:26+08:00"],
    "controllerLimitSeconds": 240, "recoveryCompleted": False,
    "observedUi": "main/settings UI used existing manager root; no recovery chooser was observed",
    "preconditionDefect": "preparation omitted the existing platform app-data root, which production prioritizes",
    "productRecoveryDefectEstablished": False,
    "normalCloseInput": "Sky Alt_L+F4 on the unique returned install-d window",
    "initialSkyInputFailure": "coordinate input geometry is unavailable; refreshed target and screenshot before further actions",
    "frozenRestoreRejection": "Different current choice preserved; review before pointer restoration",
    "frozenRestoreAclHadCompletedBeforePointerGuard": True,
    "restoreRejectionSource": "observed command transcript; not a recovered raw stdout file",
    "guardedRestoration": "native0; current pointer byte-equal to captured readback and anchor matches final UI; restore exact saved pointer; exact ACL/pointer readback",
    "liveUiScreenshotsObservedInTool": True, "screenshotsArchivedAsFiles": False,
    "actualUiAccessibilityReadbacksArchived": True,
    "automaticRetry": False, "forceTermination": False, "cdpUsed": False,
    "fullNativeRecoveryGatePassed": False, "fullPerformanceGatePassed": False,
})
print(json.dumps({"copiedOriginals": len(copied), "privateOriginalsPublished": False}))
