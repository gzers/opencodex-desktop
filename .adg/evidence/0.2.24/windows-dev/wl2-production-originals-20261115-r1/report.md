# Windows r4 实际原件适配与Owner接线增量

本轮沿TEAM14/IMP140/windows-security-coverage-r4写集，源码c30168689876e9577b04c7fe1b63a8a795923359，tree8de36584ffa28970e866e12465aee5894b080a37。分别提交46f9b6526（原创建者未Resume准入）及c30168689（实际ClientOriginals/Owner接线），已推codex/024-windows-native。不是新官方生产验收或安全完整审计。

新适配固定普通noJob broker、原SID/Profile/D/H/I/owner/current及官方2.99 image来源；实际原件Store在首次原根变更前挂入Owner，失败不因tag被消费而丢保管。非serde/Clone的阶段tag只定位同一Store，磁盘文本不重建资格。原创建者单调Resume状态/原thread/Job历史拒绝过期/已Resume/退出；不声称对外部不可信线程API证明native suspend count。

Ready/preStop/completeStop接真实typed观察及NormalExit/Absent/StopAccepted，恢复前后重复source/端点/记录/整树核验。工作树目前仅准入未变，声明官方副作用/durable Ready与PreStop快照仍未完；未知变化拒绝不能当已实现完整官方支持。后继历史移交/旧根DELETE句柄释放/新代次捕获未完成，authorize_successor仍拒绝。公共require_durable_capture/Stop恢复拒绝保持，没有新官方进程或业务/模型请求。

| 执行 | 原退出码 | 结果 | 原耗时ms |
|---|---|---|---|
| library01 | 0 | 13 passed | 155167 |
| native01 | 0 | 1 passed，原创建者负例 | 3601 |
| library02 | 0 | 2 passed | 88322 |
| dispatch01 | 0 | 28 passed、2 ignored | 114783 |
| product01 | 0 | clean MSVC debug全部bins；lib报告68warnings | 179254 |

首错null；预期拒绝不作运行失败。library02 EXE在单独摘要前被dispatch01覆盖，缺失明确列出，不以dispatch01摘要代填。dispatch01执行EXE94966784字节/SHA25688aef9c75e596c468c35b41f145950330312a8c32ce8f7140c0dd66638f8b23f，独立原source/log/result和另外两份实际EXE摘要保留。product01全部执行输入对最终工作字节一致、Gitblob行尾等价；详细制品SHA见product01-artifacts.json，完整状态见final-status.json。

B1/B2未受影响组件复用；CAP/trust非空CREATE_NEW1314/5条件未变，不重试或提权。旧Full/Partial不转换，auditSACL=excluded-unknown/fullSecurityProven=false。完整生产O2/H/S-D/P组合、后继及普通MSI仍未验。用户最新确认marenben已登录，实核session7 Active；尚无新真实GUI执行。旧空宿主/fixturemanager不冒actualGUI，后继需准备真实产品入口。

8个中断/冲突synthetic scene已私有ZIP内容校验，70文件184854字节；ZIP不克隆NTFS安全/ID，原生现场保留。正例自有guard清理；无新增账户/Profile/Public/官方进程/force/日常PATH配置策略写。全部历史删除拒绝不重试/不换机制，overallCleanupComplete=false。不可覆盖原始证据和离线verify只验字节，不签native资格。不W2/双SID/发布。
