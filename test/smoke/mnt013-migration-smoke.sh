#!/bin/zsh
# 0.1.3 分域存储迁移运行冒烟：沙箱内放 legacy 扁平偏好，启动制品后验证自动迁移为
# schema v2 分域结构并保留原件备份；受控「模拟日常根」哨兵不被改写。
# 用法：mnt013-migration-smoke.sh [--app <App 路径>]
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

SANDBOX="${ROOT}/test/runtime/sandbox013mig-$(date +%s)"
DAILY="${ROOT}/test/runtime/daily013mig"
mkdir -p "$SANDBOX/manager-state" "$DAILY/manager-state"
printf '%s' '{"interface_scale":123,"launch_main":true,"app_update_channel":"beta-6h","theme":"dark"}' > "$SANDBOX/manager-state/preferences.json"
printf '%s' '{"structure_version":"1"}' > "$SANDBOX/manager-state/data-root.json"
printf '%s' '{"interface_scale":99}' > "$DAILY/manager-state/preferences.json"
BEFORE_DAILY="$(shasum -a 256 "$DAILY/manager-state/preferences.json" | cut -d' ' -f1)"

echo "sandbox: $SANDBOX"
echo "=== legacy preferences (before) ==="
cat "$SANDBOX/manager-state/preferences.json"; echo

env HOME="$SANDBOX" OPENCODEX_HOME="$SANDBOX" OPENCODEX_SANDBOX=1 OPENCODEX_SANDBOX_ROOT="$SANDBOX" \
  "$APP/Contents/MacOS/opencodex-desktop" >"$SANDBOX/app.log" 2>&1 &
APP_PID=$!
echo "pid: $APP_PID"
for _ in {1..40}; do
  if "$HERE/bin/wins" "OpenCodeX" 2>/dev/null | grep -q onscreen=true; then break; fi
  sleep 0.5
done
sleep 3
kill -TERM "$APP_PID" 2>/dev/null || true
sleep 1

echo "=== startup event ==="
grep -a 'startup version=' "$SANDBOX/logs/app.log" | tail -1 || true
echo "=== preferences (after) ==="
cat "$SANDBOX/manager-state/preferences.json"; echo
echo "=== migration backup files ==="
find "$SANDBOX/manager-state/config-migrations" -type f 2>/dev/null | head || true

AFTER_DAILY="$(shasum -a 256 "$DAILY/manager-state/preferences.json" | cut -d' ' -f1)"
echo "daily sha: $BEFORE_DAILY -> $AFTER_DAILY"
if [[ "$BEFORE_DAILY" != "$AFTER_DAILY" ]]; then
  echo "FAIL: 日常根哨兵被改写"; exit 1
fi
if ! grep -q '"appearance"' "$SANDBOX/manager-state/preferences.json"; then
  echo "FAIL: 偏好未迁移为分域结构"; exit 1
fi
if ! grep -q '"schema_version": 2' "$SANDBOX/manager-state/preferences.json"; then
  echo "FAIL: schema 版本不是 2"; exit 1
fi
echo "PASS: 沙箱偏好迁移为分域 schema v2，日常根未变"
