# 完整属性鲜读原包双轮 r1

revision: 1；2026-11-13；IMP-34 P24-04；干净main4ba715120fd59ddbe3634e501856b436ec928f74；accepted=false，整轮失败。承接 [属性鲜读与校准](client-xattr-read-r1.md)。

| 关卡 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 8038ms，通过 | 10241ms，通过 |
| 同连接验签Status5 | 229ms，通过 | 819ms，通过 |
| 完整正常Stop10 | 6366ms，通过 | 11345ms，失败 |
| 独立根状态 | RootAbsent9 | RootAbsent18，已移动及读回关闭 |
| 完整动作结果 | Stopped | client_original_restore_pending；不放行成功 |

本轮有实际底层修复，依开跑前假设只跑一次exact原包测试。release库宿主构建6m56s，22关键源摘要与二进制1313d1ec98cf69a64c9f50e2e03c8eb0863c41fff0f2a64f7e59db7fba54f323已核；官方2.99包／descriptor／bundle原摘要保持。内核沙箱实证拒绝日常HOME／外部网络，loopback可用。库测试宿主不是固定app／安装器／最终Windows验收。

## 已证实与仍失败

第二轮真实退役登记657.561ms，对比上一诊断1053.745ms有明显局部下降；仍执行48帧／48归档与历史3/3/6，native xattr调用127965。第一轮登记171.226ms，24帧。两次运行样本不构成通用性能统计，更不能以局部改善代替完整预算。

与旧失败不同，本次第二根确已移动到计划唯一archive，Retire-readback摘要和completionSha256已写入独立RootAbsent18，closedGenerationCount=2。两代Original／Ready／PreStop／Stopped／scope／provenance／Plan／readback完整链、18份事件、首代binding与raw9events／旧vault保持均从79个原文件离线核验。第二轮只读状态和预览也通过其原门槛，不能抹掉这些已完成事实。

但完整Stop11345ms仍超原10秒，产品内部10577ms，在日志release-coordination阶段返回client_native_coordination_timeout；动作结果保留失败、canStart=false、clientRecoveryRequired=true，没有伪装Stopped成功。代码同源归因明确：with_admission_mode的operation已经返回，lease.check的超时码是expired，而本次timeout对应末尾检查，位于valid?／release?／完整program_selection::read_complete与selected比较之后。并非证明锁释放本身失败，而是协调收尾最终时间已耗尽。原预算断言、来源核验及所有旧失败保持。

下一步需分段核验这条协调收尾（当前lease复核、正常release、完整程序来源重读、最终deadline）和全动作余量，寻求不减完整性验证的修复；不能多借时间、跳过释放／程序hash或事后把已晚的动作晋升成功。RootAbsent的显式中断／重复请求及动作可启动资格规则仍待安全收口，不因磁盘恢复完成而删掉原项。

## 清理与证据

两轮launcher/service均正常exit0，各4个已捕获TCP监听消失；自有PID+birth／exe和两导出主端口独立内核快照缺席。三个准确新协调对象先留原字节、核first-seen identity和exclusive重获再删除；预存globalL原inode／mode／ACL／xattr／bytes保持且exclusive释放，未重建／恢复。79原文件逐字归档后删除Profile／程序副本／测试exe，再将全部诊断材料压缩解压逐字比对后删除构建证据临时根，工程target保留。

证据 [.adg/evidence/0.2.24/client-xattr-native-r1](../../../.adg/evidence/0.2.24/client-xattr-native-r1/manifest.json) 保留假设、源／构建／二进制、完整压缩原日志、所有状态与原字节、严格失败verifier、登记分段和同源收尾错误归因。失败verifier独立验证两根关闭事实并明确Stop10失败，不把成功verifier的断言放宽后当验收通过。未重复跑该失败源码。

Windows精确远端refs仍为代码b2e89665d／文档0eef59a01，无新推送，不推断对端未有本地工作。根中断／缺Ready／一般客户端图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定集成候选等原完整范围保持；未动日常配置／Shell／安装app／官方包或真实业务请求，未发布，目标active。
