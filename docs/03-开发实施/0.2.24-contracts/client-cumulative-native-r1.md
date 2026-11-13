# 累计事务与目录优化原包双轮诊断 r1

revision: 1；2026-11-13；IMP-34 P24-04；干净 main098ae4a15353ff92cb42d3bd78e393c9d6909f57；accepted=false，整轮失败。承接 [元数据边界](client-root-metadata-r1.md)。

| 关卡 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 8405ms，通过 | 13253ms，通过 |
| 同连接签名 Status5 | 233ms，通过 | 1164ms，通过 |
| 完整正常 Stop10 | 8638ms，通过 | 10888ms，失败 |
| 独立索引 | RootAbsent9 | RootRetirementPrepared17，无完成摘要 |

实际 release 库测试宿主构建6m54s，21个关键源摘要、干净提交／tree 与实际二进制 SHA256=1076a7ac010f1c66ef418d1539609049462677e1b7359cf4449f84025e0da95d 已核对。官方2.99包／descriptor／bundle沿原摘要，内核沙箱实证拒绝日常HOME与外部网络、允许loopback。只执行一次 exact ignored 原包测试；失败后不重试。库测试宿主不等同最终app的panic／安装交付验收。

第二轮原件恢复完成时剩1024696us；root-plan/index/registration之后在retire入口remaining0拒绝，产品内部Stop10202ms。根退役计划已独立登记source16/schema2，但根未移动、无Retire-readback／RootAbsent。此前12143ms失败和本轮10888ms失败均保持，不因较快就通过。第一轮和第二轮所有launcher/service均实际normalexit0，分别捕获的4个TCP监听消失；这些事实不替代完整恢复成功。

原binding、前9份事件原文和首代vault保持；两代Original/scope/provenance/Ready/PreStop/Stopped摘要链和第二退役Plan已从77个原文件离线重新核验。三个准确新N/K/history协调对象先留原字节、核first-seen身份和exclusive重获后清理，预存globalL inode/权限/ACL/xattr/bytes保持且exclusive释放，未删除重建或恢复。导出的两个主端口独立内核TCP快照缺席；所有观察到的PID+birth代次和自有exe均无残留。

证据 [.adg/evidence/0.2.24/client-cumulative-native-r1](../../../.adg/evidence/0.2.24/client-cumulative-native-r1/manifest.json) 含开跑前假设、源／构建／二进制、原日志压缩、整轮状态、原字节、独立verifier和清理回执。原始owned77文件逐字比较后删除测试profile、程序副本和测试exe；再次将全部诊断材料逐字归档后清理构建证据临时根。工程target保留，旧失败证据不变。未改日常配置／Shell／安装app或官方包，未真实业务同步／模型请求，未制新app/DMG/Windows候选，未发布。

下一步必须对当前真实root计划登记和retire/close剩余路径做完整分段计量，结合已恢复原件后仅约1s余量评估缺口；保留事务source/candidate/postcommit、逐帧archive前来源拒绝、原生身份／原件／namespace和原10s预算。不能重复无新假设原包轮，也不能只按合成计量判断足够。根中断／缺Ready／一般客户端图路径与完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／交付验证及固定候选等IMP34全范围保持，目标active。
