"""冒烟工具共用：窗口边界、前台化截图、OCR、点击。

只依赖同目录编译出的 bin/{wins,ocr}、系统 screencapture、cliclick。
可用环境变量覆盖：OCX_APP（System Events 里的进程名）、OCX_WIN_NEEDLE（窗口属主名片段）、OCX_DPR（缩放）。
"""

import os
import re
import subprocess
import time

HERE = os.path.dirname(os.path.abspath(__file__))
SMOKE = os.path.dirname(HERE)
BIN = os.path.join(SMOKE, "bin")
OUT = os.path.join(os.path.dirname(SMOKE), "out")
APP_NAME = os.environ.get("OCX_APP", "opencodex-desktop")
WIN_NEEDLE = os.environ.get("OCX_WIN_NEEDLE", "OpenCodeX")
DPR = float(os.environ.get("OCX_DPR", "2"))


def window_line():
    """取目标应用第一个「在屏」窗口的信息行。"""
    out = subprocess.run([os.path.join(BIN, "wins"), WIN_NEEDLE], capture_output=True, text=True).stdout
    for line in out.splitlines():
        if "onscreen=true" in line:
            return line
    return ""


def screen_locked():
    """当前图形会话是否锁屏。锁屏时 WKWebView 不被合成，截图会发白或发虚，任何像素断言都不可信。"""
    out = subprocess.run([os.path.join(BIN, "lids")], capture_output=True, text=True).stdout
    return "CGSSessionScreenIsLocked: 1" in out


def window_origin():
    """窗口左上角在屏幕上的点坐标；取不到时回退到常见默认值。"""
    nums = dict(re.findall(r'"(X|Y|Width|Height)":\s*(-?\d+)', window_line()))
    return int(nums.get("X", 1010)), int(nums.get("Y", 264))


def focus_and_capture(out):
    """隐藏其它应用、前台化目标应用、把它的在屏窗口截到 out。"""
    if screen_locked():
        raise RuntimeError("当前图形会话处于锁屏状态：截图会发白/发虚，先解锁屏幕再取证。")
    os.makedirs(os.path.dirname(os.path.abspath(out)), exist_ok=True)
    hide = ('tell application "System Events" to set visible of every process '
            'whose visible is true and name is not "%s" to false' % APP_NAME)
    raise_app = ('tell application "System Events" to set frontmost of '
                 '(first process whose name is "%s") to true' % APP_NAME)
    for script in (hide, raise_app):
        subprocess.run(["osascript", "-e", script], capture_output=True)
    time.sleep(0.5)
    subprocess.run(["caffeinate", "-u", "-t", "1"], capture_output=True)
    time.sleep(1.0)
    full = os.path.join(OUT, "_full.png")
    subprocess.run(["screencapture", "-x", full], capture_output=True)
    nums = dict(re.findall(r'"(X|Y|Width|Height)":\s*(-?\d+)', window_line()))
    x = int(nums.get("X", 1010)); y = int(nums.get("Y", 264))
    w = int(nums.get("Width", 1180)); h = int(nums.get("Height", 760))
    from PIL import Image
    Image.open(full).crop((int(x * DPR), int(y * DPR), int((x + w) * DPR), int((y + h) * DPR))).save(out)
    return (x, y, w, h)


def ocr(path):
    """识别图片文字，返回 [(x, y, w, h, text)]（像素坐标，原点左上）。"""
    out = subprocess.run([os.path.join(BIN, "ocr"), path], capture_output=True, text=True).stdout
    rows = []
    for line in out.splitlines():
        m = re.match(r"\[(\d+),(\d+),(\d+),(\d+)\]\s(.*)$", line)
        if m:
            x, y, w, h = map(int, m.groups()[:4])
            rows.append((x, y, w, h, m.group(5).strip()))
    return rows


def screen_point(box):
    """截图像素坐标 -> 屏幕点坐标。"""
    x, y, w, h, _ = box
    ox, oy = window_origin()
    return ox + (x + w / 2) / DPR, oy + (y + h / 2) / DPR


def click_text(label, out=None, idx=0, double=False):
    """刷新截图 -> OCR 定位文本 -> 点击。返回命中的文本，未找到返回 None。"""
    out = out or os.path.join(OUT, "_click.png")
    focus_and_capture(out)
    time.sleep(0.2)
    hits = [r for r in ocr(out) if r[4] == label] or [r for r in ocr(out) if label in r[4]]
    hits.sort(key=lambda r: (r[1], r[0]))
    if not hits:
        return None
    box = hits[min(idx, len(hits) - 1)]
    px, py = screen_point(box)
    flag = "dc" if double else "c"
    subprocess.run(["cliclick", "%s:%d,%d" % (flag, round(px), round(py))])
    return box[4]
