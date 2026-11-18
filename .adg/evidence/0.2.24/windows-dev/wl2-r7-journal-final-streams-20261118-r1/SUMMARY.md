# r7 已初始化 Journal 目录末尾数据流复查

源码 89e8c64478553c9715eb5babfe00f9eeed14af11。基线在已初始化但未转移的原 Journal 上完成首次读取，再向合成目录添加命名数据流。原读取返回 Pending DATA，断言失败，Cargo raw101。失败现场、原始输出和基线测试制品摘要均已记录。

修复在原两文件复查之后再次验证目录数据流，沿用原读取截止时间。MSVC 定向编译及 5 项测试通过，raw0。反例收到 InvalidData，两个原文件句柄仍保留，初始化授权仍拒绝，合成命名流仍可读。测试 EXE SHA-256 为 f2c0d1163669854aaecc39b8231a99d3e636bb56cd07f409093e6abf03eebf80。

31 个成功合成测试根已核实缺席。生成 schema 已归档并恢复工作树。193 项状态见 status.json，包含此前未受影响的验证；本轮私有桌面 V2 仍待真实输出。

入口 fetch 和代码推送经指定代理失败，原握手回执已保存。最后实际消费 main 9daf5599151b65437f94ab2cadb1988060ee38c3、governance 503e5aceba86b5a507a8368ae9af668c3eba8d26、TEAM42／IMP181／r7。远端交付以后续独立回执为准。

r2 已核对全部十份只读导出。原 Job 仍有 24 项未知／raw50，修复计划 executionReady=false。原始 Start worker 及同范围恢复接口沿已提交 handoff 等待主线。公开 Start 保持拒绝。
