# 实际candidate CLI producer：完整终态

Run42578649105/job129031965706/source f143d0910bacd47d933ce6ce89426f972368ba59；整轮 **failure / accepted:false**。

| 项目 | 结果 | 证据及边界 |
| --- | --- | --- |
| 原writer / validator | 预期反例重现 | owner-mismatch；仍拒绝Administrators owner |
| 候选创建与原子发布、失败保留 | pass | 原validator与旧字节保留都过 |
| 候选archive双构建/actual atomic入口 | pass | 正式names.2/pin不改 |
| 六组native租约/Job/Rust消费者 | pass | 同H拒绝/child仍活/原生Witness都过 |
| 实际CLI就绪 | pass | 9114.6081ms；原30秒 |
| record/health/lifetime proof | pass | 真实CLI/normal health handler |
| 官方capabilities/POST stop/还原 | fail | ERR_ASSERTION；缺阶段，不能判定具体断言 |
| 进程退出/Job empty/lease free/record absent | 未证明，按失败 | false不等同于OS仍活，也不证明已退 |
| producer fixture清理 | fail | keep精确临时HOME；未知不强杀 |
| strict Clippy | 未执行 | 先决步骤失败 |
| 证据上传 | pass | 15260 bytes；SHA256 44cb682f028c43fc82b8f0ab7ee80d45a77ea553ba0b54f3cf18599253d5c480 |
| stage根移除 | pass | 不代替producer HOME/进程清理 |

依赖缺失的猜测未导致本轮启动失败，不据此改装配。仅增加白名单阶段/HTTP元信息，不输出token、请求头或响应正文。已过且未受影响的前置不再重跑；后续core-sweep每轮先写可证伪判断；全量留冻结前。
