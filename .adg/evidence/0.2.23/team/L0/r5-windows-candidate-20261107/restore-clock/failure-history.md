# 真实失败与后继修正

- 时钟首版旧用例仍断言版本账本恢复为缺失，失败后改为业务文件真实原 None，版本账本保留本次保存事实；没有保旧时钟迎合断言。
- 真实 capture/自动方向首轮失败。诊断证明 metadata Policy 的 channel 原子与 actual observed Policy 不同，修正为实际 provider_import observation 后再调用已有 saved_state。原 fixture 在已有当前 clock floor 上强写更早时间触发 ClockUnreliable；按现引擎规则保留此原失败日志，不绕过 clock floor。
- 方向测试必须以内容匹配的明确保存 cloud 为旧来源。metadata-only receive 的部分 channel 时钟在 actual capture 中为 Unknown，现自动方向对 Unknown 云默认 Download。没有修改该共享策略；最终测试通过实际 SaveState + 现 planner 建立明确保存的旧 cloud，再做真实 restore/capture/自动 Upload。
- 版本账本是派生保存元数据，不可再强求等于原操作 after 旧字节。preview 冻结当前版本目标 guard；restore 写前 CAS 该 guard，并以当前账本规划 derived receipt，保留其它后来未选原子。此旧 guard 拒绝原日志保留。
- 最终移除所有诊断 eprintln，13 项定向与严格 Clippy 通过。成功/失败日志都保留原字节。
