# r7 完整 MSVC 制品扫描与原 10 秒复验

本轮消费 main cd7cd0a9e62b3f5f88e0054c3498cbfe18616943、治理 503e5aceba86b5a507a8368ae9af668c3eba8d26、TEAM42、IMP181、r7。代码 0816047f30f667e56bea4167ec012f6e12bfcbdd，写集仅 program_image.rs、program_image_windows.rs、program_image_windows_tests.rs。封存时两次代码推送均在指定代理的 TLS 握手失败，实际远端回执独立记录。

| 验证项 | 结果 |
|---|---|
| 原完整 MSVC 制品固定副本入口 | raw101，10079.394 ms，runtime_image_unconfirmed；原断言、stdout 和 stderr 保留。旧输出中的 completeInventoryAndManifestReadback 为固定场景标签，阶段未知。 |
| 原制品独立诊断 | raw0，1/4 worker 和参考实现均按同一 tree SHA 完整核对 12213 条目、623970751 字节。native 4 workers 2698.961 ms，其中枚举和保留句柄 1638.789 ms，最终 fence 864.931 ms。诊断无总 deadline，native 样本在参考扫描之后。 |
| 完整 MSVC 构建及受影响回归 | raw0，139473 ms，16 通过、3 个显式固定副本测试跳过。长路径、ACL、ADS、对象写删、取消与 join 均通过。 |
| 改动后同一固定副本全树与 manifest | raw0，2634.174 ms，原 10000 ms；inventory 2584.541 ms、tree digest 46.565 ms、manifest 0.827 ms，全部读回通过。 |
| 制品 | 101999616 字节，SHA-256 0b900fa43f2c9d2e3001cfcd9545d77d669a14684f88b8e592886eea20b13664。 |
| r2 归档回执 | Public out01 十个原始文件与已推送归档逐字节一致，10701 文件、1664 目录。归档 ZIP SHA-256 c3906f7814f48c00f88ce16f6de6fbdc55b6bdffcaf2f2f2c61f3af9a67c6c0a。 |
| r2 现场处置 | 原 Job 24 项未知/raw50，计划 executionReady=false，原进程和原件保留。 |
| 本轮清理 | 成功测试夹具按原 RAII 原语清理，前后无新增根。生成 schema 已封存，代码工作树还原。使用现有只读固定副本。旧自动审批拒删项及历史失败证据继续保留。 |
| 仍待 | 初次 Start 活动 lease 的原 worker contract，普通桌面私有 V2，当前 r2 现场恢复，真实单 SID 固定原包流程，生产预算/容量/CAP trust，普通 MSI 和候选冻结。 |

Windows 每个 reader 的读取缓冲区只分配一次，原生路径常见长度使用栈缓冲区，长路径按 API 返回长度有界重查。原文件全字节、原生路径、ADS、安全、成员和最终 fence 保持。诊断顺序与两轮缓存或主机负载未固定，结果未量化因果提速。此轮证据范围是固定副本，未执行官方包、生产恢复、W2、双 SID 或发布。完整既有 179 项状态见同级 wl2-r7-bound-preparation-20261118-r1/status.json，续项详见 status.json。
