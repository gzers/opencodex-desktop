#!/bin/zsh
# 安装验证：启动 /Applications 下的 0.1.1 安装实例，确认可打开且版本正确，然后退出。
# 不启动代理、不改偏好，仅做只读启动核验。
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../.." && pwd)"
APP="/Applications/OpenCodeX Desktop.app"
OUT="${ROOT}/test/out"

if "$HERE/bin/lids" | grep -q "CGSSessionScreenIsLocked: 1"; then
  echo "BLOCKED: 锁屏"; exit 2
fi

echo "version: $(plutil -extract CFBundleShortVersionString raw "$APP/Contents/Info.plist")"
open -a "$APP"
for _ in {1..40}; do
  if "$HERE/bin/wins" "OpenCodeX" | grep -q onscreen=true; then break; fi
  sleep 0.5
done
sleep 2
"$HERE/fgh.sh" "$OUT/0.1.1-installed.png"
echo "quit installed app (SIGTERM)"
pkill -TERM -f 'OpenCodeX Desktop.app/Contents/MacOS/opencodex-desktop' || true
