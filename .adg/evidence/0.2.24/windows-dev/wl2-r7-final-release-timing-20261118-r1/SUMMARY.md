# r7 最终释放与完整扫描计量

源码 e6cc1f29a5444a76da3e123d6f97b7b6ee6757f2 已推送并回读。主线 9daf5599151b65437f94ab2cadb1988060ee38c3，治理 d751f9c44dc1168d71821c5b7bbcd381b736e6b7，TEAM45、IMP184、r7。

最终释放借用原 lease。真实解锁失败保留剩余范围和 ReleasePrepared，Start 保持拒绝。每次移动前后继续完整扫描两树，每次 durable append 继续完整回读重放。同一操作内相邻验证共用一次扫描的边界见 SAME-BOUNDARY.md。

| 项目 | 结果 |
| --- | --- |
| 早期计量制品 | SHA-256 b557a04a0ef5f879a97c36cfd3ed722cf13e84fd0c80b39372e6a9c94cfc4146，raw5 通过 9.27 秒。正常对照在最终释放内超时，release verified 附近只余 418 ms。 |
| MSVC 测试制品 | SHA-256 f4a02cbdd012ba7210419f8c466b0b98dd045c2e888a233a16392849c585fdea。实际 raw5 负例 1/1、8.55 秒，独立竞争确认原 claim 仍持有。恢复中断组 1/1、125505 ms，退休组 6/6、49229 ms。 |
| 同制品 10 秒正常对照 | raw101、10070 ms。首断言 retirement_batch_tests.rs:106:10，bounded-original-deadline、底层 raw=None。固定 Bun SQLite 负控 raw73。 |
| 单次 crate opt-level=1 MSVC 正常对照 | 同源码 SHA-256 030692def4d1b7b4487463fbef8bd1c24664d3f0ab406bc59e7f626d77846501，1/1、raw0、9.40 秒，固定 Bun 负控 raw73。构建加运行 603559 ms。该合成轮的系统负载与缓存未控制，生产预算待真实原包验收。 |
| MSVC dev --bins | raw0，115371 ms。opencodex-desktop.exe SHA-256 02492f608e6e7f85f3e5b3016bcdb0aaaac2ca684a2da9d790b96cfa6f03eec8；ocxd.exe 545cd216aeba79d65e962e739ba20a145a14336545e13405773646062a873117；ocx.exe 0eb2cfa0f0eedd1f040a592d4283ebacf5912de882d03128e19f747eafa1930a。 |

源码两次 OpenSSL push 报 unexpected eof while reading，改用命令级 schannel 后推送与回读成功。本轮 fetch 首次 schannel 握手失败，同代理和同参数重试成功。原始 stdout/stderr 与制品摘要随本批保存。

旧库存 76 根，2 根缺席。自动审批在执行前拒绝删除四个成功根，全部保留。优化对照的两个新根见 crate-opt1-fixture-inventory.json。失败根保留。当前 r2 全归档沿用，原 Job 24 项未知、raw50，executionReady=false。持久首启与同范围显式恢复、原件归还及候选验收仍待完成。
