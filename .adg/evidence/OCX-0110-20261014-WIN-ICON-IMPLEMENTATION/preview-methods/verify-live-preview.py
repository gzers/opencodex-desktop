"""Verify preview frames, applied identities and protected file bytes."""
import hashlib
import json
import struct
from pathlib import Path
from PIL import Image

preview = Path(__file__).resolve().parent
repo = preview.parents[6]
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
sha = lambda path: hashlib.sha256(path.read_bytes()).hexdigest()
read = lambda path: json.loads(path.read_text(encoding="utf-8-sig"))
for name in ["app-85", "app-90"]:
    raw = (preview / (name + ".ico")).read_bytes()
    assert raw[:4] == bytes([0, 0, 1, 0])
    assert struct.unpack_from("<H", raw, 4)[0] == 7
    image = Image.open(preview / (name + ".ico"))
    assert image.ico.sizes() == {(value, value) for value in [16, 24, 32, 48, 64, 128, 256]}
    for size in image.ico.sizes():
        assert image.ico.getimage(size).convert("RGBA").tobytes() == Image.open(
            preview / name / (str(size[0]) + ".png")).convert("RGBA").tobytes()
window = read(preview / "live-preview/window-applied.json")
tray = read(preview / "live-preview/tray-applied-normalized-colour.json")
assert sha(preview / "app-90.ico") == window["imageSha256"]
assert window["status"] == "preview-active" and window["pid"] == tray["applicationPid"] == 10904
native_image = Image.open(preview / "tray-light/32.png").convert("RGBA")
assert hashlib.sha256(native_image.tobytes()).hexdigest() == tray["rawRgbaSha256"]
prior = read(repo / ".adg/evidence/OCX-0110-20261014-WIN-CAPTION-POINT/protected-after.json")
actual = {name: sha(code / "test/runtime/windows0110/data/manager-state" / name)
          for name in prior["protectedSandboxFiles"]}
assert actual == prior["protectedSandboxFiles"]
assert sha(Path(r"D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe")) == prior["dailyExecutableSha256"]
assert sha(Path(window["identity"]["executable"])) == prior["candidateExecutableSha256"]
report = {
    "previewAssetsVerified": True, "icoFramesVerified": 14, "liveRgbaMatchesCurrentFile": True,
    "dailyAndSandboxProtectedBytesUnchanged": True, "newNativeInstanceStarted": False,
    "productionBuildAccepted": False, "committed": False,
}
(preview / "live-preview/final-verification.json").write_text(
    json.dumps(report, indent=2) + "\n", encoding="utf-8")
print(json.dumps(report))
