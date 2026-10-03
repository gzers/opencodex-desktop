#!/bin/zsh
# 前台化目标应用，并把它的在屏窗口截到指定文件（Retina 按 DPR 裁切）。
# 用法：fgh.sh <输出 png> [应用进程名，默认 opencodex-desktop]
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="${1:?用法: fgh.sh <输出 png> [应用进程名]}"
APP="${2:-opencodex-desktop}"
python3 - "$HERE" "$OUT" "$APP" <<'PY'
import sys
sys.path.insert(0, sys.argv[1] + "/lib")
import ocxui
ocxui.APP_NAME = sys.argv[3]
box = ocxui.focus_and_capture(sys.argv[2])
print("crop %s,%s %sx%s -> %s" % (box[0], box[1], box[2], box[3], sys.argv[2]))
PY
