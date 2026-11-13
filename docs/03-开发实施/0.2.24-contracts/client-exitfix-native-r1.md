# 参数读取修复后的原包双轮 r1

revision: 1；2026-11-13；IMP-34 P24-04；干净main382cf593fd24e1c957690bfaec7e4f0545ad8e1b；accepted=false，整轮失败。承接 [参数读取复核](client-process-snapshot-r1.md)。

| 关卡 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 7937ms，通过 | 10131ms，通过 |
| 同连接验签Status5 | 231ms，通过 | 847ms，通过 |
| 完整正常Stop10 | 6205ms，通过 | 11428ms，失败 |
| 独立状态 | RootAbsent9 | 当前RootRetirementPrepared19 |

25个关键源／tree与实际release库宿主958302401dd8bc38b4421ec468f7e94f44449676f4998509eb73514344202e30核对，构建6m42s。官方2.99／descriptor／bundle摘要不变，内核沙箱实证日常HOME／外部网络拒绝，loopback可用。依开跑前假设只运行一次exact原包双轮，不重跑同源失败；测试库不是固定app／安装器或Windows最终候选。

## 结果与错误边界

上一轮首次参数读取EINVAL未再观察到；两代原服务／launcher均取得normalexit0和各4个已捕获TCP监听缺席证明。这支持修复后本轮已越过此前早错，不证明低概率竞态永不发生，也不代替两轮完整动作验收。

第二根确已移动到计划唯一archive、Retire-readback存在且摘要正确，独立RootAbsent18曾提交；随后close_root_absence在write返回后的原AuthorizedReadback复核失败，按原代码追加由RootRetirementPrepared17仅变revision得到的pending19。原18个事件与新增19都保留，当前completionSha256=null、状态pending，不能仍称两个已关闭历史。此分支可由证明到期等拒绝触发，日志只有client_original_session_unconfirmed、产品内部10942ms，未单列底层证明拒绝detail；可确认总10s超限和post-close复核失败，不擅自填成某一具体原生字段异常。

新提前落盘的Stop DTO保留真实Stopped观察、result=null／client_original_restore_pending、门槛关闭，测试原断言未放宽。第二代distinctRootAttemptPlanAndPid gate在source中仅current Head为RootAbsent才计算，本轮正确为false；离线可核不可变前18事件中的两组ticket／rootidentity／plan不同，但不将这个事实回填为原gate通过。

## 协调与登记计量

取得5段协调tail：四段属于两次Start，唯一Stoptail属于首次Stop，lease末验1400us、正常release426us、完整程序来源重读488578us，开始余4296068us、结束仍余3805662us，通过原deadline。第二Stop的callback在root-close已失败，后续tail日志未到达，不能拿Start或首次Stop的数值代替第二次。

退役登记首代172.402ms、第二代721.603ms，第二代48frame/archive和历史3/3/6保持。此次失败主要待定位的是close_root_absence同次source／candidate／postcommit和当前已归档代次认证，尚无这些区段的真实完整计量；不从登记改善或RootAbsent临时提交推断整个收尾已足够。下一步核查同一观察内重复归档／帧取得，保留所有事务边界、每帧archive前来源门槛、当前根缺席及readback和post-close原证明／deadline；不能删除最终证明、借预算或将late关闭当成功。

## 证据与清理

证据 [.adg/evidence/0.2.24/client-exitfix-native-r1](../../../.adg/evidence/0.2.24/client-exitfix-native-r1/manifest.json) 保留开跑前假设、源码／build／binary、全部原日志压缩、准确Stop DTO、19事件／原79文件、两代Original/Ready/PreStop/Stopped／scope／provenance／Plan／readback摘要链、首代binding/raw9events/vault保持、strict失败verifier和提取说明。提取曾误要求仅RootAbsent才计算的distinct gate为true，已按实际false核验；原不可变资料直接核验不改运行失败判定。

三个准确新协调文件先留原字节、核first-seen identity和exclusive释放再清理，globalL原inode／保护／bytes保持并释放，未重建恢复。PID+birth／exe和两导出主端口独立内核快照缺席；79原文件逐字归档后清Profile／程序／test exe，再将全部材料压缩解压逐字核对后清临时证据根，工程target和所有旧失败保留。

并行只读审查形成了下一项动作canStart与status容量资格一致性的最小方案：复用close同次postcommit完整认证的私有projection，保留真实Stopped成功结果、容量不足只关门槛、不加预算外fullscan、Start仍独立核验。它尚未实施／验证，不当完成。根中断／缺Ready／一般客户端图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选等IMP34完整范围保持。未改日常配置／Shell／已安装app／官方包或真实业务请求，未发布，目标active。
