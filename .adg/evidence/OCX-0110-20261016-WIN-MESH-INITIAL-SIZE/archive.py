"""Archive the completed code/CI/pixel phase; never run remote or native work."""
import hashlib
import json
import re
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
WORKTREE = Path("C:/Users/15119/.codex/worktrees/windows-mesh-initial-size/opencodex-desktop")
RUNTIME = WORKTREE / "test/runtime/mesh-initial-size-20261016T1857"
assert not (ROOT / "manifest.json").exists()


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(name, value):
    target = ROOT / name
    assert not target.exists(), name
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


copies = []
names = [
    "before-test.json", "before-test.original.log",
    "after-targeted-test.json", "after-targeted-test.original.log",
    "full-test.json", "full-test.original.log",
    "full-test-correct-cwd.json", "full-test-correct-cwd.original.log",
    "typecheck.original.log", "build.original.log", "local-results.json", "source-boundary.json",
    "ci-admission.json", "ci-terminal.original.json",
    "checkout-e7272dddb093dcffa0a27dd8dad5e7422288218a.original.json",
    "all-job-checkout-correction.json", "initial-admission-parser-failure.json",
    "remaining-log-format-check-failure.json", "prior-proxy-read-failure-20261016T110510Z.json",
    "pr-created.original.json", "pr-readback.original.json", "pr-create-request.json",
    "candidate-dispatch-request.json", "candidate-dispatch-attempt.json",
    "candidate-dispatch-readback.json", "candidate-after-dispatch.original.json",
    "candidate-progress-20261016T112537Z.original.json",
    "pixel-check.original.log", "pixel-check/binding.json", "pixel-check/browser.json",
    "pixel-check/browser-exit.json", "pixel-check/result.json",
]
for job in ["frontend", "backend", "backend-windows", "release-tools", "windows-smoke-evidence"]:
    names += [f"ci-38891188021-{job}.original.log", f"ci-38891188021-{job}.download.json"]
for name in [
    "compare-initial-size.mjs", "admit-ci.py", "dispatch-candidate-once.py",
    "download-ci-log.py", "download-remaining-ci-logs.py", "salvage-remaining-ci-logs.py",
]:
    names.append(name)
for name in names:
    data = (RUNTIME / name).read_bytes()
    text = data.decode("utf-8-sig")
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", text), name
    destination = "original/" + name
    target = ROOT / destination
    assert not target.exists()
    target.parent.mkdir(parents=True, exist_ok=True)
    target.write_bytes(data)
    copies.append({"path": destination, "runtimePath": str(RUNTIME / name),
                   "bytes": len(data), "sha256": sha(data)})
write("original-copies.json", copies)

pixel = json.loads((RUNTIME / "pixel-check/result.json").read_text())
images = {}
for row in pixel["rows"]:
    for variant in ["before", "after"]:
        for frame in row[variant]["frames"]:
            original = RUNTIME / "pixel-check" / frame["png"]
            data = original.read_bytes()
            digest = sha(data)
            assert digest == frame["pngSha256"] and len(data) == frame["pngBytes"]
            relative = f"pixels/{digest}.png"
            target = ROOT / relative
            if target.exists():
                assert target.read_bytes() == data
            else:
                target.parent.mkdir(parents=True, exist_ok=True)
                target.write_bytes(data)
            images[frame["png"]] = {"path": relative, "sha256": digest, "bytes": len(data)}
assert len(images) == 48
write("pixel-map.json", images)
source = subprocess.check_output(["git", "rev-parse", "HEAD"], cwd=WORKTREE).decode().strip()
assert source == "b4d487e4e04cd4c80fd5570663e1ed21dd483b19"
assert not subprocess.check_output(["git", "status", "--porcelain", "--untracked-files=no"], cwd=WORKTREE)
write("verification-scope.json", {
    "source": source, "parent": "823a8a0615153068594daef530c2e80f1d95fc29",
    "branch": "codex/0.1.10-windows-mesh-initial-size", "draftPr": 6,
    "ciRunId": 38891188021, "wholeCiSucceeded": True,
    "allFiveLogsActualCheckout": "e7272dddb093dcffa0a27dd8dad5e7422288218a",
    "sourceTree": "6d984c20825172036de79a62e47c3f7bf4f927c3",
    "localFullTestsPassed": 538, "ciFrontendPassed": 538, "ciMacPassed": 923, "ciWindowsPassed": 650,
    "componentPixelComparisons": 24, "allComponentPixelsAndPngBytesIdentical": True,
    "screenshotReferences": len(images), "uniquePngFiles": len({item["path"] for item in images.values()}),
    "initialContextBefore": [300, 150], "initialContextAfterIsFinalSize": True,
    "nativePerformanceMeasured": False, "nativeProductLaunched": False,
    "nativeGpuAllocationAttributed": False, "originalRssBudgetPassed": False,
    "oldPerformancePidsAssignedCurrentRoles": False, "queryReplayed": False, "dpiAcceptanceAdded": False,
    "candidateRunId": 38893610415, "candidateDispatchedOnce": True,
    "candidateArtifactsAdmitted": False, "foregroundReleased": True, "freshHumanWindowRequired": True,
    "historicalFailuresCleared": False, "taskClosed": False, "releaseAccepted": False,
    "correctionPrecedence": "original/all-job-checkout-correction.json supersedes earlier missing-checkout-marker claims",
})
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
print(json.dumps({"copies": len(copies), "pngReferences": len(images),
                  "uniquePngFiles": len({item["path"] for item in images.values()}),
                  "source": source, "nativePerformanceGatePassed": False}))
