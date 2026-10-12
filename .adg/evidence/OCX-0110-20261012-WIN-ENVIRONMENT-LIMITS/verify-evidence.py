import hashlib
import json
import sys
from pathlib import Path

root = Path(__file__).resolve().parent
read = lambda name: json.loads((root / name).read_text(encoding="utf-8-sig"))
environment = read("environment-query.json")
settings = read("settings-observation.json")
assert environment["sourceCommit"] == "afff490a430a81f8c233d44928aa3681a23e91e0"
assert environment["applicationToken"] == {"elevated": False, "elevationType": 3}
assert environment["hyperVInventory"]["status"] == "access_denied"
assert not environment["settingsChanged"] and not environment["authenticationAutomated"]
assert settings["scaleControlDisabled"] and settings["settingsClosed"]
assert settings["observedText"] == "无法从远程会话更改显示器设置。"
assert not settings["settingsChanged"] and not settings["rawCaptureIncluded"]
assert read("crop-method.json")["cropBox"] == [896, 49, 2048, 679]
assert not any("private" in path.name for path in root.rglob("*") if path.is_file())
assert (root / "rdp-display-controls-redacted.png").read_bytes().startswith(b"\x89PNG\r\n\x1a\n")
excluded = {"manifest.json", "archive-verification.json"}
files = sorted(path for path in root.rglob("*") if path.is_file() and path.name not in excluded)
entries = [{"path": path.relative_to(root).as_posix(), "bytes": path.stat().st_size,
            "sha256": hashlib.sha256(path.read_bytes()).hexdigest()} for path in files]
def write(name, value):
    (root / name).write_bytes((json.dumps(value, ensure_ascii=False, indent=2) + "\n").encode())
if sys.argv[1:] == ["--build"]:
    write("manifest.json", {"hashMode": "raw", "entries": entries})
else:
    assert not sys.argv[1:]
assert read("manifest.json")["entries"] == entries
write("archive-verification.json", {
    "result": "passed", "scope": "recorded environmental limitations and archive integrity only",
    "otherPhysicalDpiAccepted": False, "virtualMachineAbsentProved": False,
    "safeInstallerEnvironmentAvailableProved": False, "filesVerified": len(entries),
})
print(json.dumps({"archiveIntegrity": "passed", "files": len(entries)}))
