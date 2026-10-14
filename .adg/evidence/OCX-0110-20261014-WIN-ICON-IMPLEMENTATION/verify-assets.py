"""Verify approved Windows exports, background regression, and platform separation."""
import hashlib
import json
import subprocess
from datetime import datetime, timezone
from pathlib import Path
from PIL import Image

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
icons = code / "apps/desktop/tauri/icons"
base = "d3b83b3822bf10a511556d80dc851c2ccfb17cf5"
sha = lambda raw: hashlib.sha256(raw).hexdigest()
old_config = json.loads(subprocess.run(
    ["git", "show", f"{base}:apps/desktop/tauri/tauri.conf.json"], cwd=code,
    check=True, capture_output=True).stdout)
config = json.loads((code / "apps/desktop/tauri/tauri.windows.conf.json").read_text())
assert config["app"]["trayIcon"]["iconAsTemplate"] is False
assert all(path.startswith("icons/windows/") for path in config["bundle"]["icon"])
assert config["app"]["trayIcon"]["iconPath"].startswith("icons/windows/")
assert old_config == json.loads((code / "apps/desktop/tauri/tauri.conf.json").read_text())
assert old_config["app"]["trayIcon"]["iconAsTemplate"] is True
assert "icons/icon.icns" in old_config["bundle"]["icon"]


def check_app(frame):
    image = frame.convert("RGBA")
    size = image.width
    mask = image.getchannel("A").point(lambda value: 255 if value > 16 else 0)
    bounds = mask.getbbox()
    errors = []
    if any(pixel[3] > 16 and min(pixel[:3]) >= 240 for pixel in image.get_flattened_data()):
        errors.append("opaque-white-backing-or-artwork")
    if not bounds or min(bounds[:2]) <= 0 or max(bounds[2:]) >= size:
        errors.append("no-transparent-safety-margin")
    if bounds and min(bounds[2] - bounds[0], bounds[3] - bounds[1]) * 100 // size < 85:
        errors.append("visible-subject-below-85-percent")
    return errors


old_image = Image.open(icons / "icon.ico")
red = {str(size[0]): check_app(old_image.ico.getimage(size)) for size in sorted(old_image.ico.sizes())}
assert all(errors for errors in red.values())
new_image = Image.open(icons / "windows/icon.ico")
assert new_image.ico.sizes() == {(value, value) for value in [16, 24, 32, 48, 64, 128, 256]}
green = {}
for size in sorted(new_image.ico.sizes()):
    frame = new_image.ico.getimage(size)
    errors = check_app(frame)
    assert not errors, (size, errors)
    source = Image.open(icons / "windows" / f"{size[0]}x{size[0]}.png")
    assert frame.convert("RGBA").tobytes() == source.convert("RGBA").tobytes()
    green[str(size[0])] = "passed"
tray = []
for tone, color in [("light", (255, 255, 255)), ("dark", (26, 41, 72))]:
    for size in [16, 20, 24, 32, 40, 48, 64]:
        path = icons / "windows" / f"tray-{tone}-{size}.png"
        image = Image.open(path).convert("RGBA")
        assert image.size == (size, size)
        bounds = image.getchannel("A").point(lambda value: 255 if value > 16 else 0).getbbox()
        assert bounds and min(bounds[:2]) > 0 and max(bounds[2:]) < size
        assert min(bounds[2] - bounds[0], bounds[3] - bounds[1]) * 100 // size >= 85
        assert all(tuple(pixel[:3]) == color for pixel in image.get_flattened_data() if pixel[3] > 16)
        assert any(image.getpixel((x, y))[3] <= 16
                   for y in range(size // 4 + 1, size * 3 // 4)
                   for x in range(size // 4 + 1, size * 3 // 4))
        tray.append({"path": path.relative_to(code).as_posix(), "size": size, "sha256": sha(path.read_bytes())})
unchanged = {}
for relative in ["icons/icon.icns", "icons/32x32.png", "icons/256x256.png",
                 "icons/tray-16.png", "icons/tray-32.png"]:
    original = subprocess.run(["git", "show", f"{base}:apps/desktop/tauri/{relative}"],
                              cwd=code, check=True, capture_output=True).stdout
    current = (code / "apps/desktop/tauri" / relative).read_bytes()
    assert original == current
    unchanged[relative] = sha(current)
report = {
    "atUtc": datetime.now(timezone.utc).isoformat(), "baseSource": base,
    "userApprovedSubjectCoverage": 0.90, "oldWindowsIconRegression": red,
    "newWindowsIcoFrames": green, "trayFrames": tray, "unchangedMacAndDefaultResources": unchanged,
    "defaultConfigurationSemanticallyUnchanged": True,
    "localRustCompilerAvailable": False, "rustCompilationAcceptance": False,
    "nativeImagePreviewApproved": True, "finalSourceNativeAccepted": False,
    "fullVersionGatePromoted": False,
}
(root / "local-assets-red-green.json").write_text(
    json.dumps(report, ensure_ascii=False, indent=2) + "\n", encoding="utf-8")
print(json.dumps({"oldFramesFailNewBackgroundContract": len(red), "newIcoFramesPassed": len(green),
                  "trayFramesPassed": len(tray), "macAndDefaultResourcesUnchanged": True}))
