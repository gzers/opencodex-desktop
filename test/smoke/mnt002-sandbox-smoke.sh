#!/bin/zsh
# 0.1.2 沙箱隔离冒烟：在独立沙箱身份启动制品，验证启动构建来源与日常根基线未被改写。
# 用法：mnt002-sandbox-smoke.sh [--app <App 路径>]
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
APP="${ROOT}/apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app"
while [[ $# -gt 0 ]]; do
  case "$1" in
    --app) APP="$2"; shift 2 ;;
    *) echo "未知参数: $1"; exit 2 ;;
  esac
done

# 沙箱与受控的「模拟日常根」；日常根只放哨兵，整轮不被写入。
SANDBOX="${ROOT}/test/runtime/sandbox002"
DAILY="${ROOT}/test/runtime/daily002"
OUT="${ROOT}/test/out"
mkdir -p "$SANDBOX" "$DAILY/manager-state" "$OUT"
printf '%s' '{"interface_scale":123,"app_update_channel":"beta-6h"}' > "$DAILY/manager-state/preferences.json"
printf '%s' '{"structure_version":"1"}' > "$DAILY/manager-state/data-root.json"
BEFORE_PREFS="$(shasum -a 256 "$DAILY/manager-state/preferences.json" | cut -d' ' -f1)"
BEFORE_MTIME="$(stat -f %m "$DAILY/manager-state/preferences.json")"

if "$HERE/bin/lids" | grep -q "CGSSessionScreenIsLocked: 1"; then
  echo "BLOCKED: 锁屏"; exit 2
fi

env HOME="$SANDBOX" OPENCODEX_HOME="$SANDBOX" OPENCODEX_SANDBOX=1 OPENCODEX_SANDBOX_ROOT="$SANDBOX" \
  "$APP/Contents/MacOS/opencodex-desktop" >"$SANDBOX/app.log" 2>&1 &
APP_PID=$!
echo "pid   : $APP_PID"
for _ in {1..40}; do
  if "$HERE/bin/wins" "OpenCodeX" | grep -q onscreen=true; then break; fi
  sleep 0.5
done
sleep 2

echo "=== 启动事件（构建来源） ==="
grep 'startup version=' "$SANDBOX/Library/Application Support/com.gzers.opencodex.desktop/logs/app.log" | tail -1 || true
echo "=== 沙箱数据根 ==="
ls "$SANDBOX/Library/Application Support/com.gzers.opencodex.desktop" 2>/dev/null || true

kill -TERM "$APP_PID" 2>/dev/null || true
sleep 1

AFTER_PREFS="$(shasum -a 256 "$DAILY/manager-state/preferences.json" | cut -d' ' -f1)"
AFTER_MTIME="$(stat -f %m "$DAILY/manager-state/preferences.json")"
echo "daily preferences sha: $BEFORE_PREFS -> $AFTER_PREFS"
echo "daily preferences mtime: $BEFORE_MTIME -> $AFTER_MTIME"
if [[ "$BEFORE_PREFS" != "$AFTER_PREFS" || "$BEFORE_MTIME" != "$AFTER_MTIME" ]]; then
  echo "FAIL: 日常根哨兵被改写"; exit 1
fi
echo "PASS: 日常根未变"

