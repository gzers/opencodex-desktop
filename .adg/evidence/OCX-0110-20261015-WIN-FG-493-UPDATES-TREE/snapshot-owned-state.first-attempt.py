"""Read/copy bounded files from this launch's own root, never WebView or daily user data."""
import hashlib
import json
import re
import sys
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
phase = sys.argv[1]
assert re.fullmatch(r"[a-z][a-z0-9-]{0,63}", phase)
launch = json.loads((root / "ordinary-launch.json").read_text(encoding="utf-8-sig"))
assert launch["source"] == "493da80675e6508e46e0fd83d86ee7216abf96fe"
anchor = Path(launch["dataRoot"]).resolve()
assert anchor == Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\foreground-updates-493-20261015T1130\data").resolve()
destination = root / ("owned-state-" + phase)
assert not destination.exists()
destination.mkdir()
entries = []
for part in ("manager-state", "cache/updates", "backups"):
    folder = anchor / part
    if not folder.exists():
        continue
    assert not folder.is_symlink() and folder.resolve().is_relative_to(anchor)
    for path in sorted(folder.rglob("*")):
        assert not path.is_symlink() and path.resolve().is_relative_to(anchor)
        if not path.is_file():
            continue
        assert path.stat().st_size <= 5 * 1024 * 1024
        data = path.read_bytes()
        relative = path.relative_to(anchor)
        target = destination / relative
        target.parent.mkdir(parents=True, exist_ok=True)
        with target.open("xb") as stream:
            stream.write(data)
        entries.append({"path": relative.as_posix(), "bytes": len(data),
                        "sha256": hashlib.sha256(data).hexdigest()})
report = {"atUtc": datetime.now(timezone.utc).isoformat(), "phase": phase,
          "source": launch["source"], "pid": launch["pid"], "dataRoot": str(anchor),
          "files": entries, "sourceDataChanged": False, "uiInputInjected": False,
          "webviewProfileCopied": False, "crossFileAtomicSnapshotClaimed": False}
(destination / "snapshot.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"phase": phase, "files": len(entries), "sourceDataChanged": False}))
