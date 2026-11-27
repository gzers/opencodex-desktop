# Windows 0.2.24 CI 已完成作业的复核

源码 `7889ecc3586eb811cffc4e80ec1d6f97998caaae`，Actions `45942760452`，`accepted:false`。

| 作业 | 结果 | 证据范围 |
| --- | --- | --- |
| frontend `140320539065` | pass | `ui-dialog-glass-v2.test.ts` 28 项通过，用时 74,348 ms。全作业 142 个测试文件、913 项通过。此前 `45917490865` 的 Chromium 关闭超时仍保留，复跑通过未定位该超时原因。 |
| windows-smoke-evidence `140320539077` | pass | 复算制品 `15173700467` ZIP 的 SHA-256，与 Actions 一致。临时普通 Profile 的 Known Folder 读回、进程代际变化拒绝、原先的子进程已退出，重启移交夹具 40 项通过。安装器矩阵结果限脚本契约。 |
| backend（Linux）`140320539113` | pass | 单独完成前置查询和元数据进程树断言。其余主库回归 2,180 项通过、29 项忽略、1 项已单独执行，作业成功。 |
| backend-windows `140320538869` | in progress | 采集时仍在 Windows backend regression。 |

日志为 Actions 原始行摘录。制品回执保留核验范围及四个 ZIP 成员摘要。本次核对未运行产品，真实桌面首次 Status、离线准备与安装器验收仍未完成。

- [前端日志](raw/frontend-140320539065.log)
- [Windows 冒烟日志](raw/windows-smoke-140320539077.log)
- [作业与制品摘要](raw/receipt.json)
- [Linux 后端日志](raw/backend-140320539113.log) · [作业回执](raw/backend-140320539113.json)

## 后续回读

Windows 作业已通过步骤 8 严格 Clippy、10 普通账户后端回归、11 TokenUser 创建归属、12 原生凭据隔离、13 受管安装升级与离线重装及 14 诊断采集。步骤 15 的 Release EXE 正在构建，最终 EXE 冒烟和后续安装恢复仍无终态。[作业步骤回执](raw/jobs-progress.json)


## Windows 作业终态

Windows 作业完成，步骤 15 的 MSVC Release EXE 构建和步骤 18 的最终 EXE 冒烟通过。步骤 20 的安装恢复驱动失败，退出码 1。摘要错误为 `Native WebView install/recovery assertions failed`，制品中的原始首错为 `Unsupported native model UI acceptance version`，位置 `test/smoke/windows-model-ui.mjs:34`，调用链来自旧安装恢复驱动。后续普通账户最终 EXE 冒烟跳过。

已复算 Actions 制品 `15177215014` 的 ZIP SHA-256，与 Actions 摘要一致。原始 `execution.log` 为 910 字节，SHA-256 `a826da61e0c1b80145e7a09e252e44d1939d53f31713a8433a6a3613d4154c32`。[作业终态与制品元数据](raw/windows-140320538869-terminal.json) · [CI 外层错误](raw/windows-140320538869.log) · [制品原始首错](raw/install-recovery-execution.log)

`main@6aa38bde9` 为 0.2.24 裸 EXE 单独接入了缺固定依赖时的拒绝和基线保全断言。主线作业尚待终态，已供应的固定 2.99 实包失败恢复仍待有效停止证明与可重放夹具。`accepted:false`。
