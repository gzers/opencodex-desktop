#!/bin/zsh
# 构建 WKWebView 几何探针（真实渲染层取证；Blink 探针查不出的 WebKit 差异用这里复现）。
# 产物落在 test/smoke/bin/（已 gitignore）。
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../.." && pwd)"
OUT="${ROOT}/test/smoke/bin"
mkdir -p "$OUT"
swiftc -O "$HERE/main.swift" -o "$OUT/wkprobe"
swiftc -O "$HERE/main2.swift" -o "$OUT/wkprobe2"
echo "built: $OUT/wkprobe  $OUT/wkprobe2"
