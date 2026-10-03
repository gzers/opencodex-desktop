#!/usr/bin/env python3
"""点击「设置」页的某个分区页签（先按 PageUp 回到顶部再找）。

用法：sect.py <页签名，如 "WebDAV 同步"> [输出 png]
"""

import os
import subprocess
import sys
import time

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib"))
import ocxui  # noqa: E402


def main():
    if len(sys.argv) < 2:
        print('用法: sect.py <页签名> [输出 png]')
        return 2
    target = sys.argv[1]
    out = sys.argv[2] if len(sys.argv) > 2 else os.path.join(ocxui.OUT, "_sect.png")
    for _ in range(10):
        ocxui.focus_and_capture(out)
        time.sleep(0.2)
        rows = [r for r in ocxui.ocr(out) if r[4].strip() == target and r[1] < 420]
        if rows:
            px, py = ocxui.screen_point(rows[0])
            subprocess.run(["cliclick", "c:%d,%d" % (round(px), round(py))])
            time.sleep(1.2)
            ocxui.focus_and_capture(out)
            print("clicked", target)
            return 0
        subprocess.run(["osascript", "-e",
                        'tell application "System Events" to key code 116'], capture_output=True)  # PageUp
        time.sleep(0.3)
    print("TAB_NOT_FOUND", target)
    return 2


if __name__ == "__main__":
    sys.exit(main())
