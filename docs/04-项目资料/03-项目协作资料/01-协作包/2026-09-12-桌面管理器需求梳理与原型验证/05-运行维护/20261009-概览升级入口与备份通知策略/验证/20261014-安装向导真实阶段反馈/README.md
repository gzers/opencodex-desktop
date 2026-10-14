# 安装向导真实阶段反馈

2026-10-14，Mac仅后台。代码feature/0.1.10-integration-candidate@f545cb3317b2c935ae07c8bca1674d3acf818552；父提交aae2a3fe。按IMP-20§6、W1/AC15/AC17核查设置安装向导，发现其仍将后端兼容percent阶段常量直接作为工作量显示；更新中心面板流程已有不定进度，但安装向导旧代码在installing/verifying等显示45/88%，失败statepercent=100仍渲染“安装失败 · 100%”，并在本地命令结果/来源最终核对前显示done。这不符合现行阶段反馈合同。

## 最终行为

SettingsRoute的向导不再消费percent作为已完成比例。处理中保留真实阶段、命令输出/离线步骤和可取消入口，用可访问不定进度条；done信号到达但事务仍进行时显示“正在核对安装结果”，只有installing=false、实际installOutcome存在、无installError且phase=done才显示完成条。失败保留原因并停止进度条，不显示100%；重启后生效和保护关联仍待核验分开保留，不把包安装完成说成代理已运行新版本。

不定条只表示未知工作量，aria-valuenow缺省、aria-valuetext给阶段；实际完成时aria-valuenow100，状态文案无阶段百分比。动画复用已有EffectsStore.strategy.animated：高档且窗口可见/前台才滑动，中低档及后台静态条，恢复前台按同策略恢复。不引入system减少动态、计时器、网络或新偏好。base.css复用现有progress-slide，完成无动画；后端兼容percent字段/安装协议、脚本许可与事务门禁不改。

## 红绿与范围

初9项挂载回归在旧源码9失败、新3文件共22通过（向导9、保护状态/面板进度旧回归13）。加入画质/可见性用例后，相同最终完整测试文件6246字节/SHA b6d08e591d247748e0825c8921e3ccbfb6787c8f8b012944522df45d5169f6a2，对父aae的两生产文件实际10失败；最终源码完整UI98文件/511通过，其中新增10项。受控mock命令停留pending以验证preparing/downloading/installing/extracting/verifying/activating/done，真实失败回调显示错误、done/outcome/finish顺序及重启/保护提示、三档/隐藏/前台策略均断言。相同测试红验证后finally恢复生产最终字节，提交blob摘要核验一致。

类型检查/生产构建、diff检查通过；4条既有动态导入警告、jsdom canvas警告保留。UI组件/状态测试不是原生视觉、动画性能或实际安装器验证；本轮只改SettingsRoute/base.css和新增测试，未重跑本地Rust、未启动任何原生实例/执行安装或FZ50脚本，没有新性能样本。

## 新旧SHA与CI

新pushCI38533772616、PRCI38533779767最后in_progress，原始run/head/event/状态分别保存。PR4说明/title已更新、draft/open/head=f545/merged_at=null；实际新CI检出/完整终态和新制品仍待核验。候选38524086177/artifact11875775415继续只归aae，不含本补丁；没有重用旧包证明新界面、没有派发第二轮候选。

main仍eae；新main×f545的只读merge-tree无冲突，返回整个f545tree，与源码一致；只作内容模拟，无merge commit或branch移动。原封存aae合并模拟tree仍归旧候选，未改原证据。门禁清单已补充最新源码/旧制品分离，之后若确认新SHA候选再做单实例协调验收，Windows目前可继续旧固定制品已有矩阵，Mac无原生操作。

## 保护记录恢复审阅与下一缺口

protection.rs严格拒绝损坏/无法判断关联，当前retry在反序列化/validate失败前不会取得保护变更guard，不写/删关联或解除备份。安装manifest的ActivationProof包含attempt_id及entry/package/bin摘要，却不包含PreferencesProtectionRef（backup_id/SHA）；仅凭当前安装版本/前缀或任意用户备份不能证明缺失关联，不能安全重建已验证安装或解除保护。当前没有可证明合法来源与精确备份的恢复入口，仍为开放实施缺口；本轮未新增强制清除/伪造关联或将保守阻断写成已恢复。

另一个后台排查线索：SettingsRoute.installPrefixError仍按POSIX根路径判断，Windows盘符/UNC路径可能在向导“下一步”前被错误拒绝。尚未做对应挂载红绿或真机验证，不在本提交改动/称通过；下一步按Windows真实来源DTO复现后修复，后端系统保护/写权限边界保持。

八项门禁与原macOSCPU+1.817629pp/RSS+117.008965MiB失败、冷热各20/100ms且5%/10min0.5pp和10MiB/50ms/缓存100切页10重启零自动查询/真实周期预算全部保持。原TASK3878字节/revision2/in_progress/pending、空acceptance_results不变，RCP总体failed。FZ50许可未获不执行，不真实迁移、日常替换、核心事实晋升、自动合并、签名、发布或关闭；Mac原生前仍先确认Windows实例0。
