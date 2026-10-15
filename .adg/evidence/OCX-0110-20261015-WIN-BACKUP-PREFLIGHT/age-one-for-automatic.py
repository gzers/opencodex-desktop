"""Age one reviewed ordinary fixture that is still within the recent-ten count."""
import hashlib
import json
import os
from datetime import datetime, timedelta, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
launch = json.loads((root / "ordinary-launch.json").read_text(encoding="utf-8-sig"))
anchor = Path(launch["dataRoot"]).resolve()
assert anchor.name == "data" and anchor.parent.name == "foreground-backup-ca26-20261015"
snapshot_root = root / "backup-state-manual-cleanup-two"
snapshot = json.loads((snapshot_root / "snapshot.json").read_text(encoding="utf-8-sig"))
ordinary = sorted((value for value in snapshot["manifests"] if value["action"] == "manual-preferences"
                   and not value["management"]["pinned"]), key=lambda value: (value["created_at"], value["backup_id"]))
assert len(ordinary) == 10
chosen = ordinary[0]
matches = [path for path in (anchor / "backups").rglob("backup-manifest.json")
           if path.parent.name == chosen["backup_id"]]
assert len(matches) == 1
path = matches[0]
assert path.resolve().is_relative_to(anchor) and not path.is_symlink()
relative = path.relative_to(anchor)
original = path.read_bytes()
assert original == (snapshot_root / relative).read_bytes()
output = root / "automatic-age-fixture"
assert not output.exists()
output.mkdir()
(output / "before-manifest.json").write_bytes(original)
value = json.loads(original.decode("utf-8-sig"))
value["created_at"] = (datetime.now(timezone.utc) - timedelta(days=60)).replace(microsecond=0).isoformat().replace("+00:00", "Z")
revised = (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()
(output / "after-manifest.json").write_bytes(revised)
pending = path.with_name("automatic-age.pending")
assert not pending.exists() and pending.resolve().is_relative_to(anchor)
pending.write_bytes(revised)
os.replace(pending, path)
report = {
    "backupId": value["backup_id"], "path": relative.as_posix(), "syntheticAgeDays": 60,
    "beforeSha256": hashlib.sha256(original).hexdigest(), "afterSha256": hashlib.sha256(revised).hexdigest(),
    "onlyCreatedAtChanged": True, "wasWithinRecentTenBeforeNewBackup": True,
    "payloadUnchanged": hashlib.sha256((path.parent / "preferences.json").read_bytes()).hexdigest() == value["sha256"],
    "systemClockChanged": False, "realCycleEvidenceClaimed": False, "filesDeleted": False,
}
(output / "result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
