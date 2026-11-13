# 原包根退役登记真实分段 r1

revision: 1；2026-11-13；IMP-34 P24-04；干净main264934c0936a1fab39e00d36c228b4813573b24b；accepted=false，整轮失败。承接 [分段入口](client-retirement-cost-r1.md)，本轮只增加诊断，未做性能修复。

| 关卡 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 8129ms，通过 | 12812ms，通过 |
| 同连接验签Status5 | 226ms，通过 | 1231ms，通过 |
| 完整正常Stop10 | 7604ms，通过 | 11681ms，失败 |
| 终态 | RootAbsent9 | RootRetirementPrepared17，根未移动 |

release库测试宿主构建6m20s，21关键源摘要与二进制9a5eb08ad9ff15ccd57a76eeb9e97b9a16374ab99990b1efeb4d507914fe2971核对。官方2.99和原资源摘要不变，沙箱实证拒绝日常HOME／外部网络，loopback可用。只执行一轮exact原包测试，不重试。该测试库不代替最终app／安装器，诊断accepted:false。

## 实际登记分段

| 互斥区段 | 首代 | 第二代 |
|---|---:|---:|
| 总计 | 257.786ms | 1053.745ms |
| 来源完整认证 | 48.156ms | 278.369ms |
| 同事务计划准入 | 41.183ms | 84.635ms |
| 候选完整读回 | 82.238ms | 338.963ms |
| 提交后完整读回 | 83.122ms | 345.650ms |
| SQL写入＋COMMIT | 0.721ms | 0.532ms |

顶层覆盖99.9%以上；其余为storage+BEGIN和最终登记事实，未归属19.042／22.126us。第二代来源／候选／提交后合计962.981ms，才是主要登记成本。嵌套Protection868.906ms、live-prefix236.018ms、历史内容双遍495.298ms、provenance544.220ms彼此交叉，不能加总。当前Tree/Grok嵌套只有3.837ms。本轮证伪“主要耗在SQL提交”的方向，但尚未验证任何底层修复的收益。

第二代原件恢复仍在预算内，finaltime后只余288171us，登记完成后retire入口剩0拒绝，产品内部Stop10996ms。计量会增加诊断开销，本轮并非证明产品性能变好或变坏；前后轮失败各自保持。两轮launcher/service实际normalexit0、各4个已捕获监听消失，不算完整正常Stop成功。

## 夹具校准结论

此前合成Stopped第二代登记读30帧，而真实正常历史读48帧。差值明确来自第一代历史：fixture_closed_generation以Captured就退役，只有Original一帧；真实首代正常Stop保留Original/Ready/PreStop/Stopped四帧。六遍完整历史内容观察各差3帧，6×3=18。后续必须用完整四帧已关闭首代校准合成性能夹具，不能继续拿较轻夹具预测原包预算。原轻夹具仍适用于其声明的存储边界，不改写旧记录为完整正常Stop证据。

后继优先评估不减检查的底层鲜读优化，例如扩展属性小值直接有界读取、ERANGE有界回退；当前仅候选，未实施或验证。三事务边界、逐帧来源／ACL／xattr／namespace／原件／原生归属以及Stop10不变。若优化不能覆盖实际缺口，应继续按完整真实读回链分析，不能只省掉最后一次错误或重新运行同源凑成功。

## 证据和清理

证据 [.adg/evidence/0.2.24/client-partition-native-r1](../../../.adg/evidence/0.2.24/client-partition-native-r1/manifest.json) 保留开跑前假设、源码/构建/二进制、全部压缩原日志、两个完整状态、原77文件字节、逐阶段计量与独立verifier。离线提取首次因首代没有历史指标而KeyError，已明确首代历史零／次代begin3/finish3/fullpass6，不重跑native或放宽失败条件；提取说明保留。

两代原件／三phase链、首代binding/raw9events/vault保持及第二Plan已离线验证。三个准确新协调对象留原字节、核first-seen identity和exclusive释放后清理，globalL原inode／Protection／bytes保持并释放，未重建/恢复。PID+birth／自有exe及导出主端口独立缺席；原77文件逐字归档后清理Profile/程序/test exe，全部诊断材料再次逐字归档后清临时证据根，工程target及旧失败保留。

并行只读核对确认停止动作canStart与状态查询的后继容量规则仍待统一，不能在原预算外追加完整扫描或永久禁用真实Start凑一致。WindowsW1独占贡献、W2固定候选、根中断／缺Ready／一般图路径／完整进程集合与刷新／NAME／复杂代理／交付验证等完整范围保持；无日常配置／Shell／已安装app／官方包或业务请求变更，未发布，目标active。
