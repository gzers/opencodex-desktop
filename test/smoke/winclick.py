#!/usr/bin/env python3
"""按 OCR 文字点击应用窗口内的目标。

用法：winclick.py <文字> [刷新截图 png] [--dbl] [--idx N]
"""

import argparse
import os
import sys

sys.path.insert(0, os.path.join(os.path.dirname(os.path.abspath(__file__)), "lib"))
import ocxui  # noqa: E402


def main():
    parser = argparse.ArgumentParser(description="按 OCR 文字点击应用窗口内的目标")
    parser.add_argument("label", help="要点击的文字")
    parser.add_argument("shot", nargs="?", default=os.path.join(ocxui.OUT, "_winclick.png"))
    parser.add_argument("--dbl", action="store_true", help="双击")
    parser.add_argument("--idx", type=int, default=0, help="存在多个匹配时取第几个（按上到下、左到右）")
    args = parser.parse_args()
    hit = ocxui.click_text(args.label, args.shot, idx=args.idx, double=args.dbl)
    if hit is None:
        print("NOT_FOUND", args.label)
        return 2
    print("%s %r" % ("DBL" if args.dbl else "CLICK", hit))
    return 0


if __name__ == "__main__":
    sys.exit(main())
