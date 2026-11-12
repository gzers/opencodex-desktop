# 0.2.24 已恢复会话的独立终态与重新使用

revision: 1；2026-11-12。归IMP-34 P24-04，承接[整树与两锁读回](client-full-readback-r1.md)。代码main00d01df24。当前原样官方2.99 Mac常规静态图两次真实Start/Stop/完整文件恢复闭环已通过；本批诊断accepted=false，不作整版、通用图、所有客户端范围或固定候选验收。

## 持久终态与正确重开

Index当前新格式user_version3：Head新增completionSha256，State新增Restored。只有RestorePrepared可由close_restored消费真正不可序列化ReadbackProof关闭；普通advance无法跳到Restored，JSON标签/一个摘要或磁盘outcome不能制造Proof。Proof现在绑定完整References、当前独立plan、data/client原生Identity及原Stop绝对deadline，不能换会话、原件、计划或根后复用。

关闭前读取原件四阶段、startup-scope/live及completion各自已独立登记的SHA，核对profile/client/data原生代次和原Scope关联、Ready与Live关联，随后核对完成回执refs/plan及实际complete outcome SHA/决定，重开Pending验证原计划。通过后同一索引事务CAS提交Restored、completionSHA并读回。提交前后均受原Proof时间检查；若提交后发现超时，另追加保留的RestorePrepared安全降级事件而不留下成功启动门。本批没有注入慢I/O来代替这条分支的完整时间验证，不能声称任意阻塞磁盘下严格实时保证。

观察端在shared action锁内只读校验current/event一致及登记的完成原件链；缺回执、摘要/决定/Scope/Live变化即Unknown，不能扫描目录学习新baseline、初始化索引、自动重放旧apply或把官方status错误抹成已恢复。正常Restored映射client_recovery=NotRequired，缺来源/处理中等原未知仍拒绝。

reserve_start只接受无历史的合法新索引或已核对Restored。后者先重验完成链，再创建新ticket、递增全局revision并保留全部旧events/原件目录，不清旧历史、不把旧原件当新一轮捕获。后续合法用户配置编辑不会被旧receipt与当前树强行比对，不自动恢复其编辑；新会话按自己的当前原件重新受控捕获。

旧索引1/2不兼容、不自动迁移。本版尚未发布的旧工程测试现场按原目录留证，生产不读旧格式凑通过。状态/索引schema不同于原件frame schema4和官方2.99配置schema，不混称它们升级。

## 产品入口

普通MacStop在原服务/父normal0、原4捕获TCP缺席、claim/N/L/K内实际三项补偿与后像归档、完整Codex/Grok路径/Protection读回、精确两稳定锁重获释放和完成回执之后，登记终态并读回才返回Stopped成功。UI/CLI/tray共享原命令边界，重开状态因此能显示无需恢复并允许Start。未知/失败继续保留pending，原30/5/10不变，无强杀或上游补丁。

Restart命令的共享控制流已从完成Stop继续进入新Start，原Stop deadline clamp保持；本批真正执行的是Stop→状态重开→新Start→Stop，**没有执行单独的Restart动作**，其总预算/一体命令受影响验收仍待后继。不能以这次两轮替代Restart/崩溃显式恢复/完整生命周期集合等剩余项。

## 检查、真实两次会话

36项去重相关函数通过：index21、completion6、session9（含既有辅助入口）；相关manual入口ignored不计通过。首index组20过1失败是新增伪造终态反例复用revision6，提前被events唯一键拒绝；只把测试伪造revision改为7再定向通过该项，不降产品判定、不重跑20已过项。最终编译/格式/全目标严格Clippy通过。

新供应原样固定官方2.99，准确build binary SHA 878d10e10253bc73fee833973483749a2305585470e1fdce83d24493d17226ed与相关检查和执行副本匹配。OS sandbox拒绝日常HOME读写/外网，仅loopback、自有根与精确协调DB/既有globalL借用。第一次完整断言原样通过后，显式加第二次会话验证，不改原catalog样例或放宽恢复断言。

| 项目 | 第一次 | 第二次 |
| --- | --- | --- |
| Start | 8842ms，真实Started/ready | 6677ms，真实Started/ready |
| Status | 226ms，原PID/port/ready | 未额外新增独立查询，Start自身真实ready保持 |
| Stop含全部恢复/关闭 | 8646ms，返回Stopped/无需恢复 | 9013ms，返回Stopped/无需恢复 |
| 索引 | Restored/revision7 | 新ticket，Restored/revision14 |
| 七路径原字节/缺席 | 全true | 全true |
| 重开观察 | 1234ms，Stopped+NotRequired、canStart=true，无PID/port | 后续独立只读观察再次识别revision14 |

每次原服务与父normal0、4实际捕获TCP全缺席、实际三件apply/原后像归档、完整树安全属性及两稳定锁释放、完成回执与索引读回均出现在准确产品trace。原Start30/Status5/Stop10秒保持。整个原生集成函数exit0/result=passed，外层诊断仍accepted=false，不能当整版放行。

## 回执完整性与历史留存

之后只加一个ignored只读观察helper，不改产品算法；重编时准确源码/binary独立记录，没有据此重复服务。8场景实际读取已有成功现场：合法回执→Restored；receipt缺失、receipt/outcome/scope/live改成坏内容→全部拒绝；自有测试还原原字节/0400后→Restored；完成后用户编辑当前config.toml→仍Restored。各次索引原字节不变、无修复/服务启动。fixture的原件破坏与恢复只用于这些明确的合成完整性反例，不是产品偷偷修证据。

独立SQLite读取核对schema3、events1..14全在、两个不同ticket和两套不同Originals。各自四frame、Scope/Live、独立plan、completion/outcome及所有content引用SHA/0400/长度吻合，旧完成事件也仍保存原独立摘要。最终额外完整原生Codex/Grok快照SHA与第二次completion逐字一致。没有把历史第一份completion拿来覆盖第二次或自动清理原件。

## 证据、清理和剩余范围

证据.adg/evidence/0.2.24/client-session-close-r1/保留假设、初失败/定向修正、每版源码/build/binary、真实双会话原输出、索引14events/两份完成原件与plan/receipt、8完整性场景和manifest。全部只含合成配置，不保存runtime secret/admin token。

自有服务/launcher/Node/Bun/test与完整性helper命令已缺席；从两独立Live记录读出的两个真实服务主端口经完整Darwin TCP表framing核对均缺席。其它TCP只引用准确产品trace，不编造未导出全端口表的独立清理回放。3个精确新外部协调DB各重获/释放EXCLUSIVE后，与自有程序/Profile/两套客户端blob/index/stage/archive/control副本和重复临时回执清理；既有globalL原inode/字节/0600/owner保持，独立排他取得释放后留原文件、无新sidecar。未改日常/官方/系统设置，无业务请求或发布。

本批未构建新release/app/DMG，最终固定候选与MSVC/Windows产品接线仍未过。普通当前静态图Mac完整Stop及重新Start已取得本批证据；一般图/别名/Fast/组合、外部Agent/一般路径/首次缺根/大树/跨卷/flags适用范围、活跃刷新协调、显式中断恢复/回退、Restart命令完整预算、原历史revision4首错、完整生命周期集合/其它监听证据继续。WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README与候选验收仍在IMP-34，目标active。
