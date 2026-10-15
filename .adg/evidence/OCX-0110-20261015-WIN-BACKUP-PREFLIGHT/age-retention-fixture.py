"""Create explicit isolated manifest-age fixtures; not elapsed-time evidence or product UI input."""
import hashlib
import json
import os
from datetime import datetime, timedelta, timezone
from pathlib import Path

root = Path(__file__).resolve().parent
launch = json.loads((root / "ordinary-launch.json").read_text(encoding="utf-8-sig"))
anchor = Path(launch["dataRoot"]).resolve()
expected = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop\test\runtime\foreground-backup-ca26-20261015\data").resolve()
assert anchor == expected and launch["source"] == "ca26e771b0808b8b1b58c7ffadf5d1bad46189ff"
snapshot_root = root / "backup-state-fourteen-fresh"
snapshot = json.loads((snapshot_root / "snapshot.json").read_text(encoding="utf-8-sig"))
prefs = json.loads((anchor / "manager-state/preferences.json").read_text(encoding="utf-8-sig"))
assert prefs["updates"]["app_update_auto_check"] is False
policy = anchor / "manager-state/backup-cleanup-policy.json"
if policy.exists():
    assert json.loads(policy.read_text(encoding="utf-8-sig"))["mode"] == "manual"
output = root / "retention-age-fixture"
assert not output.exists()
records = []
for path in (anchor / "backups").rglob("backup-manifest.json"):
    assert not path.is_symlink() and path.resolve().is_relative_to(anchor)
    value = json.loads(path.read_text(encoding="utf-8-sig"))
    if value["action"] == "manual-preferences":
        records.append((path, value))
assert len(records) == 13
pinned = [(path, value) for path, value in records if value["management"]["pinned"]]
ordinary = sorted(((path, value) for path, value in records if not value["management"]["pinned"]),
                  key=lambda row: (row[1]["created_at"], row[1]["backup_id"]))
assert len(pinned) == 1 and len(ordinary) == 12
selected = [(pinned[0], 90), (ordinary[0], 61), (ordinary[1], 60)]
output.mkdir()
now = datetime.now(timezone.utc).replace(microsecond=0)
changes = []
for (path, value), days in selected:
    relative = path.relative_to(anchor)
    original = path.read_bytes()
    prior = snapshot_root / relative
    assert original == prior.read_bytes(), "Only the reviewed idle snapshot may be aged."
    payload = path.parent / "preferences.json"
    payload_bytes = payload.read_bytes()
    assert hashlib.sha256(payload_bytes).hexdigest() == value["sha256"]
    (output / (value["backup_id"] + ".before-manifest.json")).write_bytes(original)
    value["created_at"] = (now - timedelta(days=days)).isoformat().replace("+00:00", "Z")
    revised = (json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode()
    (output / (value["backup_id"] + ".after-manifest.json")).write_bytes(revised)
    temporary = path.with_name("age-fixture.pending")
    assert temporary.resolve().is_relative_to(anchor) and not temporary.exists()
    with temporary.open("xb") as stream:
        stream.write(revised)
    os.replace(temporary, path)
    assert payload.read_bytes() == payload_bytes
    changes.append({
        "backupId": value["backup_id"], "path": relative.as_posix(),
        "beforeSha256": hashlib.sha256(original).hexdigest(), "afterSha256": hashlib.sha256(revised).hexdigest(),
        "syntheticAgeDays": days, "createdAt": value["created_at"],
        "pinned": value["management"]["pinned"], "payloadSha256Unchanged": value["sha256"],
        "onlyCreatedAtChanged": True, "storedAtAndIdAndDirectoryUnchanged": True,
    })
report = {
    "atUtc": now.isoformat(), "dataRoot": str(anchor), "changes": changes,
    "ordinaryExpiredCandidateIds": [value["backup_id"] for _, value in ordinary[:2]],
    "pinnedExpiredMustRetainId": pinned[0][1]["backup_id"],
    "syntheticAgesAreNotRealPeriodEvidence": True, "systemClockChanged": False,
    "payloadsChanged": False, "filesDeleted": False, "uiInputInjected": False,
}
(output / "result.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
