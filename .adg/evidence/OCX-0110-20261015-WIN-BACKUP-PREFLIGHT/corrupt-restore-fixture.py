"""One reversible same-size integrity fixture within the new isolated backup root."""
import hashlib
import json
import sys
from datetime import datetime, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
launch = json.loads((root / "ordinary-launch.json").read_text(encoding="utf-8-sig"))
anchor = Path(launch["dataRoot"]).resolve()
assert anchor.parent.name == "foreground-backup-ca26-20261015"
automatic = json.loads((root / "automatic-retention-analysis.json").read_text())
assert len(automatic["newIds"]) == 1
backup_id = automatic["newIds"][0]
assert backup_id == "bk_20261015015618_1393e76a"
matches = list((anchor / "backups").rglob(backup_id + "/preferences.json"))
assert len(matches) == 1
target = matches[0]
assert target.resolve().is_relative_to(anchor) and not target.is_symlink()
manifest = json.loads((target.parent / "backup-manifest.json").read_text(encoding="utf-8-sig"))
fixture_root = root / "restore-integrity-fixture"
mode = sys.argv[1]
if mode == "corrupt":
    if fixture_root.exists():
        assert not any(fixture_root.iterdir()) and (root / "integrity-fixture-preparation-error.json").exists()
    else:
        fixture_root.mkdir()
    original = target.read_bytes()
    assert hashlib.sha256(original).hexdigest() == manifest["sha256"]
    newline = original.index(b"\n")
    revised = original[:newline] + b" " + original[newline + 1:]
    assert json.loads(original) == json.loads(revised) and len(original) == len(revised)
    (fixture_root / "payload-original.json").write_bytes(original)
    (fixture_root / "payload-corrupted-same-size.json").write_bytes(revised)
    target.write_bytes(revised)
    report = {
        "atUtc": datetime.now(timezone.utc).isoformat(), "mode": mode, "backupId": backup_id,
        "payloadRelativePath": target.relative_to(anchor).as_posix(),
        "originalSha256": manifest["sha256"], "corruptedSha256": hashlib.sha256(revised).hexdigest(),
        "bytes": len(original), "semanticJsonUnchanged": True, "manifestChanged": False,
        "activePreferencesChanged": False, "isolatedFixtureOnly": True, "systemClockChanged": False,
    }
    (fixture_root / "corrupt-result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
elif mode == "repair":
    assert fixture_root.exists() and not (fixture_root / "repair-result.json").exists()
    original = (fixture_root / "payload-original.json").read_bytes()
    corrupted = (fixture_root / "payload-corrupted-same-size.json").read_bytes()
    assert target.read_bytes() == corrupted and hashlib.sha256(original).hexdigest() == manifest["sha256"]
    target.write_bytes(original)
    report = {
        "atUtc": datetime.now(timezone.utc).isoformat(), "mode": mode, "backupId": backup_id,
        "repairedSha256": hashlib.sha256(target.read_bytes()).hexdigest(), "originalBytesRestored": True,
        "manifestChanged": False, "activePreferencesChanged": False, "isolatedFixtureOnly": True,
    }
    (fixture_root / "repair-result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
else:
    raise ValueError("Only the reviewed corrupt/repair operations are allowed.")
print(json.dumps(report))
