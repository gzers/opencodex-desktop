"""Copy only the eleven previously archived sandbox JSON files after native exit."""
from datetime import datetime, timezone
import hashlib
import json
from pathlib import Path, PurePosixPath

root = Path(__file__).resolve().parent
prior = root.parent / "OCX-0110-20261015-WIN-FG-493-MANUAL-WINDOW"
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
post = read(root / "exit-postcheck.original.json")
assert post["nativeCount"] == post["nativeCountAfterProtectionChecks"] == 0
launch = read(prior / "administrator-launch.json")
snapshot = read(prior / "owned-state-followup/snapshot.json")
anchor = Path(launch["dataRoot"]).resolve()
assert anchor == Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\foreground-updates-493-20261015T1130\data").resolve()
assert snapshot["pid"] == post["formerPid"] == launch["pid"] == 78512
assert len(snapshot["files"]) == 11
destination = root / "reread"
assert not destination.exists()
entries = []
for row in snapshot["files"]:
    relative = PurePosixPath(row["path"])
    assert not relative.is_absolute() and ".." not in relative.parts and "\\" not in row["path"]
    assert relative.suffix == ".json"
    source = anchor / relative
    assert source.resolve().is_relative_to(anchor)
    assert not any(path.is_symlink() or path.is_junction() for path in [source, *source.parents])
    data = source.read_bytes()
    assert len(data) <= 5 * 1024 * 1024
    target = destination / relative
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open("xb") as stream:
        stream.write(data)
    digest = hashlib.sha256(data).hexdigest()
    entries.append({
        "path": row["path"], "bytes": len(data), "sha256": digest,
        "unchangedFromAdministratorFollowup": digest == row["sha256"] and len(data) == row["bytes"],
    })
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(),
    "source": post["source"], "formerPid": launch["pid"], "files": entries,
    "allElevenFilesUnchanged": all(row["unchangedFromAdministratorFollowup"] for row in entries),
    "sourceDataChanged": False, "webviewOrDailyDataCopied": False,
}
assert not (root / "state-reread.json").exists()
(root / "state-reread.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"files": len(entries), "allUnchanged": report["allElevenFilesUnchanged"]}))
