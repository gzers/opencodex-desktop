# Windows 安装器构建缺陷修复（2026-11-23）

源码 `dffb3b1d64745f815476e77fc12103d3d34d90e4`（tree `9832faf40673aacc7b7818408b5a0baedacd98b8`），父提交是上一轮的 `0b78ea69d`，main 基线仍为 `b684e1788`。本轮只修安装器构建链路上两个真实缺陷，没有改动官方 2.99、运行包固定输入、日常配置或机器策略。

## 两个缺陷

`beforeBundleCommand` 写的是 `python ../../.github/scripts/write_windows_native_entry_024.py --binaries target/release --config tauri.conf.json --output resources/ocx-native-entry.json`。上一轮把它记为“工作目录与 target/release 期望不一致”，实际根因更简单：Tauri 执行钩子时的工作目录是 `apps/desktop`（`beforeBuildCommand` 的 `npm --prefix ui run build` 能成立就是这个原因），于是 `../../` 落到 `apps/.github`，脚本本身都找不到。三种写法都实测过：从 `apps/desktop/tauri` 跑 `../../` 得 `apps/.github`（不存在）；从 `apps/desktop` 跑 `../../` 得仓库根（存在）；从 `apps/desktop/tauri` 跑 `../../../` 得工作树根（不存在）。修法是让脚本按自身位置推导 `target/release`、`tauri.conf.json` 与 `resources/ocx-native-entry.json`，钩子本身只留一条不含路径的命令。

MSI 这边的失败被上一轮记成“WiX light 对 ICE 警告返回 204”。实际 `light` 返回 204 是因为有 `error LGHT0204 : ICE30`：`ocx.exe` 与 `ocxd.exe` 各自被两个组件安装。一份来自 Win64 模板的 `OCX024NativeShim` / `OCX024NativeConsole`，另一份来自 bundler 自动生成的 `Bin_ocx` / `Bin_ocxd`——这两个组件是 Cargo 里 `ocx`、`ocxd` 两个 `[[bin]]` 目标带来的，不是配置里的 `externalBin`（本轮用显式空 `externalBin` 的探针确认过，组件依旧出现）。修法是模板组件只保留固定 GUID 的安装身份注册值，文件交给 bundler 的组件安装；安装器校验脚本的成员名同步改成 `Bin_ocx` / `Bin_ocxd`。

## 结果

同一棵树上按标准布局重建（不设 `CARGO_TARGET_DIR`、不传 `--target`）：MSVC release raw0、432,022 ms；随后 `tauri bundle` raw0、118,798 ms，NSIS 与 MSI 都生成，`light` 不再有 ICE30。

程序与包：`opencodex-desktop.exe` 12,032,512 字节 `2eb01bc4…`；`ocx.exe` 251,392 字节 `3ddfadf7…`；`ocxd.exe` 743,936 字节 `04b9ad6d…`；MSI 300,384,256 字节 `6d4ca083…`；NSIS 299,489,765 字节 `94900554…`。注意这些摘要与上一轮的候选不同：本轮是从干净的标准布局重建的，二进制本身也因钩子与模板改动而重新链接。

只读 7-Zip 26.06 成员核验：MSI 13 项、NSIS 18 项嵌入成员与本次构建输入逐字节一致（`raw/msi-members.json`、`raw/nsis-members.json`）。MSI 的 `ocx.exe`/`ocxd.exe` 现在只由 `Bin_ocx`/`Bin_ocxd` 提供。

定向测试：`cargo test --locked --lib terminal_cli_windows` 在 MSVC 下 1 通过、4 按设计忽略（需隔离普通账户或真实 MSI）。安装器模板的纯 Python 合同测试 13 通过，4 项失败在 HEAD 原始文件上以同样方式失败（本机 CRLF 检出、测试仍断言 0.2.23 运行包名、以及该测试会读取未被复制到临时根目录的 `installer-hooks.nsh`），因此不构成本轮回归；同一组在 HEAD 与工作区的失败项完全一致。

## 未做

两个包都没有签名，本轮没有安装、没有执行安装/卸载语义验收，也没有跑 W2 双 SID 或发布。


