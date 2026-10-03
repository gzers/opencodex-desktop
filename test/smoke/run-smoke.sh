#!/bin/zsh
# 用隔离 HOME 启动构建好的 .app，并抓一张概览截图。
# 用法：run-smoke.sh [--app <App 路径>] [--home <隔离 HOME>] [--out <输出 png>]
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
APP="${ROOT}/apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app"
HOME_DIR="${ROOT}/test/runtime/home"
OUT="${ROOT}/test/out/overview.png"

while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) APP="$2"; shift 2 ;;
    --home) HOME_DIR="$2"; shift 2 ;;
    --out) OUT="$2"; shift 2 ;;
    *) echo "未知参数: $1"; exit 2 ;;
  esac
done

mkdir -p "$HOME_DIR" "$(dirname "$OUT")"
echo "app   : $APP"
echo "home  : $HOME_DIR"
echo "out   : $OUT"

if "$HERE/bin/lids" | grep -q "CGSSessionScreenIsLocked: 1"; then
  echo "BLOCKED: 当前会话已锁屏；锁屏时截图发白/发虚，先解锁屏幕再取证。" >&2
  exit 2
fi

HOME="$HOME_DIR" OPENCODEX_HOME="$HOME_DIR" "$APP/Contents/MacOS/opencodex-desktop" >"$HOME_DIR/app.log" 2>&1 &
APP_PID=$!
echo "pid   : $APP_PID"

for _ in {1..30}; do
  if "$HERE/bin/wins" "OpenCodeX" | grep -q onscreen=true; then break; fi
  sleep 0.5
done
sleep 2
"$HERE/fgh.sh" "$OUT"
echo "关闭应用（SIGTERM）……"
kill -TERM "$APP_PID" 2>/dev/null || true
