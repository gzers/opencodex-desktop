# 原契约、固定候选与合并门禁复核

2026-10-14。Mac按用户分工继续后台。此轮以原TASK182/IMP-20与当前候选aae2a3fe复核收口状态，生成逐门禁清单及只读Git冲突处置证据；没有修改生产代码、候选制品或原TASK，也未执行原生/性能/安装测试。

[当前执行清单](../../分析/07-固定候选验收与合并发布门禁.md)为继续实施与设备验收入口，不是稳定核心事实、完成证书或发布批准。原性能失败和所有门禁保留。

## 证据审计

原TASK3878字节/SHA bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93，provider校验revision2/in_progress/pending；9个AC binding与当前DMD逐一重新算digest全部一致，未重新造TASK。23份绑定原task_digest的封存RCP校验通过，acceptance_results总数0；其中20份包含原performance failed，其余七项整体验收名没有matching gate receipt。缺整体验收回执不等于没有历史局部测试，本清单不撤销局部证据，也不据此创造通过。

固定候选feature/0.1.10-integration-candidate@aae2a3fe、Windowsrun38524086177/artifact11875775415及包摘要继续使用上一封存[候选证据](../20261014-aae-存储修复双CI与Windows候选/README.md)。本轮只读取、记录选定证据路径/摘要和范围；没有复制源码到docs、重跑CI或候选。

事件注册统计当前8 triggers/39 jobs/81 events/27policies/87emissions/8scheduling/9UIfeedback/12paths/4cleanup。79active事件都有source登记，backup-completed/migration-completed两事件与generic sync/backup/migration jobs保持planned。解析source约定后87登记点文件都存在，原生emit与publisher词法入口另留原搜索结果。文件存在、配置数量、已有33项catalog测试不能证明实际所有GUI/CLI/观察器/事务路径都完成终态投递或恢复，event-coverage继续未闭合。初source路径推断缺ui/src和扩展名，已纠正，不判产品缺陷。

## Git集成模拟

四个PR均open/draft，main仍eae85322；PR1headbe/base main，PR2headfd/base focus，PR3headbb/base details，PR4headaae/base icons。五共享文件在main/focus/details/icons/candidate和docs内容一致，candidate对main差异不含docs/.adg。实际用户脏工作区和两个未跟踪协作探针保持。

merge-tree(main,aae)无冲突且返回tree d9d894fbad25c4c29e041b921ccf255f6034766e，与候选整个tree相等。merge-tree(details,icons)无冲突；merge-tree(details,aae)在commands/about.rs content和about_query_ownership_tests.rs add/add冲突。原因是fd被以等价补丁cherry-pick到bb系，而details独立仍含fd，后续a7改过同两文件；不是功能已证明失败。

用外部独立GIT_INDEX_FILE读取冲突树，仅将这两路径替换为aae的已测blob，write-tree再次精确等于整棵aae tree；其他文件无需处置。保留最终独立task owner、持久终态/错误合并与回归，不能把冲突解决回fd旧版本。未创建merge commit、移动HEAD/branch或改真实index；模拟不是自动合并授权，未来合并新SHA仍须CI/制品/门禁。

详见integration-audit.json与三份原merge-tree日志；不以模拟tree相等宣布main已包含焦点/图标/说明/新修复或正式发布可行。

## 不关闭的缺口

八项required_checks在current-gate-matrix.json逐项列出现有局部证据、当前缺口及证明条件。完整macOS/Windows原生、安装/数据保留、真实更新/备份/事件、缓存原始计数/周期/配对性能仍开放。损坏面板保护关联的重试目前仍安全拒绝并保留备份、未有已证明受控修复出口，此项未被成功happy-path或保守阻断等价替代；实现/异常处置与设备证据须继续补齐，不擅自解除。

系统菜单还原旧子集不抵caption实际点击归属，旧背景光后台静态实验不抵原CPU/RSS因果或性能。没有新Windows现场状态，唯一实例最新真实存活/结束须Windows确认；Mac原生必须先协调Windows实例0。FZ50未许可，不执行；原预算/日常0.1.7/真实数据/分支归属不变。目标active，原TASK不改，RCP总体failed，不写验收结果/核心晋升/证书或合并签名发布。
