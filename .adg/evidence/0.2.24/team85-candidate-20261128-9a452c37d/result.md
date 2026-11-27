# TEAM85 同源候选核验

2026-11-28。Windows 安装器可用于普通账户定向复验，整版 `accepted:false`。本轮暂停 W2／双 SID 和公开发布。[TEAM85 接收与复验合同](../../../../docs/03-开发实施/TEAM-0.2.24.md#team85-普通账户回执与复验合同)

## Windows x64

源码 `main@9a452c37d3005119042d81ee1ae8f62c4920c4d0`。双后端严格 Clippy、回归和其余三项必需检查已同源通过。[同源 CI 46040314845](https://github.com/gzers/opencodex-desktop/actions/runs/46040314845) · [构建 46049421165](https://github.com/gzers/opencodex-desktop/actions/runs/46049421165)

Actions ZIP SHA-256、安装器 SHA-256、Minisign 和两个安装器实际内嵌的官方 OpenCodex 2.99.0 运行归档均已核验。Windows CI 从 MSI 和 NSIS 实物提取内嵌文件并记录 Authenticode；本机将提取记录绑定到两份安装器的摘要，并用独立运行归档复核内嵌摘要。[Windows x64 安装器 15224497695](https://github.com/gzers/opencodex-desktop/actions/runs/46049421165/artifacts/15224497695) · [Windows 运行归档 15224462563](https://github.com/gzers/opencodex-desktop/actions/runs/46049421165/artifacts/15224462563) · [Windows 内嵌内容核验 15224582361](https://github.com/gzers/opencodex-desktop/actions/runs/46049421165/artifacts/15224582361)

| 安装器 | 文件 | SHA-256 | 更新签名（Minisign） | Authenticode |
| --- | --- | --- | --- | --- |
| MSI | `OpenCodeX Desktop_0.2.24_x64_en-US.msi` | `c4b530dc23397984923559018bb56b2a54ebf8736d34cef05a0dc73c7b1482f4` | 通过 | `NotSigned` |
| NSIS | `OpenCodeX Desktop_0.2.24_x64-setup.exe` | `3c28db9b660f22ca04ad63dcd546eba583631f81fa9634e95ee893263af54ff0` | 通过 | `NotSigned` |

独立运行归档 `runtime-bundle-0.2.24-win32-x64.tar.gz` 的 SHA-256 为 `14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7`，与两个安装器内嵌归档一致。

## macOS arm64

本机核对 DMG 中的单 .app、Applications 快捷方式、代码签名完整性、只读装载、更新签名与官方运行归档。应用使用临时签名（ad hoc），缺少 Developer ID 签名，公证装订未核验。[macOS arm64 候选 15223809005](https://github.com/gzers/opencodex-desktop/actions/runs/46049421165/artifacts/15223809005)

| 制品用途 | 文件 | SHA-256 |
| --- | --- | --- |
| dmg | `OpenCodeX Desktop_0.2.24_aarch64.dmg` | `544e152b9b637a804f4457ea32500eea7c72d0c4cccc4f23e5e9ab8f33930611` |
| updater | `OpenCodeX Desktop.app.tar.gz` | `c3d2b490318fa4ca03bdcfbda175071a8a6e403a91d377205ecbc8a721efaa87` |

## 原始证据

[同源 CI 回执](ci.latest.json) · [构建回执](run.json) · [Actions 制品索引](artifact-index.latest.json) · [Windows 本机复核](verification.json) · [MSI 内嵌检查](msi.json) · [NSIS 内嵌检查](nsis.json) · [Windows Authenticode](authenticode.json) · [macOS 本机复核](macos-verification.json)

[TEAM84 普通账户原始回执的接收表](../team85-windows-ordinary-intake-20261128/result.md)继续保留。候选制品核验完成，真实普通账户定向复验及本轮清理证据待回交。
