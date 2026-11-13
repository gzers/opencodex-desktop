# 累计目录扫描与资格接线原包双轮 r1

revision: 1；2026-11-13；IMP-34 P24-04；干净main287ee0c9d2a2048b83f8db54d945caedf02e29b5；定向原包双轮通过，accepted=false，非固定app／整版验收。承接 [目录扫描优化](client-generation-directories-r1.md) 和 [启动资格接线](client-stop-eligibility-r1.md)。

| 原门槛 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 7160ms，通过 | 9749ms，通过 |
| 同连接签名Status5 | 231ms，通过 | 785ms，通过 |
| 完整normalStop10 | 6016ms，通过 | 9572ms，通过 |
| 独立根关闭 | RootAbsent9 | RootAbsent18 |
| stopped status5 | 961ms，通过 | 1840ms，通过 |

28关键源、干净commit／tree及实际release库宿主9639b54d824b4b64637076d1d8b0d02cda64bacdbd8846e5474b96bac12a5b4d核对，构建6m38s。同原官方2.99、descriptor、bundle摘要未变；内核沙箱实证日常HOME和外部网络拒绝，loopback可用。开跑前假设后仅一轮exact ignored用例，exit0、唯一双轮completion marker、全部原断言通过，不重试同源凑结果。

## 当前通过范围

两轮actual service／launcher正常exit0，分别4个捕获TCP监听消失，客户端artifact／完整根原缺席恢复及Retire intent／readback完成；current RootAbsent18、closedGenerationCount2，18份独立事件与两代Original／Ready／PreStop／Stopped／scope／provenance／Plan／readback摘要链通过离线重验。首代binding/raw9events和vault保持，两代attempt／根identity／Plan不同。两个成功Stop DTO均保持Stopped成功、无failure、canStarttrue，status和preview资格一致；后继Start独立核验，不采用先前DTO作权限。

第二代退役登记599.753ms，48frame/archive与完整history3/3/6保持；第一代174.489ms。首次Stoptail末验1268us、release455us、完整source480974us，结束余3995076us；第二次末验1140us、release385us、完整source523049us，结束余442132us。原Stop最终返回总9572ms，仅约428ms动作余量，不能推导所有硬件／任意大树／更多代次或并发压力都有充裕预算。所有先前超时、参数读取早错和pending demotion证据原目录保留，不改写为通过。

本轮只证明该固定源码／release库测试宿主在声明的两代合成客户端配置及真实原官方服务下通过正常定向生命周期。它不是最终产品app的panic=abort／资源布局／UI入口验收，也不替代Restart、根中断恢复、缺Ready／一般图路径／外部Agent／完整进程集合与活跃刷新、其它平台或整版发布门禁。

## 证据与清理

证据 [.adg/evidence/0.2.24/client-generation-native-r1](../../../.adg/evidence/0.2.24/client-generation-native-r1/manifest.json) 保留假设、build／源／实际binary、所有压缩原日志、原始状态／回执与79个原文件字节、登记／tail计量、未放宽的成功verifier及独立整组verifier。成功verifier沿用原严格两轮动作、原预算、status、proof和历史断言；accepted仍false只表示非整版验收，directedTwoCyclesPassed明确true。

三个精确本轮新N/K/history协调对象先留原字节、核first-seen identity／exclusive重获释放后删除；预存globalL原inode／mode／ACL／xattr／bytes保持并exclusive释放，未重建/恢复。tracked PID+birth／自有exe全缺席，导出两主端口独立内核TCP快照缺席；79原文件逐字核对归档后删除Profile／程序副本／测试exe，全部诊断材料再次压缩解压逐字比对后清临时证据根，unit TMP为空，工程target保留。无日常配置／Shell／安装app／官方包修改、真实业务同步／模型请求，未发布。

## 下一步

结束针对同一正常双轮已通过项的无依据重复运行，转入原计划未完：先接“根已退休／读回已存在但metadata因旧动作超时仍pending”的显式收尾。这必须用新的用户确认、当前完整program／归档／Grok／缺席／锁释放认证和独立recovery proof，不能延长旧normalStop证明或把原失败晋升成功。只读实现审查已给具体方案及前后端缺口，留在retired-metadata-finalization-review.json；尚未实施／验证。root未移动的continue／rollback、缺正常阶段证据和新创建根取消保持各自原风险边界，不能套用已退休收尾。

NAME、一般客户端／路径与完整进程图／活跃刷新、复杂代理、WindowsW1/W2／CLI安装更新卸载接线、官方交付verifier／README及固定app/DMG/MSI/NSIS候选等IMP34完整范围仍未完成。Windows最终提示词只在固定集成候选达到P24-05后交用户，不因本次正常双轮通过提前放行，目标active。
