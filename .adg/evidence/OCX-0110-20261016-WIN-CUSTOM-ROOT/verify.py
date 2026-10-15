import hashlib
import json
from pathlib import Path
ROOT = Path(__file__).resolve().parent

def read(name):
    return json.loads((ROOT / name).read_text(encoding="utf-8-sig"))

manifest = read("manifest.json")
actual = {path.relative_to(ROOT).as_posix() for path in ROOT.rglob("*")
          if path.is_file() and path.relative_to(ROOT).as_posix() not in {"manifest.json", "verification.json"}}
assert actual == {entry["path"] for entry in manifest["files"]}
for entry in manifest["files"]:
    data = (ROOT / entry["path"]).read_bytes()
    assert len(data) == entry["bytes"] and hashlib.sha256(data).hexdigest() == entry["sha256"]
prepare = read("preparation.original.json")
assert len(prepare["unchangedStorageBlobs"]) == 5
assert all(item["identical"] for item in prepare["unchangedStorageBlobs"])
ordinary, admin = read("custom-ordinary-launch.json"), read("administrator-relocated-launch.json")
assert ordinary["applicationUser"]["sidSha256"] == admin["applicationUser"]["sidSha256"]
assert ordinary["applicationUser"]["elevationType"] == 3 and not ordinary["applicationUser"]["elevated"]
assert admin["applicationUser"]["elevationType"] == 2 and admin["applicationUser"]["elevated"]
def normalize(value):
    return value.removeprefix("\\\\?\\").lower()
for phase, launch in [("custom-ordinary-paths", ordinary),
                      ("administrator-relocated-install-settings", admin)]:
    view = read(phase + ".json")
    assert view["applicationUser"]["processId"] == launch["pid"]
    paths = [normalize(item["name"].removesuffix(" 复制")) for item in view["ax"]
             if item.get("type") == "ControlType.Button"]
    assert normalize(prepare["custom"]) in paths and normalize(prepare["externalHome"]) in paths
for phase in ["custom-ordinary", "administrator-relocated"]:
    record = read(phase + "-exit.json")
    assert record["exitObserved"] and record["exitCode"] == 0 and not record["forcedTermination"]
backup_paths = list((ROOT / "backup-originals").glob("*/backup-manifest.json"))
assert len(backup_paths) == 1
backup = json.loads(backup_paths[0].read_text())
assert normalize(backup["target_path"]).startswith(normalize(prepare["custom"]) + "\\")
payload = (backup_paths[0].parent / "preferences.json").read_bytes()
assert len(payload) == backup["bytes"] and hashlib.sha256(payload).hexdigest() == backup["sha256"]
restored = read("restoration.json")
assert restored["anchorOriginalBytesRestored"] and restored["originalAnchorLogUnchanged"]
assert restored["startupPointerUnchanged"] and restored["nativeCount"] == 0
assert (ROOT / "anchor-log-before.original.log").read_bytes() == (ROOT / "anchor-log-after.original.log").read_bytes()
assert (ROOT / "custom-log.original.log").read_text(encoding="utf-8-sig").count(
    "startup version=0.1.10 commit=02a0d4a8cbae8ef1450908d86b2c75fcad65f04f") == 2
summary = {"status": "verified-seeded-custom-binding-startup-subset", "files": len(actual),
           "actualExeSource": prepare["actualExeSource"], "sameSidDualTokens": True,
           "actualCustomBackupWrite": True, "anchorRestored": True,
           "uiPathSavingPassed": False, "actualOpenCodexChildHomeInheritancePassed": False,
           "fullStorageGatePassed": False, "fullVersionGatePassed": False}
(ROOT / "verification.json").write_text(json.dumps(summary, indent=2) + "\n", encoding="utf-8")
print(json.dumps(summary))
