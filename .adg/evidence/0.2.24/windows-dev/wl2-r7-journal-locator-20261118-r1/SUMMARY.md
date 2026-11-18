# Windows r7 原 journal 目录定位摘要

实际受测源码 61e3a1e8eef34a17db51e131534527eae5f1d8c7。随后合入 Mac 主线 9daf5599151b65437f94ab2cadb1988060ee38c3，交接代码 4b8284f152d4940fd9e7f7b30abd304c909f6a98。主线增量仅涉及 10 个 macOS 文件，两个 Windows 源的 Git blob 与受测版本一致。治理输入 503e5aceba86b5a507a8368ae9af668c3eba8d26，TEAM42 / IMP181 / r7。

V2 journal 聚合对象用原生保留目录的路径计算定位摘要。目录成员与初始化状态继续进入观测摘要。源、安全基线或成员未知时仍为 Pending。

首个原生基线在故障点 1 发现目录 identity 相同但 pathDigest 改变，Cargo raw101，用时 240192 ms。首轮运行器输入误把两个路径拼为一条，未启动 Cargo，原输入保留。修复版完整 MSVC 测试 raw0，用时 269555 ms，4 项通过，覆盖 10 个故障前缀、10 个槽位映射、9 个拒绝场景和 1 个原 Start 过期后的只读场景。同制品的 3 项关联回归通过。TEST EXE SHA-256 51ae3050151c2ad363fd2d7a9063adc012280eff5680468a88724452fed44efe。

核对 32 个成功测试根均已清理。原生失败测试根保留，生成 schema 原字节已归档，代码工作树已复原。status.json 共 190 项，复用此前已过且未受影响的证据。

普通桌面私有 V2 仍待实际结果。原 r2 Job 查询有 24 项未知 / raw50，修复计划仍为 executionReady=false。初始 Start 原 worker 接口待主线交接。生产 Start 保持拒绝。
