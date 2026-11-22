# Windows 安装器原生入口来源文件

日期：2026-11-22。代码：main ec11a25e0。

## 问题

NSIS 模板、WiX 模板与发布校验脚本都要求 apps/desktop/tauri/resources/ocx-native-entry.json：

- windows/nsis023.nsi 用 File 打包它，并在安装时检查存在。
- windows/wix023.wxs 以固定组件 ID 嵌入它。
- .github/scripts/verify_windows_installer_024.py 把它列为必须核对的嵌入载荷。

仓库中没有任何分支包含该文件，也没有任何步骤生成它，只有 Windows 组件测试脚本会在临时工程里现造一份。正式出包会在打包阶段因为缺少该文件失败。

## 修法

新增 .github/scripts/write_windows_native_entry_024.py：

- 读取本次构建目录里的 ocx.exe 与 ocxd.exe，要求都是 PE 镜像。
- 写出 schemaVersion、package、version、entryKind 与两个文件的 SHA256。
- 字段与 modules/runtime/terminal_cli_windows.rs 的读取条件一一对应：schema 1、包名 opencodex-desktop、版本等于本包版本、entryKind 为 installer-native-exe、恰好两个文件且摘要相符。

生成时机放在 apps/desktop/tauri/tauri.windows.conf.json 的 beforeBundleCommand。它记录的是本次真正被嵌入的那两个二进制，不会因为提前单独编译而和归档内容不一致。该文件属于构建产物，已加入 apps/desktop/tauri/.gitignore。

## 验证边界

本机没有 Windows C 工具链，无法运行 MSVC 编译或真实 MSI/NSIS 打包。已完成的检查：脚本语法、合成二进制下的正常生成、缺少文件时的拒绝、以及产物字段与 Rust 端读取条件逐条比对。

真实 Windows 打包、签名与嵌入载荷核对仍须在 Windows 环境执行；本文件不声称安装器已通过。
