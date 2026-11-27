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
