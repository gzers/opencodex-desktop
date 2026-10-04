#!/bin/zsh
# 0.1.1 真实制品回归：验证原生「重载主界面」入口与启动构建来源。
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
APP="${ROOT}/apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app"
HOME_DIR="${ROOT}/test/runtime/home011"
OUT="${ROOT}/test/out"
mkdir -p "$HOME_DIR" "$OUT"
rm -rf "$HOME_DIR/Library/Application Support/com.gzers.opencodex.desktop"
LOG="$HOME_DIR/Library/Application Support/com.gzers.opencodex.desktop/logs/app.log"

if "$HERE/bin/lids" | grep -q "CGSSessionScreenIsLocked: 1"; then
  echo "BLOCKED: 锁屏"; exit 2
fi

HOME="$HOME_DIR" OPENCODEX_HOME="$HOME_DIR" "$APP/Contents/MacOS/opencodex-desktop" >"$HOME_DIR/app.log" 2>&1 &
APP_PID=$!
echo "pid   : $APP_PID"
for _ in {1..40}; do
  if "$HERE/bin/wins" "OpenCodeX" | grep -q onscreen=true; then break; fi
  sleep 0.5
done
sleep 2

echo "=== 启动事件（构建来源） ==="
grep 'startup version=' "$LOG" | tail -1

echo "=== 原生菜单项「重载主界面」存在性 ==="
osascript -e 'tell application "System Events" to tell process "opencodex-desktop" to get name of every menu item of menu 1 of menu bar item "视图" of menu bar 1' 2>&1

echo "=== 点击原生「重载主界面」 ==="
osascript -e 'tell application "System Events" to tell process "opencodex-desktop" to click menu item "重载主界面" of menu 1 of menu bar item "视图" of menu bar 1' 2>&1
sleep 2
echo "=== 重载事件日志 ==="
grep 'reload main' "$LOG" | tail -3

"$HERE/fgh.sh" "$OUT/0.1.1-after-reload.png"
echo "关闭应用……"
kill -TERM "$APP_PID" 2>/dev/null || true
