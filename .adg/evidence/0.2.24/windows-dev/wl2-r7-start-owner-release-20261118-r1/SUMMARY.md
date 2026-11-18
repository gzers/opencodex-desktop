# r7 首启原 claim 与原生释放回执

首启拿到第一个范围后，第二角色被真实 SQLite 写事务阻挡，返回 raw33。旧入口释放了已取得的原 claim，外部探针成功抢到。MSVC 断言 raw101，原日志和失败根保留。

准备释放修复 737bca2e3cd8899cda0d0d86bf8b5fbb446ebca1 已推送并经远端回读。解锁一组范围后注入 raw5，外部仍拿不到原 claim，journal 保持 ReleasePrepared。首轮套件 6/7，首错是固定 Bun 旧夹具路径缺失（Win32 raw3）。同组使用校验过的原 Bun 路径后 7/7，raw0，SQLite 负控 raw73。

最终恢复释放补丁保留在本地。原四项逐节点恢复先耗尽窗口。批量注入反例进入最终释放后没有取得 raw5。使用同一制品及固定 Bun 的批量正例在最终释放调用内报 bounded-original-deadline。原 10 秒及断言保持，最终释放阶段耗时待定位。

自动审批在执行前拒绝删除两处已成功合成目录，两处均保留。六个新增失败根已核实保留，生成的 Windows schema 已随轮次归档。r2 沿用完整只读归档。Job 24 项未知，raw50，executionReady=false，原现场保持。

已消费 main 9daf5599151b65437f94ab2cadb1988060ee38c3 和治理 d751f9c44dc1168d71821c5b7bbcd381b736e6b7（TEAM45／IMP184）。原持久 worker 和基于原范围的显式恢复待接入。公开 Start 保持拒绝。普通用户 MSI、原包单 SID 生产闭环及候选冻结仍待完成。
