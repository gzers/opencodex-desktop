# r7 原生租约实际范围回归

源码 e54a1156de42d445903faedb93aed4923f400b3a，基线 e6cc1f29a5444a76da3e123d6f97b7b6ee6757f2。已消费 TEAM46 / IMP185 / r7。

MSVC 基线 0/1，raw101，144518 ms 含编译。首错 coordination/preparation/tests.rs:111:5，断言 lease.check(deadline).is_err()。基线制品 SHA-256 8b6f5b75bdf1c07b737b31b5bc05a436e116e3149029aa6a42814e1e39fd7352。修复版 5/5、raw0，148815 ms 含编译。原范围缺失时拒绝，原 claim 保持冲突，真实 SQLite raw33。

修复制品 SHA-256 a8f32eff2cba875ae93339e14a12ab701754043af26d59da9d987b48d01e7985，102022656 字节。校验后用固定 Bun 跑同制品的分段 7/7、24551 ms，恢复中断 1/1、116199 ms，均 raw0。总耗时跨独立动作窗口。原制品路径复用，基线二进制仅有原摘要。

失败目录保留，当前 r2 和原 Job 不动。完整 216 项见 status.json。持久首启和同范围显式恢复、普通桌面私有 V2、普通用户 MSI、原包单 SID 生产预算与候选仍待。公开 Start 保持拒绝。
