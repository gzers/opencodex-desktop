# r5共享接口合入、原生路由与状态同代采样

TEAM15/IMP141/r5，main358e7fc47和docs676cec18a已精确fetch合入。代码merge76c4f7185后分批a6ea0c1ce、bc64e6f1001e8533e4ab845d24ce89edff9cdf86，tree19aa5deda9baffa02f63ab5d40bb658bd66dd66f。唯一冲突windows_official_lifecycle024.rs保留完整Windows原文件，merge-resolution给原blob/工作摘要和CRLF等价，完整recovery_state_at保留；其它共享变更正常合入，未全仓ours/theirs。docs承接40ccb3557，正式TEAM/IMP来自集成线，不由Windows改权威进度。

RoutePin在实际H/config.json保留原文件/父pin/ID/8投影及精确内容，逐项/合并scope仍r4；2MiB、single-link、无ADS/reparse，解析消费共享declaration。None/Err拒绝，Some(empty)区分；活跃Store每次source边界重验原配置，Closed释放，后继新捕获。不从生成catalog倒推，不将routes向量变恢复资格。原工作树不变检查/副作用未知拒绝仍保持。

OfficialStatusSource同一采样fetch/light→native status_sample与recovery_for_sample共享相等原deadline。新增私有不可serde/Clone的一次receipt绑定原Slot Arc/session/scope/stopped/deadline，在同H原Owner互斥下比较，并执行同一完整Closed原生读回；缺失/过期/错上下文/锁忙不清恢复。registry准入改try_lock防无界等待。原recovery_state_at完整函数保持；回执不能制造Closed。真实WindowsSource缺Owner路径trace有fetch/recovery截止5000019us、light/recovery5000177us，均各自同一原5秒，注入调度时间未来1小时不作native起点。不声称已测试实际Closed正例或Owner并发更替成功。

| 执行 | raw | 结果 | ms |
| --- | --- | --- | --- |
| product01 | 0 | 合并基线custom-protocol MSVC debug bins | 167036 |
| library01 | 0 | route11/status33/source4/native6，共54项 | 192109 |
| library02 | 0 | 最终相同54项，时钟断言更精确/日志边界纠正 | 94258 |
| product02 | 0 | 最终冻结源custom-protocol MSVC debug bins | 208243 |

无非预期编译/测试错误，首错null。library01成功反例stdout错误称retained，实际Fixture正常scope清理，未打印保留路径，library02纠正原文且全部原日志保留，未虚造私有ZIP。最终产品输入114份、最终测试Rust68份对最终工作字节/Git blob行尾等价；初轮脚本未包含共享6输入，冻结构建期间另补逐字shared输入清单，后继脚本正式纳入。前端32文件1162800字节保持。实际执行库EXE在覆盖前固定摘要。制品非release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77689344 | 3c150e07df2d77167f5f71e946877cdf394e89434b9c0ee05bc89f7bf4972aff |
| ocxd | 5398528 | 7316c0375239b3f091d7a53d002ee5d51dde14622aa341011c6c12770bc020f6 |
| ocx | 306176 | a73a003a1392d24a89417b4e8fdabc0a2534bb18b04c43e129824a6f2ae41144 |

完整B1/B2/B3/O1/O2/H/S-D/P/MSI/Route/Sample表见final-status。已过trueGUI空broker不重跑；Full-v1/partial不转，scope audit excluded-unknown/fullSecurityProven=false，原30/5/10/Restart30Stop子10不变。NativeLease/官方副作用和真正原包H-S-D闭环未完，Start仍拒绝。固定官方bundle/内tgz及9文件逐字核验，无官方运行或修改；本地SQLiteWindowsVFS片段只是后继实现参照，不当Bun互操作证书。

无新账户/Profile/Public根/GUI/官方进程，成功合成反例自行scope清理；历史失败现场/旧broker40588/Public20文件8拒读/普通MSI及所有旧清理拒绝保留、不重试，overallCleanupComplete=false。不日常PATH配置策略修改，无业务同步/模型请求/W2/双SID/发布。
