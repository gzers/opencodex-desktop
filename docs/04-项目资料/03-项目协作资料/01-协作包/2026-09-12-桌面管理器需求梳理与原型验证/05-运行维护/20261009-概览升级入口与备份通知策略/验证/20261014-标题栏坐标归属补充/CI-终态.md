# 最新坐标归属工具 CI 终态

2026-10-14，Asia/Shanghai。feature/0.1.10-caption-diagnostics@5a36cda04018fb367c2f77e3a2a1bdc22d991e03 的 CI38431616359 已 completed/success：release-tools、windows-smoke-evidence、backend、frontend、backend-windows 五必需 job 均 success，main-only build 为 skipped。head SHA 与远端工具提交一致。

CI 的 Read-only caption observer portable contracts 实际运行16项，日志 Ran 16 tests / OK。ci-5a36-contract-excerpt.log 是原完整日志按 step 名称筛选的25行衍生摘录；原日志884610字节/SHA256 d2edcfb17053f1d77c3a31545af9df08727f24c8bab0cbf8b343a0f19314c882 保留外部，来源见 ci-5a36-results.json，API终态见 ci-5a36-final.json。原进行中快照和旧工具CI结果不改。

本次是test/CI工具变更的完整后台回归，16项仍为fake Win32合同；没有在实际be实例运行新WindowFromPoint工具、发送原生输入或制作/安装新候选，不证明caption还原或Windows API绑定实际通过。PR#1保持草稿，main/eae未纳入焦点修复；工具源与实际应用制品分别绑定。

下一步Windows在现有be候选核验身份后后台运行新只读工具；取得新的人前台信号后，再联合记录实际物理坐标、WindowFromPoint归属、父窗口命中与点击后状态。全程一个隔离实例。Mac原生和配对性能仍须先协调Windows实例数为0。原性能failed、原预算/门禁、TASK revision2/in_progress/pending与空验收保持；不迁移真实数据、替换日常版、自动合并、签名或发布，不执行未许可FZ50脚本。
