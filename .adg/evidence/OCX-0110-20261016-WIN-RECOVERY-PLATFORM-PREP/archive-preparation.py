"""Archive directory identities and guard results, never profile contents."""
import ast
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime")
INITIAL = RUNTIME / "recovery-platform-preparation-20261016T1606"
PREPARED = RUNTIME / "recovery-platform-preparation-v2-20261016T1612"
RAW = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-RAW-SCHEDULING-RUN"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(path, value):
    assert not path.exists(), str(path)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode("utf-8"))


assert not (ROOT / "manifest.json").exists(), "Frozen preparation cannot be extended"
copies = []
for folder, source, names in [
    ("first-attempt", INITIAL, ["profile-isolation-plan.json", "profile-isolation-failure.json", "partial-restoration-result.json"]),
    ("prepared", PREPARED, [
        "profile-isolation-plan.json", "profile-isolation-result.json", "preflight-ready-readback.json",
        "validate-guards.ps1", "guard-validation.original.json", "validate-guards-initial.original.ps1",
        "guard-validation-initial-failure.json", "cleanup-empty-guard-probe.ps1", "final-preparation-readback.json",
    ]),
]:
    for name in names:
        data = (source / name).read_bytes()
        destination = ROOT / folder / name
        destination.parent.mkdir(exist_ok=True)
        assert not destination.exists()
        destination.write_bytes(data)
        copies.append({"path": destination.relative_to(ROOT).as_posix(), "bytes": len(data), "sha256": sha(data)})

bindings = {}
source = "823a8a0615153068594daef530c2e80f1d95fc29"
baseline = "de6862d40389d527527b9db8ef52e186a58b625f"
for commit in [baseline, source]:
    for path in ["apps/desktop/ui/src/App.vue", "apps/desktop/ui/src/styles/base.css",
                 "apps/desktop/ui/src/features/runtime/motion/meshGlow.ts",
                 "apps/desktop/ui/src/features/runtime/components/RuntimeMotionMark.vue"]:
        data = subprocess.check_output(["git", "show", f"{commit}:{path}"], cwd=PROJECT)
        bindings[f"{commit}:{path}"] = sha(data)
app = subprocess.check_output(["git", "show", f"{source}:apps/desktop/ui/src/App.vue"], cwd=PROJECT).decode("utf-8")
assert '<component v-if="component" :is="component"' in app
assert "<KeepAlive" not in app and "<RouterView" not in app
write(ROOT / "source-inspection.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(), "baseline": baseline, "candidate": source,
    "sourceBlobSha256": bindings, "inspectionOnly": True,
    "observations": [
        "App.vue renders one selected route component; this source contradicts all routes being mounted at once.",
        "The old mesh host uses the motion-mark rectangle; current backdrop uses the main page and extends 80 CSS pixels above its clip.",
        "No native renderer/GPU attribution for the earlier idle-pair PIDs can be reconstructed from these source reads.",
    ],
    "allRoutesMountedAtOnceHypothesisSupported": False, "rssRootCauseEstablished": False,
    "formalPairRepeated": False, "productionCodeChanged": False, "performanceBudgetChanged": False,
})

frozen = PROJECT / ".adg/evidence/OCX-0110-20261016-WIN-823-FRONT-PREP"
write(ROOT / "preparation-binding.json", {
    "atUtc": datetime.now(timezone.utc).isoformat(), "configurationCleanupAuthorityCommit": "9e6292244e99750d66672ce0e3bcfee1f6849e62",
    "runtimePrepared": str(PREPARED), "runtimeFirstAttempt": str(INITIAL), "originalCopies": copies,
    "frozenRecoveryControllerDirectory": frozen.relative_to(PROJECT).as_posix(),
    "frozenControllerSha256": {name: sha((frozen / name).read_bytes()) for name in [
        "prepare-recovery.ps1", "launch-recovery.ps1", "restore-recovery.ps1", "run-binding.ps1", "contract.json",
    ]},
    "namespace": "actual ordinary calling Codex host; four logical paths resolve to two native directory IDs",
    "profileContentsArchived": False, "profileContentsDeleted": False, "productLaunched": False,
    "currentPreparedRootsAbsent": True, "fullProfileRestorationHelperExecuted": False,
    "firstPartialRestorationExecutedAndNativeIdsVerified": True, "foregroundAuthorizationGranted": False,
    "fullNativeRecoveryGatePassed": False,
})
for path in RAW.glob("*.py"):
    ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
write(RAW / "checkpoint-manifest.json", {
    "scope": "immutable file checkpoint; the RUN directory remains open for a later same-CI network recovery",
    "files": [{"path": p.relative_to(RAW).as_posix(), "bytes": p.stat().st_size, "sha256": sha(p.read_bytes())}
              for p in sorted(RAW.rglob("*")) if p.is_file() and p.name != "checkpoint-manifest.json"],
    "completeCiLogAdmission": False, "recordingStarted": False,
})
print(json.dumps({"copiedRuntimeOriginals": len(copies), "profileContentsCopied": False,
                  "sameCiNetworkCheckpointSaved": True, "productLaunched": False}))
