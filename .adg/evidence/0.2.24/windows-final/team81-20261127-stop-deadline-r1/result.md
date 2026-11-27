# 0.2.24 Windows 原生 Stop 截止时间复核

源码 `97f100b32e03b7d8952933a4bdbeaf519e270907`，分支 `feature/024-windows-native-ci`，`accepted:false`。

待证伪判断：Stop 在完整镜像校验或 Job 最终读回跨过原期限后，仍可能返回成功。旧代码的完整验证没有收到截止时间，且最终成功前未复核期限。现将原截止时间交给 `program_image::verify_until`，镜像校验后复核时间，并在写入保留 Owner 的停止状态前再次检查。Restart 共用该停止路径。

| 检查 | 结果 |
| --- | --- |
| MSVC 格式检查 | 通过 |
| Windows 生命周期定向 | 4 通过、4 忽略，退出 0 |
| 既有 deadline 断言 | 23 通过、1 忽略，退出 0。覆盖 Windows 镜像期限和清单绑定拒绝。 |
| 固定实包 10 秒镜像读回、真实生产 Stop | 未执行，仍待同源候选和明确空闲证明。 |

本轮无失败断言。原缺口由源码定位，未获得生产跨期限复现。未创建桌面夹具。构建参数文件已随退出清除，临时构建根仅留既有 `node-compile-cache`。保留原预算和未知拒绝。

- [生命周期日志](raw/stop-deadline-msvc-r1.log) SHA-256 `3311792deae8b7375ee334676f819f2f7e74442068f73245944389b7e3d2e9d0`
- [deadline 日志](raw/stop-image-deadline-msvc-r1.log) SHA-256 `ff67501d5b8707ed887e3a0d785fe8fdaeed42d6912050d8eb8f049ce45a7e27`
