# TEAM83 Windows 原生 Start 截止时间

2026-11-27。判断待证伪：Windows Start 或 Restart 在 LaunchPlan 完整校验后再次扫描镜像，旧调用没有传入原截止时间。若这一阶段耗尽预算，产品必须拒绝创建或恢复进程，并保留已建立的原 Owner。

Windows 源码提交 `5e08b7d54650840ec6beebf1db1605be3aa4f440`，已推 `feature/024-windows-native-ci`。本变更文件的提交前字节与 main `37a141fbee8e357db3d42f2aa830c6be2c8be195` 一致，集成只需挑拣这一笔。畸形 PATH 夹具沿用 main 自己的修复。第二次扫描沿用 ActionWindow 原截止时间，创建挂起进程前及恢复进程前各复核剩余时间。在后一检查耗尽时，保留已经登记的 Owner 和 Job。完整镜像及未知拒绝沿用原规则。

| 检查 | 结果 | 证据 |
|---|---|---|
| 本机 MSVC 原生生命周期定向库测试 | pass，退出 0，4 过、4 个实机用例忽略 | [原始输出](raw/windows-lifecycle-msvc.log) |
| 镜像盘点越过截止时间后拒绝返回且等待 reader 结束 | pass，退出 0 | [原始输出](raw/windows-image-deadline.log) |
| 完整镜像维持原截止时间及根清单绑定 | pass，退出 0 | [原始输出](raw/windows-image-original-deadline.log) |
| 真实生产 Start 跨过截止时间时的创建和恢复边界 | blocked，待同源制品及实机前提。本轮通过的是组件断言 | 同源 MSI／NSIS 待交接 |
| 离线完整包、确认后缺包下载、首次 GUI Status、真实启停 | pending，保留旧回执和已过项 | [前一轮证据](../team81-20261127-7580b0d-r1/result.md) |

本轮首错：无定向测试失败。rustfmt 与 Git 差异检查通过。测试 EXE SHA-256 为 `a57c44bfaf4e4127ba4991d710ede09b6cd35469e49a74059ed0affb6537c04f`。构建前已有的两个 schema 文件已逐字节恢复，SHA-256 均为 `37ad285871f02aca13444391a6d2f14c4a7c6e8e464ff8cd5d32214d72387d49`，仅 Windows 生命周期源码入代码提交。本轮仅执行库夹具，未开启实际产品、创建测试账户或变更安装器和系统策略。历史证据保留。

原始日志 SHA-256 见 [摘要清单](raw/sha256.txt)。截至本次记录，Actions `45956736730` 和 `45956890861` 的 Windows 后端回归仍在运行，其已上传的冒烟证据和前端证据均不是安装器。候选未到，Start／Status／Stop 保持 30／5／10 秒及未知拒绝。暂停 W2／双 SID，未发布，`accepted:false`。
