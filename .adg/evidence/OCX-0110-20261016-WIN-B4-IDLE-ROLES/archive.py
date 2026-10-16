"""Archive measured counters and the incomplete lifecycle without running UI."""
import hashlib
import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent
PROJECT = ROOT.parents[2]
RUNTIME = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop/test/runtime/foreground-b4-pair-20261016T204338")
assert not (ROOT / "manifest.json").exists(), "Never rewrite frozen evidence"


def sha(data):
    return hashlib.sha256(data).hexdigest()


def write(name, value):
    path = ROOT / name
    assert not path.exists()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())


names = [
    "human-window.json", "fixture-ready.json", "cleanup-checkpoint.json",
    "run-observation.ps1", "derive-counter-roles.py", "counter-role-analysis.json",
    "observe-late-cleanup.ps1", "late-cleanup-observer-ready.json", "late-cleanup-exit.json",
    "baseline019-close-action.json", "baseline019-postspan-roles.original.json",
    "baseline019-postspan-roles-attempt.json", "candidate0110-close-window-expired.json",
    "candidate0110-sky-preclose-cached.json",
]
for version in ["baseline019", "candidate0110"]:
    names += [
        f"idle-{version}-launch.json", f"{version}-sky-initial.json",
        f"{version}-sky-settled-ready.json", f"{version}-settled-roles.original.json",
        f"{version}-settled-roles-attempt.json", f"{version}-exit-observer-ready.json",
        f"{version}-exit.json", f"{version}-collector-invocation.json",
        f"{version}-collector-exit.json", f"{version}-collector.stdout.original.log",
        f"{version}-collector.stderr.original.log", f"{version}/identity.json",
        f"{version}/terminal.json", f"{version}/progress.json", f"{version}/samples.original.jsonl",
    ]
for optional in ["fixture-restored.json", "cleanup-final-readback.json"]:
    if (RUNTIME / optional).exists():
        names.append(optional)
copies = []
for name in names:
    data = (RUNTIME / name).read_bytes()
    assert not re.search(r"S-1-\d+(?:-\d+){1,}", data.decode("utf-8-sig")), name
    relative = "original/" + name
    path = ROOT / relative
    assert not path.exists()
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_bytes(data)
    copies.append({"path": relative, "runtimePath": str(RUNTIME / name),
                   "bytes": len(data), "sha256": sha(data)})
write("original-copies.json", copies)
bindings = []
for directory in ["OCX-0110-20261016-WIN-B4-CANDIDATE", "OCX-0110-20261016-WIN-B4-FRONT-PREP"]:
    path = ROOT.parent / directory
    bindings.append({"evidenceDirectory": path.relative_to(PROJECT).as_posix(),
                     "manifestSha256": sha((path / "manifest.json").read_bytes()),
                     "verificationSha256": sha((path / "verification.json").read_bytes())})
write("source-bindings.json", bindings)
analysis = json.loads((RUNTIME / "counter-role-analysis.json").read_text())
write("scope.json", {
    "source": "b4d487e4e04cd4c80fd5570663e1ed21dd483b19",
    "humanWindow": ["2026-10-16T12:43:38+00:00", "2026-10-16T13:13:38+00:00"],
    "baselineSamples": 597, "candidateSamples": 597,
    "bothCompletedCounterSpansAtLeast600Seconds": True,
    "counterArithmeticAndRoleAccountingOnly": True,
    "cpuIncreasePercentagePoints": analysis["cpuIncreasePercentagePoints"],
    "meanRssIncreaseMiB": analysis["meanRssIncreaseMiB"],
    "gpuProcessRssDifferenceMiB": analysis["roleRssDifferenceMiB"]["gpu"],
    "candidateNormalExitWithinDeclaredObservationProven": False,
    "formalPairLifecycleAdmitted": False,
    "candidateOptionalPostspanRoleSnapshotTaken": False,
    "candidateCloseInputSentAfterExpiry": False,
    "candidateCloseInputSentBeforeExpiry": False,
    "exactFailedCloseAttemptTimestampPersisted": False,
    "originalAndLateCleanupObservationFailuresRetained": True,
    "fixtureRestoredAtArchive": (RUNTIME / "fixture-restored.json").exists(),
    "rawRssFormulaAndBudgetsChanged": False, "gpuSharedUsageAddedToRss": False,
    "historicalPidsAssignedRoles": False, "individualGlBuffersAttributed": False,
    "dpiAcceptancePerformed": False, "queryLibraryReplayed": False,
    "fullPerformanceGatePassed": False, "historicalFailuresCleared": False,
    "taskClosed": False, "releaseAccepted": False,
})
(ROOT / ".gitattributes").write_bytes(b"* -text\n")
print(json.dumps({"originalCopies": len(copies), "samplesPerVersion": 597,
                  "meanRssIncreaseMiB": analysis["meanRssIncreaseMiB"],
                  "formalPairLifecycleAdmitted": False}))
