"""0.1.3 运行 UI 冒烟（U-03 官方远端查询 / U-05 网络检查连接）。

沙箱身份启动 0.1.3 制品，真实点击设置页入口并触发远端查询与连通性检查，
OCR 读取结果。只读操作，不改写偏好；非锁屏前置检查沿用 ocxui。
"""
import os
import subprocess
import sys
import time

HERE = os.path.dirname(os.path.abspath(__file__))
sys.path.insert(0, os.path.join(HERE, "lib"))
import ocxui  # noqa: E402

ROOT = os.path.abspath(os.path.join(HERE, "..", ".."))
APP = os.path.join(
    ROOT,
    "apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app",
)
SANDBOX = os.path.join(ROOT, "test/runtime/sandbox013ui")
OUT = os.path.join(ROOT, "test/out")


def ocr_texts(path):
    return ocxui.ocr(path)


def find(path, needles):
    rows = ocr_texts(path)
    return [r for r in rows if any(n in r[4] for n in needles)]


def main():
    if ocxui.screen_locked():
        print("BLOCKED: 锁屏")
        return 2
    os.makedirs(SANDBOX, exist_ok=True)
    os.makedirs(OUT, exist_ok=True)
    env = dict(os.environ, HOME=SANDBOX, OPENCODEX_HOME=SANDBOX,
               OPENCODEX_SANDBOX="1", OPENCODEX_SANDBOX_ROOT=SANDBOX)
    proc = subprocess.Popen(
        [os.path.join(APP, "Contents/MacOS/opencodex-desktop")],
        env=env, stdout=open(os.path.join(SANDBOX, "app.log"), "wb"),
        stderr=subprocess.STDOUT,
    )
    print("pid:", proc.pid)
    try:
        for _ in range(40):
            if "onscreen=true" in subprocess.run(
                [os.path.join(HERE, "bin", "wins"), "OpenCodeX"],
                capture_output=True, text=True).stdout:
                break
            time.sleep(0.5)
        time.sleep(2)

        overview = os.path.join(OUT, "013-01-overview.png")
        ocxui.focus_and_capture(overview)
        print("overview hit 设置:", bool(find(overview, ["设置"])))

        ocxui.click_text("设置", out=overview)
        time.sleep(1.5)
        settings = os.path.join(OUT, "013-02-settings.png")
        ocxui.focus_and_capture(settings)
        print("设置页命中标签:", [r[4] for r in ocr_texts(settings) if r[4] in
              ("通用", "版本升级", "关于", "安装配置")])

        ocxui.click_text("版本升级", out=settings)
        time.sleep(1.5)
        upgrade = os.path.join(OUT, "013-03-upgrade.png")
        ocxui.focus_and_capture(upgrade)
        print("升级页命中远端行:", [r[4] for r in find(upgrade, ["远端", "最新", "官方", "更新", "检查"])])

        ocxui.click_text("检查更新", out=upgrade)
        time.sleep(10)
        after = os.path.join(OUT, "013-04-after-check.png")
        ocxui.focus_and_capture(after)
        for box in find(after, ["远端", "最新", "有更新", "已最新", "不可用", "未比较", "官方"]):
            print("AFTER-CHECK:", box[4])
    finally:
        proc.terminate()
        time.sleep(1)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
