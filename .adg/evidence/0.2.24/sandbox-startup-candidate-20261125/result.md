# 0.2.24 TEAM81 隔离启动候选

2026-11-25。源码 main `a69ae55a3fe21b58f415da727b0d1bdff8f37568`。Windows x64 构建成功，Actions head SHA 与该源码一致。当前仅固定 Windows 定向制品，accepted:false。[源码](https://github.com/gzers/opencodex-desktop/commit/a69ae55a3fe21b58f415da727b0d1bdff8f37568) · [Actions](https://github.com/gzers/opencodex-desktop/actions/runs/45630909723)

TEAM80 的 MSI 行政解包已有完整载荷。旧候选在 `OPENCODEX_SANDBOX=1` 时跳过 `startup_bundle::start`，实际 111,841 ms 后仍无隔离 `program`，环境就绪及首次 Status 无法观测。修复移除该调用前的沙箱排除条件，沿用固定描述清单、已解析的安装根和数据根。沙箱使用自己的 `program` 与独立 HOME。[TEAM80 原始回执](https://github.com/gzers/opencodex-desktop/blob/06fcda42e67c5c698d2f7144961fa41fb4852709/.adg/evidence/0.2.24/windows-final/team80-20261125-r1/result.md)

源码格式、差异检查、严格全工作区 Clippy、31 项 UI 与 12 项 Rust 定向回归通过。Mac ARM64 用已验证的实际 2.99.0 随包载荷运行离线准备 worker，通过落盘、配置字节不变、安装租约释放和刷新回调断言，耗时 65.41 秒。该观察只覆盖离线准备，不计入服务的 Start／Status／Stop 预算。[实际 worker 输出摘录](offline-worker.log)

Windows 构建使用已离线验证的官方 2.99.0 运行资源。官方 tar 的 SHA-256 为 `b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c`，与源码固定值一致。[Windows 运行资源证据](https://github.com/gzers/opencodex-desktop/actions/runs/45630909723/artifacts/15032100055)

| Windows 文件 | SHA-256 | Minisign 更新签名 | Authenticode |
| --- | --- | --- | --- |
| OpenCodeX Desktop_0.2.24_x64_en-US.msi | `f8bbece2817e40ae3b5528d2d00caa055cebc158617cad13569da15de6fe4fe8` | 已独立验签 | `NotSigned` |
| OpenCodeX Desktop_0.2.24_x64-setup.exe | `4999df3bcfbc47294ff08a4093673e1ca11d313e15dc091d9a6d6114266b8db4` | 已独立验签 | `NotSigned` |

候选 ZIP 和内容报告 ZIP 都按 Actions 摘要重新复算。两个安装器的 SHA-256 与候选账本及 Windows 内容报告一致，更新签名以该源码的公钥独立验证。内容报告为离线安装包检查。真实 Windows 单用户 GUI、断网、缺包和首次 Status 尚待回交，只有明确空闲后才执行启停。TEAM80 的清理拒绝和原残留保留。[候选](https://github.com/gzers/opencodex-desktop/actions/runs/45630909723/artifacts/15030939242) · [内容报告](https://github.com/gzers/opencodex-desktop/actions/runs/45630909723/artifacts/15032642280) · [机器回执](verification.json) · [验签日志](verification.log) · [TEAM81](../../../../docs/03-开发实施/TEAM-0.2.24.md#team81-windows-隔离启动复验)
