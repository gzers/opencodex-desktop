"""Freeze every raw file including nested manifest.json; preserve the historical subset manifest."""
import hashlib
import json
import subprocess
import sys
import tempfile
from pathlib import Path

root = Path(__file__).resolve().parent
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
old = read(root / "manifest.json")
old_report = read(root / "archive-verification.json")
assert len(old) == old_report["manifestFiles"] == 209
covered = {row["path"] for row in old} | {"manifest.json", "archive-verification.json"}
nested = sorted(path for path in root.glob("owned-state-*/manager-state/**/manifest.json"))
assert len(nested) == 9
for path in nested:
    assert path.relative_to(root).as_posix() not in covered
    folder = next(parent for parent in path.parents if parent.name.startswith("owned-state-"))
    snapshot = read(folder / "snapshot.json")
    relative = path.relative_to(folder).as_posix()
    rows = [row for row in snapshot["files"] if row["path"] == relative]
    assert len(rows) == 1 and path.stat().st_size == rows[0]["bytes"] and sha(path) == rows[0]["sha256"]
# The original verifier intentionally freezes a historical file set. Recheck it
# in a byte-identical temporary replay, without editing it to accept additions.
runtime = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime")
assert runtime.is_dir() and not runtime.is_symlink()
with tempfile.TemporaryDirectory(prefix="archive-493-readonly-", dir=runtime) as temporary:
    replay = Path(temporary).resolve()
    assert replay.is_relative_to(runtime.resolve()) and replay.name.startswith("archive-493-readonly-")
    prior_paths = covered | {path.relative_to(root).as_posix() for path in nested}
    for name in prior_paths:
        path = root / name
        target = replay / name
        assert target.resolve().is_relative_to(replay)
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes(path.read_bytes())
    # Preserve the empty directory from the failed first read-only snapshot.
    (replay / "owned-state-before-manual-check").mkdir(exist_ok=True)
    original = subprocess.run([sys.executable, "-X", "utf8", str(replay / "verify-foreground.py")],
                              capture_output=True, text=True, check=True)
    assert json.loads(original.stdout) == old_report
manifest_path = root / "archive-complete-manifest.json"
report_path = root / "archive-complete-verification.json"
# Exclude only these two root output paths, never all matching basenames.
paths = sorted((path for path in root.rglob("*") if path.is_file() and path not in {manifest_path, report_path}),
               key=lambda path: path.relative_to(root).as_posix())
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size, "sha256": sha(path)}
           for path in paths]
if "--build" in sys.argv:
    assert not manifest_path.exists() and not report_path.exists()
    manifest_path.write_text(json.dumps(entries, indent=2) + "\n", encoding="utf-8")
else:
    assert read(manifest_path) == entries
assert {path.relative_to(root).as_posix() for path in nested} <= {row["path"] for row in entries}
report = {
    "archiveIntegrity": "passed", "scope": "Complete raw archive plus unchanged prior UI verification",
    "manifestFiles": len(entries), "source": old_report["source"],
    "historicalSubsetManifestFiles": 209, "historicalManifestAndReportUnchanged": True,
    "nestedMigrationManifestsIncludedDirectly": 9, "nestedMigrationManifestsAlsoVerifiedBySnapshots": True,
    "allExistingFilesExceptTwoExactRootOutputsIncluded": True,
    "nativeCountAtHistoricalCheckpoint": old_report["nativeCountAtCheckpoint"],
    "restartExitObservedAtHistoricalCheckpoint": False, "fullVersionGatePassed": False,
}
if "--build" in sys.argv:
    report_path.write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    assert read(report_path) == report
print(json.dumps(report))
