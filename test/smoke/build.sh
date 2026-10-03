#!/bin/zsh
# 编译本机冒烟工具到 test/smoke/bin/。需要 Xcode 命令行工具（swiftc）。
set -e
HERE="$(cd "$(dirname "$0")" && pwd)"
BIN="$HERE/bin"
mkdir -p "$BIN"
for tool in ocr wins lids capwin; do
  echo "building $tool ..."
  swiftc -O "$HERE/lib/$tool.swift" -o "$BIN/$tool"
done
chmod +x "$BIN"/*
echo "done -> $BIN"
