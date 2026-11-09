# 首个 core-sweep：完整状态与假设对照

Run42581940251/job129042512455/source ab0b5d043d9a93b5d9434bf5f0758423fa15f58a。8回执齐全，7通过/1失败，**accepted:false**；582.067秒（9分42秒），达到8–10分钟操作目标；准备构建独立，不计入核心执行。

| 项目 | 结果 | 首错 |
| --- | --- | --- |
| 01 hypothesis-and-inputs | pass |  |
| 11 two-real-sids | pass |  |
| 12 private-bundle-supply | pass |  |
| 13 runtime-readiness-and-ports | fail | readiness: user1: no unique successful actual runtime start receipt; environment-ready is not service-ready; ports: user1: runtime_start_not_healthy; stage=wait-owned-health |
| 15 two-user-paths | pass |  |
| 16 same-user-ipc | pass |  |
| 17 cross-user-ipc | pass |  |
| 18 account-and-process-cleanup | pass |  |

假设H-core-entry-20261109-01未被证伪：两个原13/14断言均保留，合并失败没有遮住路径/IPC/清理结果。该结果只验证新装配线，生产names.2与原停止扫描尚未改，不能称为修复验收。原失败不覆盖。

失败原件：user1/runtime-1/owned-health-observations.json 的首次完整health/status同PID8008、port59100、HMAC通过，但elapsed/total31288ms，超过原30000ms；action=result:started并不豁免确认总预算。因此不是已证明服务永远不健康，也不能把31.288秒的迟到健康补判通过。

补充Stop原始app.log显示原预算内已进入官方请求：初始owner0ms、复查25ms、原生manager42ms、最终owner81ms、激活85ms、record317ms、连接317ms、首次签名351ms、token352ms、capabilities352ms、再次签名366ms、写前复核374ms、POST-client-restore593ms；到9743ms该阶段仍未完成，整次10021ms失败。不可据此跳过官方客户端还原、延长10秒或删除记录。完整CLI状态诊断仍可能超5秒；缺保护/启动事实保持unknown。

两账户私有供应约07:59:04/06终态，OSProfile/DPAPI/IPC与原严格清理继续有回执。清理通过不能修正之前启动/停止失败。整个原artifact1411680字节ZIP/SHA25636220f74339e894dd28f160032ee634ba3f965ecafbfb526550c7190500d710e及所有evidence-manifest成员摘要已核验。

下一批：保持已过且未受影响的原型/安装器/回归门闭合；结合已采用的新建TokenUser记录、本H生命周期证明方案，集中处理完整启动确认和官方停止链，不再派一关一轮。先固定具体源码原因及反证条件，才允许下一core轮。普通UI和全量冻结验收仍属整版未完。
