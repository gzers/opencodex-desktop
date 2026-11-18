# Windows r7 同一首启 worker 显式恢复

Windows 实施提交 `5331c4bfca129e11b10c9110d2da06c43d50eefa`。合入主线后的产品源码和远端回读均为 `7123e2f283f0b8a052270ba47dbc12626d70798d`。已消费 TEAM47、IMP186 和 r7 合同。

首启确定未 Resume 后，原 worker 保留原 Owner、协调范围和失败前缀。显式动作在同一线程验证原对象、原失败动作、四角色范围和独立截止时间。错误动作、重复派发、缺范围、过期和未知 Resume 均拒绝。状态读取独立于 worker 动作。恢复结论保持与 Stop、Closed 和后继许可分离。

MSVC 产品构建 raw0，122715 ms，三制品摘要见 [artifacts.json](bins01/artifacts.json)。20 项定向测试 raw0。首轮测试编译 raw101，首错 E0027，旧测试解构遗漏 WorkerControl.registration，另有两处 E0599。首次和修复后的原始输出均保留。测试快照早于最后缩进调整和主线合并，逐文件区别见 [source-binding.json](source-binding.json)。

44 个可从成功测试日志定位的夹具目录均已核实缺席。既存生成的 Windows schema 保留为未跟踪文件，旧失败和审批拒绝删除的目录保留。状态表记录本轮与历史证据的适用范围。旧 r2 已回收的十份原件继续复用。原 Job 24 项未知，零权限复制 raw50，r2 修复计划 executionReady=false。

普通桌面私有 V2、原 Owner 的真实部分获取与生产整链、固定官方包单 SID 预算、普通用户 MSI、CAP trust、原缺席安全退根和候选冻结仍待完成。公开 Start 保持拒绝。
