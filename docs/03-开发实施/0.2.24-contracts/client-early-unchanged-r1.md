# 早期中断且完整原件保持时的显式收尾核心

revision: 1；2026-11-12。归IMP-34 P24-04。代码main97bd68a53，启动失败回执门修复main590536842。本批accepted=false、非冻结候选；仅新增原生unchanged.review/finish及独立终态/已知phase展示文字，尚无对应产品IPC/确认按钮，不冒整个早期中断恢复已完成。

## 适用条件

只处理Captured/Ready/StopPrepared/Stopped/带独立原件引用的Failed，且restorePlan/completion引用均缺席、当前完整Codex/Grok原字节/原缺席与安全属性仍匹配Original的情况。Preparing没有独立引用、原件不完整、HOME/目录代次不符、客户端被改写、未知暂存/链接/不可读对象，均拒绝保留；不能新拍原件或从扫描结果收养未登记文件。

StartupSession及Originals按原scope/live/frames独立摘要核验；计划尚未形成不等于文件没变。当前完整树必须逐项匹配Original，沿既有精确两锁有限例外；两锁在最新独立frame中的存在/身份还必须与当前一致，不能接受后来陌生创建或替换。对Original原本有锁仍核对其身份。两锁可能0/1/2个：缺席项使用descriptor-relative lstat确认缺席，不创建文件/SQLite或把未读当不存在；已存在项原生pin/VFS核对、EXCLUSIVE真正重获/正常释放，两侧重读确认缺席项未出现，全部原始权限保护保持。

新独立finish动作从入口原10秒计时，Index.open_existing绝不初始化缺失存储，排他动作锁内复核attempt/revision/整树确认摘要。实际完整官方镜像/当前选择准入后，当前HOME完整OS归属必须Stopped、runtime/pid记录缺席；有独立Live时还核对原service/launcher PID和port缺席，不重建过去正常退出证明。完整树/锁释放读回后再核对当前OS/镜像，任何未知保持失败。全程不调用Pending/apply、不写客户端配置、不删除原件，不自动启动。

## 独立终态和证据

Unchanged不可与Restored文件补偿混同：它证明无需写回，原件当前已完整保持。新不可序列化Proof绑定原Head/数据根与客户端原生身份/原期限，只有真实读回可签发。create-only unchanged-readback-<原revision>.json记录原Head、完整两树摘要和可选锁身份；已有同名回执必须逐字相同，不覆盖坏值。Index.close_unchanged核对原Head CAS、独立回执引用与保护/原scope及Live，append新事件后才变Unchanged。若最后持久提交超时，追加pending事件回原阶段、保留原始引用；按新revision生成独立回执，不覆盖旧失败证据。

独立观察只读核对已登记回执/原件/scope/live/事件，缺失损坏Unknown。不重查关闭后合法用户编辑来触发旧文件回放；合法Unchanged与Restored一样使新Status为NotRequired，reserve_start只有独立验证闭合回执后可开新ticket，旧events/原件继续保留。没有旧版本目录或格式迁移。前端类型和phase文字增加“原件保持，原会话已结束”，本批未增加交互入口。

## 新发现的失败回执门

真实捕获中断原结果为StartingFailed却canStart=true/clientRecoveryRequired=false；实际持久Captured阻止了下一次执行，但界面回执与持久限制不一致。590536842在管理原件的Mac失败动作结果分支只读观察独立索引：pending或读取Unknown均设置clientRecoveryRequired并关闭三动作；无索引/已合法闭合或Ready仍按既有状态。不增加Stopped事实、正常退出或端口证明，原StartingFailed/失败原文保留。DTO新增反例及真实Index busy/坏回执/闭合状态函数检查证明此逻辑；未为这个回执门修复无假设重跑服务，早期原生原回执仍保留其旧canStart=true，不能改写历史测试为新修复已原生验收。

## 验证结果

65去重后端：session24（含0/1/2锁、外部编辑/未知stage保留、坏回执Unknown、合法关闭后用户编辑保持、Preparing无绑定拒绝以及Ready/StopPrepared/Stopped/Failed各独立引用保持）、index23、completion7（可选锁零创建/后来出现/消失拒绝和原两锁完整原生预算）、failure DTO11。首编译私有可见性错误early-test.log保留，修为仅原件模块可见；没有扩大系统权限或移除检查。最终格式、编译及全目标严格Clippy过，前端44项/类型/生产build通过；仅类型与已有phase文案增量，不另建界面原型或重复浏览器验收。

一个新隔离Profile供应原样官方2.99，在Original+scope独立登记Captured后、官方服务启动前注入中断：Start3283ms、serviceStartedfalse/recordnull，Codex/Grok未改变，index2 Captured。测试专用分支明确立即结束，不继续Status/Stop或失败后拍新原件；原完整测试exit101/failed与原index2保持，不能算普通Start成功。

另进程原生finish：错revision/digest拒绝索引不变；外部config编辑导致review无可用确认/finish拒绝且保留字节；合成runtime记录即使无服务也拒绝保留，测试自行恢复合成fixture后取新确认。正例2415ms完成Unchanged revision3；重复旧确认拒绝。真实OfficialStatusSource重开NotRequired、无PID/port、仅canStarttrue。独立只读原生完整Codex/Grok快照摘要匹配已登记回执，原index2与新index3/3events、Original+scope独立SHA、5内容blob及最终7路径原字节/缺席均核对。

原生成功使用build-native源，最终额外回执门修复及测试由独立source-final记录，不把后加源码冒为早期binary；本批没有新.app/DMG/签名候选GUI→IPC或Windows结果。原生引擎未接产品确认命令，需后继接线。

证据[本批目录](../../../.adg/evidence/0.2.24/client-early-unchanged-r1/)及manifest保留首编译错误、原捕获失败/旧门值、独立index2/index3、阶段输出/实际binary/source摘要/原内容blob/完整树/真实最终状态。OS sandbox拒绝外网和日常HOME，仅允许loopback，无业务模型请求。自有进程缺席、无服务监听代次开始，2个实际创建的新协调DB经EXCLUSIVE重获/释放后和完整自有程序/Profile/原件/测试副本清理；预列第3个DB未创建，不冒3个清理。既有globalL inode/字节/模式/owner保持，可排他重获释放且无sidecar。unit TMPDIR空，归档逐字核对后删除重复临时根；工程target/dist保持，无日常/官方/系统或业务配置变更，不发布。

## 后继

接本分支显式产品确认入口；仍需处理Preparing缺引用/不完整捕获、早期真实客户端已改写但未形成plan的恢复，以及首次缺根/一般客户端与路由图/完整进程集合/旧revision4首错、Windows贡献和集成、NAME、复杂System代理、官方交付验证/README及固定候选/W2。不能将“完整原件当前不变”窄分支扩展成“所有早期阶段均已支持”。目标active。
