# Windows原件闭合历史与旧根句柄释放增量

沿TEAM14/IMP140/windows-security-coverage-r4，fetch仍main2ebdd3ae0/docs84ec482ee。在既有Windows独占写集推进，源码754814dc14903868e73223a1a8862c9b33b14e51，tree226fc0476e3a8b451c009b3a316da87cd5c8134e，已推codex/024-windows-native。没有共享trait/路由解析器/Mac/官方包/安装器/Vue/日常配置修改，既有untracked schema未手动编辑/暂存。

本轮先补后继所需原件历史前置：原对象恢复并完整读回后，在原Profile卷私有custody/original-history封存原件字节/原ID/八安全投影及原缺席；复用有界分块存储而标为独立OriginalHistory，归档新文件ID不冒原件ID。数据不入D，canonical v2阶段新增必需historyRecord（None或严格Snapshot），Closed必须引用准确session/binding/policy/custody卷/私有对象ID与摘要，工作Snapshot不得混作原件历史。旧缺字段/Full/partial不转换迁移，纯解析即使textClosed/current匹配也只有PendingOnly。

实际ClientOriginals complete_stop中，在原Stop截止内经typed NormalExit/Absent/StopAccepted/端点/记录/source与完整恢复读回，先封存并刷盘发布Closed历史，再完整读回才释放旧原根DELETE句柄。释放后旧原件存储历史独立核私有封存数据/退休工作树/父根/原上下文，而不跟随新的.live Codex/Grok内容。后续用户创建/编辑可被新的原生捕获读到，同时不篡改旧历史基线；此为原生存储能力，尚未连接真实后继新代登记/Owner替换。

同批NormalExit转为非serde/nonClone ClosedExitHistory，保留原始退出角色/码、原Stop截止与窗口内完成时间；后继动作读取原保留process/Job/source/完整历史时使用新动作预算，不将过期旧Stop证明换deadline后补签。原check(deadline)仍要求准确旧截止，只有原窗口完整检查后的转换可产生该历史类型。实际工厂未跑新官方服务正例，仅编译和受影响原语回归；不冒typed生产验收。

原生custody顶层准确子名从 retained original位置/retired工作根/Ready/PreStop/OriginalHistory原生对象导出，并有独立有限枚举；多余/缺失拒绝、不清除。TreeCaptureV2任一native recheck失败即poison，不能靠去掉冲突文件自动补进度。各目录内部既有全树ID/bytes/8投影/无reparse/单链接/无ADS读回保持；重复读不是防止所有不可信间隙写的承诺。审计SACL/未请求扩展excluded-unknown，fullSecurityProven=false。

| 执行 | 原退出 | 结果 | 耗时ms |
|---|---|---|---|
| library01 | 101 | 29过3败，原件历史重新open与原DELETE handle冲突raw32 | 143693 |
| library02 | 0 | 修复后48过2忽略 | 128842 |
| library03 | 0 | 补准确custody集合/poison后49过2忽略 | 121671 |
| product01 | 0 | clean最终MSVC custom-protocol debug全部bins，lib68warnings | 132797 |

首错为seal_closed_originals内observe_restored_original重新open原根返回Win32 32；后续部分current用例读缺失archive的raw3是上游封存未完成所致，不能当注入成功。修复直接用原保留root句柄读回，不放宽分享/改权限；partialcurrent反例额外断言确实命中真实onebytewrite+sync故障。失败source/log/result/原scene不覆盖。原图闭合前释放拒绝、first-part封存失败保持Restored17/current和原根、真实Closed18之后current部分写损坏保留原根/不释放、历史内/顶层外改拒绝、后续live用户变化与旧历史分离均有新native组件证据；仍不从fixture签ClosedEligibility或下一Start。

三次真实库测试EXE在覆盖前取摘要；97份产品执行输入对最终工作字节一致/Gitblob行尾等价，library03的53份Rust同最终，归final-source.json。Desktop77356544/SHA256fcd4fdc9231e2d1d41d85a1cf2dc8ec46b438020e98ae1058ab7fcfa18446f6e；ocxd5398528/SHA256e8220bc793989c3a155c14fd1d60afc3ef68fcf7bf1541529242011526d8445c；ocx306176/SHA25672f2fc3e2f53f98f29029831b2e1b1a0df5f511f15488bd2ff0573bf9de97e85。不是release预算/冻结候选/发布。附successor-export01.error.json记录一次证据读取脚本默认GBK解码失败code1，改显式UTF8后只读导出成功；非Rust/native运行失败，不倒填firstError。

54个保留场景839文件43529262字节已私有ZIP内容核验，非NTFS身份/安全克隆，原场景均留。其中两处本轮全过的合成正例.tmp1fI41E/.tmpN3C2VK（64文件236993字节）在归档后拟追加逐项当前字节/owner/reparse/占用检查再删除，整条PowerShell exec在创建进程前被自动审批blocked by policy拒绝；该条命令内检查及删除均未执行，另只读确认两根在。cleanup-positive-scenes-rejected.json保留事实，未换工具/换脚本/重试这两根或任何旧拒绝。其余失败/中断/预期Unknown场景原样保留，正例guard清理自身未保留对象。overallCleanupComplete=false，无新账户/Profile/Public/GUI/官方/force/日常PATH策略写。

完整状态矩阵归final-status.json，后继真实未完调用点归successor-remaining.json：新capture仍要求D/windows为空，没有已登记闭合集合/前驱引用消费；authorize_successor重验旧历史后仍明确拒绝；Slot还只保留一个Owner，尚未接原多代Owner/Job/原始退出历史保管以及新捕获/Resume。不能删旧记录绕空目录，也不能扫textClosed收养。下一步实现精确历史集合原生移交/新代登记，保留每个原创建者权威，原预算/Unknown不减。

前轮shared-minimum-request仍请求集成线提取Mac私有纯路由声明解析器，Windows未擅改共享层。完整官方副作用/活跃NativeLease/后继/GUI原官方Owner/S-D生产组合、普通MSI及历史清理仍未完；publicStart/后继拒绝保持。0c真实GUI准备输入未覆盖，out观察仍空，待marenben原桌面执行。B2 CAP/trust非空1314/5无新前提不重跑。allAccepted/productionExecuted/actualGUI=false，无业务同步/模型请求/W2/双SID/发布。
