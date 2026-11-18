# V2 待恢复记录绑定原准备槽

代码 dc3e3248550081c9070d1a56664a76148a01219b 已推送至 codex/024-windows-native。本轮消费 TEAM42、IMP181 和 r7，主线基线为 cd7cd0a9e62b3f5f88e0054c3498cbfe18616943。输入、构建和累计状态见 final-intake.json、run-results.json 和 status.json。

原准备槽以实际捕获、宿主和 Owner 身份绑定记录。原失败 Start、固定恢复窗口及 QPC 贯穿两次原生读回，同时复核仍保留的 Source、Route 和 journal。缺失的 Source 保持 Pending，恢复许可仍由原生流程决定。

最终 product05 MSVC 测试构建返回 0，用时 135738 ms，三项全过。此前 37 项受影响回归在 product04 制品上全过。测试 EXE SHA-256 为 7b177e0c3884442e7bae099a946ed98b1d48674728cf595055d242974fd4b639。八个最终源文件已与提交逐个核对，仅归一 CRLF。

首轮 Cargo 返回 101，LLVM 内存不足，rustc 退出 0xc0000409。仅把本次命令的 codegen units 从 16 改为 4 后重试。product03 中断，原始退出码缺失，部分日志保留。回归首轮选择器错过 Windows 同采样测试全名，在执行前停止，修正选择器后通过。

核验 39 个明确记录的测试根均不存在。两份生成 schema 原字节已归档，受跟踪文件已还原，新增生成文件已移出代码工作树。旧两次删除的自动审批结果为 blocked by policy，原目录与归档保持。

当前 r2 完整归档已保留。原 Job 查询依然未知，修复计划仍为 executionReady=false。初始 Start worker 的最小请求已通过 Git 交给 Mac。普通桌面私有 V2 验证仍等原入口输出，生产 Start 继续拒绝。
