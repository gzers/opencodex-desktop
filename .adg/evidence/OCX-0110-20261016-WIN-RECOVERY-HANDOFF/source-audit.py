"""Bind the old overlap and new source without executing either application."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path("C:/Users/15119/.codex/worktrees/0110-windows-gates/opencodex-desktop")
DIAG = Path("C:/Users/15119/.codex/worktrees/windows-etw-query-threads/opencodex-desktop")
DOCS = ROOT.parents[2]
SOURCE = "823a8a0615153068594daef530c2e80f1d95fc29"
OLD = "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
assert not (ROOT / "manifest.json").exists()
assert not (ROOT / "source-audit.json").exists()


def git(repo, *args):
    return subprocess.run(["git", "-c", "core.longpaths=true", "-C", str(repo), *args],
                          check=True, capture_output=True).stdout


def sha(data):
    return hashlib.sha256(data).hexdigest()


def read(path):
    return json.loads(path.read_text(encoding="utf-8-sig"))


assert git(CODE, "rev-parse", "HEAD").decode().strip() == SOURCE
assert not git(CODE, "status", "--porcelain") and not git(DIAG, "status", "--porcelain")
changed = git(CODE, "diff-tree", "--no-commit-id", "--name-only", "-r", SOURCE).decode().splitlines()
assert len(changed) == 6
blobs = {}
for path in changed:
    data = git(CODE, "show", SOURCE + ":" + path)
    working = (CODE / path).read_bytes()
    assert data.replace(b"\r\n", b"\n") == working.replace(b"\r\n", b"\n")
    blobs[path] = {"gitBlobSha256": sha(data), "workingFileSha256": sha(working),
                   "identicalExceptCrLf": True}
path = "apps/desktop/tauri/src/infrastructure/windows_startup_root.rs"
before = git(CODE, "show", OLD + ":" + path)
previous_candidate = git(CODE, "show", "d94c215d869c0560717d79dc02881ce8858afe88:" + path)
assert before == previous_candidate and b"after_pick.restart()" in before
after = git(CODE, "show", SOURCE + ":" + path)
assert b"after_pick.restart()" not in after and b"prepare_current_restart()" in after
old_evidence = ROOT.parent / "OCX-0110-20261015-WIN-FG-02-ROOT"
recovered = read(old_evidence / "recovered-launch.json")
timeline = read(old_evidence / "recovery-process-timeline.original.json")
old_stop = read(old_evidence / "ordinary-unwritable-select-exit.json")
assert recovered["source"] == OLD and recovered["observedMaxExeCountDuringRecovery"] == 2
two = [row for row in timeline if len(row["pids"]) == 2]
assert len(two) == 1 and set(two[0]["pids"]) == {recovered["pid"], recovered["recoveryPid"]}
assert old_stop["pid"] == recovered["recoveryPid"] and old_stop["exitObserved"] and old_stop["exitCode"] == 0
references = {
    name: sha((old_evidence / name).read_bytes())
    for name in ("manifest.json", "recovered-launch.json", "recovery-process-timeline.original.json",
                 "ordinary-unwritable-select-exit.json")
}
local = read(ROOT / "controlled-final-result.original.json")
assert local["checks"] == 40 and local["status"] == "passed" and not local["cleanup"]
assert local["waiterSha256"] == blobs["apps/desktop/tauri/src/infrastructure/windows_restart.ps1"]["workingFileSha256"]
ui_identical = git(CODE, "rev-parse", SOURCE + ":apps/desktop/ui").decode().strip() == git(
    CODE, "rev-parse", "d94c215d869c0560717d79dc02881ce8858afe88:apps/desktop/ui").decode().strip()
assert ui_identical
shared = {}
for name in ("AGENTS.md", "README.md", "README.en.md", "LICENSE", ".gitignore"):
    values = [git(repo, "show", "HEAD:" + name) for repo in (CODE, DIAG, DOCS)]
    assert len({sha(value) for value in values}) == 1
    shared[name] = True
summary = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "source": SOURCE,
    "parentSource": "d94c215d869c0560717d79dc02881ce8858afe88",
    "oldFailureSource": OLD, "oldStartupBlobSha256": sha(before),
    "oldStartupBlobUnchangedThroughD94": True, "oldObservedMaximumExeCount": 2,
    "twoPidSnapshotAtUtc": two[0]["atUtc"], "exactOverlapDurationEstablished": False,
    "oldEvidenceReferences": references, "changedPaths": changed, "sourceBlobs": blobs,
    "bothCodeWorktreesClean": True, "sharedFilesIdentical": shared, "uiTreeUnchangedFromD94": True,
    "ordinaryStartupDoesNotPrepareRestart": True, "windowsRecoverySourceChanged": True,
    "nativeRecoveryExecutedOnNewSource": False, "oldFailureMarkedPassed": False,
    "newCandidateBuilt": False, "productStarted": False, "foregroundActivated": False,
}
(ROOT / "source-audit.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps({k: v for k, v in summary.items()
                  if k not in {"sourceBlobs", "oldEvidenceReferences", "changedPaths", "sharedFilesIdentical"}}))
