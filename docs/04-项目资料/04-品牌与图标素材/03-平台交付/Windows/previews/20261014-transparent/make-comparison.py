"""Create a clearly labelled static visual comparison, not a system screenshot."""
import json
from pathlib import Path
from PIL import Image, ImageDraw, ImageFont

root = Path(__file__).resolve().parent
code = Path(r"C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop")
regular = r"C:\Windows\Fonts\msyh.ttc"
bold = r"C:\Windows\Fonts\msyhbd.ttc"
font = lambda size, heavy=False: ImageFont.truetype(bold if heavy else regular, size)
canvas = Image.new("RGB", (1120, 770), "#eef1f6")
draw = ImageDraw.Draw(canvas)
draw.text((36, 25), "Windows 图标预览", font=font(28, True), fill="#15233c")
draw.text((36, 70), "静态尺寸对照 · 未提交 · 测试实例当前使用 90% 方案", font=font(16), fill="#52617a")
labels = ["当前：白色底板 / 主体75%", "方案A：透明 / 主体85%", "方案B：透明 / 主体90%"]
variants = [
    code / "apps/desktop/tauri/icons/256x256.png",
    root / "app-85/256.png",
    root / "app-90/256.png",
]
for index, (label, path) in enumerate(zip(labels, variants)):
    x = 36 + index * 360
    draw.text((x + 10, 111), label, font=font(17, True), fill="#24344f")
    for y, color, ink, heading in [(148, "#252d40", "#edf2fa", "深色背景"), (350, "#ffffff", "#27364e", "浅色背景")]:
        draw.rounded_rectangle((x, y, x + 330, y + 184), radius=14, fill=color)
        draw.text((x + 16, y + 13), heading, font=font(14), fill=ink)
        source = Image.open(path).convert("RGBA")
        for offset, size in [(22, 32), (92, 48), (189, 96)]:
            item = source.resize((size, size), Image.Resampling.LANCZOS)
            canvas.paste(item, (x + offset, y + 91 - size // 2), item)
            draw.text((x + offset + size // 2 - 11, y + 144), str(size) + "px", font=font(13), fill=ink)

draw.text((36, 560), "独立托盘：无底板、无渐变，按任务栏背景使用亮 / 暗前景", font=font(18, True), fill="#253550")
for index, (name, background, ink, caption) in enumerate([
    ("tray-light", "#252d40", "#ffffff", "深色任务栏 · 白色主体"),
    ("tray-dark", "#ffffff", "#27364e", "浅色任务栏 · 深色主体"),
]):
    x, y = 36 + index * 540, 599
    draw.rounded_rectangle((x, y, x + 510, y + 125), radius=14, fill=background)
    draw.text((x + 15, y + 11), caption, font=font(14), fill=ink)
    for offset, size in [(35, 16), (120, 20), (220, 24), (350, 32)]:
        source = Image.open(root / name / f"{size}.png").convert("RGBA")
        canvas.paste(source, (x + offset, y + 53), source)
        draw.text((x + offset - 3, y + 91), str(size) + "px", font=font(12), fill=ink)
draw.text((36, 743), "此图用于比较素材；实际应用标题栏 / 运行图标 / 托盘已在唯一隔离实例临时预览。", font=font(13), fill="#52617a")
canvas.save(root / "comparison.png")
report = json.loads((root / "generation-report.json").read_text(encoding="utf-8"))
for variant in report["variants"]:
    for frame in variant["frames"]:
        image = Image.open(root / variant["name"] / f"{frame['size']}.png").convert("RGBA")
        bbox = image.getchannel("A").point(lambda value: 255 if value > 16 else 0).getbbox()
        assert bbox == tuple(frame["bbox"])
        assert all(image.getpixel(point)[3] == 0 for point in
                   [(0, 0), (image.width - 1, 0), (0, image.height - 1), (image.width - 1, image.height - 1)])
        coverage = (bbox[2] - bbox[0]) / image.width
        assert 0.80 <= coverage <= 1.0
        if variant["monochrome"]:
            expected = tuple(bytes.fromhex(variant["monochrome"].removeprefix("#")))
            assert all(tuple(pixel[:3]) == expected for pixel in image.get_flattened_data() if pixel[3] > 16)
print(json.dumps({"comparison": str(root / "comparison.png"),
                  "rgbaFramesVerified": sum(len(item["frames"]) for item in report["variants"]),
                  "cornersTransparent": True, "trayMonochrome": True, "productionBuildClaimed": False}))
