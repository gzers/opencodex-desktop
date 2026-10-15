import hashlib
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parent
CODE = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")

def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))

def digest(data):
    return hashlib.sha256(data).hexdigest()

manifest = read("manifest.json")
excluded = {"manifest.json", "verification.json"}
actual = {p.relative_to(ROOT).as_posix() for p in ROOT.rglob("*") if p.is_file()
          and p.relative_to(ROOT).as_posix() not in excluded}
assert actual == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and digest(data) == entry["sha256"], entry["path"]

ci = read("ci-readback.json")
assert len(ci["runs"]) == 3 and all(
    item["head"] == "02a0d4a8cbae8ef1450908d86b2c75fcad65f04f"
    and item["conclusion"] == "success" and item["attempt"] == 1 for item in ci["runs"])
phases = ["ordinary-first", "ordinary-restart", "ordinary-relocated",
          "administrator-relocated", "ordinary-unwritable-cancel",
          "ordinary-unwritable-select", "recovered", "negative-malformed", "negative-future"]
sid = "373954eb0fac4c7c274e7e4bacc1f7c5b170657bab283f87d84f5b35eb572310"
for phase in phases:
    launch, exit_record = read(phase + "-launch.json"), read(phase + "-exit.json")
    token = launch["applicationUser"]
    assert token["sidSha256"] == sid
    assert token["elevated"] == (phase == "administrator-relocated")
    assert token["elevationType"] == (2 if phase == "administrator-relocated" else 3)
    assert launch["exeSha256"] == "7383c275f58fed7e25678420a6d3f5c739f9b23d7373acc40fab1e7cd98db6d9"
    assert not launch["sandboxUsed"] and not launch["cdpUsed"]
    assert launch["pid"] == exit_record["pid"] and exit_record["exitObserved"]
    assert exit_record["exitCode"] == (1 if phase.startswith("negative-") else 0)
    assert not exit_record["forcedTermination"]

ordinary = read("restart-install-settings.json")
admin = read("administrator-relocated-install-settings.json")
assert ordinary["startupPointer"] == admin["startupPointer"]
assert "install-a" in json.loads(admin["startupPointer"])["anchor"]
assert all(not record["applicationMenuAttached"] for record in (ordinary, admin))
assert ordinary["applicationUser"]["elevated"] is False
assert admin["applicationUser"]["elevated"] is True
assert admin["screenshotSaved"]
assert any(item.get("name") == "运行环境与安装发现" for item in admin["ax"])
assert read("unwritable-cancel-postcheck.json") == {
    "nativeCount": 0, "roamingExists": False, "defaultDataExists": False, "pointerPresent": False}
assert read("unwritable-acl-restored.json")["originalDaclRestored"]
negative = read("negative-pointer-results.json")
assert negative["originalTestPointerRestored"] and len(negative["cases"]) == 2
assert all(case["exitCode"] == 1 and case["nativeCount"] == 0 and case["pointerUnchanged"]
           and not case["roamingCreated"] and not case["secondInstallDataCreated"]
           for case in negative["cases"])

manual_id, protection_id = "bk_20261015142354_9b295adb", "bk_20261015142444_4cd98b27"
manual = read(f"backup-originals/{manual_id}/backup-manifest.json")
protection = read(f"backup-originals/{protection_id}/backup-manifest.json")
for ident, metadata in ((manual_id, manual), (protection_id, protection)):
    payload = (ROOT / f"backup-originals/{ident}/preferences.json").read_bytes()
    assert digest(payload) == metadata["sha256"] and len(payload) == metadata["bytes"]
assert manual["management"]["pinned"]
assert protection["management"]["transaction_state"] == "committed"
assert protection["management"]["related_backup_id"] == manual_id
assert (ROOT / "after-restore-preferences.original.json").read_bytes() == (
    ROOT / f"backup-originals/{manual_id}/preferences.json").read_bytes()
assert read("before-restore-preferences.original.json")["appearance"]["theme"] == "dark"
assert read("after-restore-preferences.original.json")["appearance"]["theme"] == "light"
assert any(item.get("name") == "没有可清理的备份。" for item in read("retention-preview.json")["ax"])
assert any("未能确认面板运行状态" in item.get("name", "")
           for item in read("relocated-home-applied.json")["ax"])
assert read("recovered-launch.json")["observedMaxExeCountDuringRecovery"] == 2
assert read("fixture-correction.json")["performanceSampleInvalidated"]
assert read("seeded-invalid-preferences.original.json")["visual_effects"] == "medium"
private = read("private-picker-originals.json")
private_root = CODE / "test/runtime/foreground-02-current-20261015/private-picker-observations"
for entry in private["originalFiles"]:
    payload = (private_root / entry["name"]).read_bytes()
    assert len(payload) == entry["bytes"] and digest(payload) == entry["sha256"]

result = {"status": "verified-recorded-subsets-and-limitations", "files": len(actual),
          "privatePickerOriginalsVerifiedLocally": len(private["originalFiles"]),
          "sameSidOrdinaryAndAdministratorObserved": True,
          "recoveryObservedMaxExeCount": 2, "strictOneExeGatePassed": False,
          "externalHomeSettingsAcceptancePassed": False, "completeInstallerGatePassed": False,
          "fullVersionGatePassed": False}
(ROOT / "verification.json").write_text(json.dumps(result, indent=2) + "\n", encoding="utf-8")
print(json.dumps(result))
