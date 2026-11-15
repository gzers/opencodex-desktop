# Windows双根阶段与退休工作树持续读回修复

沿TEAM14/IMP140/windows-security-coverage-r4允许写集，消费main2ebdd3ae0/docs84ec482ee（本轮fetch仍同SHA）。代码分批推2b2e3a9c0与40203e193122395c1cef05b24b10643f42ebfd73，最终tree e61a902dfab26eafc030d6bba2a640965da00bf6。仅Windows原件tree/v2、定向测试及其build选择器；共享trait/官方包/安装器/Vue/日常配置未改，既有untracked windows-schema.json留原状。

第一处缺口：原TreeProgress在seen=false时放过没有双根步骤的Ready及后继纯阶段链。before01两项新回归在旧逻辑真实失败（101/0过2败），原生路径确实写出phase2 Ready，但一直是PendingOnly记录，没有native许可/官方Start。修复后Ready/PreparedRestore/OfficialTeardown入口必须双根均完成MoveToCustody/CreateWorkingCopy；Restored/Closed必须双根均完成RestoreOriginal，未完成意图仍拒绝。完整synthetic阶段链也不产生ClosedEligibility。旧数据夹具改为合法Captured→Aborted，原真实partialwrite+sync故障、同对象CAS、原ID/poison/阶段保留断言均保持，不靠非法Ready测试它们。旧不完整v2记录拒绝而不转换/迁移，Full/Partial语义未变。

第二处缺口：原RetiredWorking仅存路径，working=None后最终读回不再核退休工作树；retired-before01的根级和canonical两项回归均真实失败（101/0过2败），原件还回但退休未知新文件被遗漏。修复后保留确切退休目录原句柄、私有父目录原生pin和完整工作像，rename后立即安装，后续原件/恢复/阶段读回重复核全树ID/内容/8投影/枚举及父身份。任一退休读回错误设置poison，保留原件/custody/退休现场；不删未知文件、不修改ACL、不引入第二journal。重复观测不证明所有外部间隙写入都不存在；声明范围内观察到漂移必须拒绝。

| 执行 | 原退出 | 结果 | 耗时ms |
|---|---|---|---|
| before01 | 101 | 0过2败，证伪未执行双根也可Ready | 136692 |
| after01 | 0 | 15过 | 91080 |
| retired-before01 | 101 | 0过2败，证伪退休工作树被忘记 | 108741 |
| after02 | 101 | E0308编译失败，tests0，无新EXE | 15866 |
| after03 | 0 | tree/v2全23过 | 136710 |
| product01 | 0 | clean最终MSVC custom-protocol debug全部bins；lib68warnings | 117606 |

after02是旧缺席根测试仍把新RetiredWorking当PathBuf传给open；改为直接读保留root句柄，原work_id相等、原缺席和禁止第二working断言均未减。原错误source/log/result及无新制品说明保留，不给缓存EXE冒本轮摘要。其余4次真实执行EXE分别已在覆盖前记录摘要。首错before01 raw101，不虚填Win32 API码或把预期证伪追溯改pass。

最终product01的96份执行输入对工作字节一致、Gitblob行尾等价，after03的52份Rust输入对最终相同；详见final-source.json。Desktop77124096/SHA2562488b2838d825ce9ac767df72d067ed3bde5e1942e9f40730bb879eb93e83a77；ocxd5398528/SHA2563ebd15d0c0252cc675823dba642811455d7793911f36199c217c4748fffa57d9；ocx306176/SHA2562c5a5f1040086f35b6bceac464c700378a58602a888d3531415f48c148ebe6b5。当前前端dist32份/1162800字节与上一GUI准备轮核对相同；未重跑未受影响前端/MSI/CLI/旧Full1314/B2权限失败。不是release预算/冻结候选/发布。

16个失败/中断/预期拒绝场景共129文件311020字节已私有ZIP内容核验，原现场保留；ZIP不克隆NTFS安全/身份。正例guard清理自身对象；无新账户/Profile/Public/GUI/官方服务/force/日常PATH配置策略变更。全部旧自动审批删除拒绝未重试/绕过，overallCleanupComplete=false。完整状态矩阵和所有限制归final-status.json，离线verify仅证明字节与声明一致，不是native或产品验收。

真实GUI的C:/Users/Public/ocx-h024-actual-gui-20261115-r1/三输入及原0c来源保持不可变；当前新EXE单列，不覆盖该入口。观察out01仍空，等待marenben原桌面启动，控制器未代跑。该入口即使过也只证明实际GUI/空broker组件，官方Owner/同最终source生产组合还须另验。原件官方声明副作用与持久Ready/PreStop快照、后继历史/旧根句柄释放、真实GUI官方broker及S/D生产完整组合、普通MSI和历史清理仍未完；公共Start/后继拒绝不变，allAccepted/productionExecuted/actualGUI=false。B2 CAP/trust非空前提1314/5保持，不提权/改策略重试。原30/5/10/Restart30含Stop子10保持，无业务同步/模型请求/W2/双SID/发布。
