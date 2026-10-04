"""驱动已运行的 0.1.3 应用：设置→版本升级→检查应用更新，OCR 读取结果。只读操作。"""
import os, sys, time
HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "lib"))
import ocxui  # noqa

OUT = os.path.join(os.path.dirname(HERE), "out")
os.makedirs(OUT, exist_ok=True)


def labels(path):
    return [r[4] for r in ocxui.ocr(path)]


def main():
    if ocxui.screen_locked():
        print("BLOCKED: locked")
        return 2
    p = os.path.join(OUT, "014-app-check-0.png")
    ocxui.focus_and_capture(p)
    print("initial labels:", [l for l in labels(p) if l in ("设置", "版本升级", "桌面管理器版本")][:5])
    hit = ocxui.click_text("检查应用更新", out=os.path.join(OUT, "014-click.png"))
    print("click 检查应用更新 ->", hit)
    time.sleep(12)
    q = os.path.join(OUT, "014-app-check-1.png")
    ocxui.focus_and_capture(q)
    for l in labels(q):
        if any(k in l for k in ("最新", "可用", "失败", "更新", "v0.1")):
            print("AFTER:", l)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
