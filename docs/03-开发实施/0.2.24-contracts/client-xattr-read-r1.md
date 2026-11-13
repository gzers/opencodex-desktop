# 完整扩展属性鲜读与四帧历史校准 r1

revision: 1；2026-11-13；IMP-34 P24-04；main4ba715120fd59ddbe3634e501856b436ec928f74；accepted=false，非固定候选。承接 [真实成本分段](client-partition-native-r1.md)。本批模块证据没有官方服务轮次，不能宣称 Stop10 已修复。

## 修复与安全边界

普通小扩展属性／名称列表在固定256字节以内直接鲜读完整值，只有返回长度严格小于实际buffer且不超剩余MAX_SECURITY预算时接受；ERANGE或buffer填满只进行一次查询大小＋精确回读。查询后超限在分配前拒绝，最后长度不等于查询长度或其它错误都拒绝，不重试为新基线，不把不可读当缺席。剩余预算0仍用非零buffer实际读空值。全部属性名及值合计上限保持，NUL结尾／重复key规则保持。

初始方案对全部key依赖ERANGE，虽然普通user属性测试通过，但只读审查发现P1：HFS／AppleDouble的ResourceFork可能成功返回前缀。该初始方案未提交。最终对 `com.apple.ResourceFork` 始终保留query size＋exact read，并对普通filled buffer也回退。原生ResourceFork 1/255/256/257/1024/4096字节完整相等、仅尾字节变化必须区分、MAX_SECURITY+1拒绝均通过；模拟stream返回filled-prefix及超remaining仅查询即拒绝也通过。

原query/exact方法本来就无法持续监控查询后并发增长，最终实现不宣称解决这个原有界限；也不宣称两种算法在所有瞬态采样时刻有完全相同轨迹。没有跨调用缓存、裁剪／忽略ResourceFork、跳过ACL/xattr或任何来源校验。原Protection、ownership/mode/flags、每帧来源／归档／namespace、三事务边界和deadline全部保持。独立补审针对实际修订解除P1、无剩余阻断，未执行测试或服务。来源依据与审阅限制在review-p1/final中保留。

## 夹具校准与收益

新增完整首代历史夹具：Original／Ready／PreStop／Stopped四帧和restore-plan登记后退役，独立Head关闭在9；第二代Stopped15、restore-plan16、retirement17。合成状态／移动seam不构造真实正常Stop或生产移动许可。原Captured轻夹具及既有存储反例保持，不回填旧计量。

四个独立同形完整历史夹具按旧／新／新／旧对照，明确断言48帧／48 archive、provenance61、namespace60、Protection61744、capture339、完整历史begin/finish各3及内容双遍6均相同。扩展属性native calls 187405→103631；登记耗时766.871／494.321／492.963／774.577ms，约36%局部收益。普通自有文件小属性对照6→3 native calls，内容与后续变化均重新读取。合成树仍不同于真实运行tree／writer marker，不能据此直接放行完整Stop10。

## 验证和证据

原件模块305项通过／9原有忽略，协调模块27项通过／6原有忽略，版本化命令4项通过／1原有原包用例忽略，共336项不同受影响测试。先行六项定向结果已被完整模块包含，不重复计数。全部通过后仅保留严格all-targets Clippy、fmt、diff-check已通过的结果，不扩大到无关特效或Windows真机。

证据 [.adg/evidence/0.2.24/client-xattr-read-r1](../../../.adg/evidence/0.2.24/client-xattr-read-r1/manifest.json) 保留开跑前假设、初始普通属性通过但不覆盖ResourceFork的日志、P1及修订、完整六项与模块日志、校准计量、四个变更源和测试二进制摘要、重验脚本和清理回执。unit TMP为空，全部原字节压缩解压逐字比较后删除本批临时根，工程target保留。无日常配置／Shell／已安装app／官方2.99或真实业务请求变更。

后继已固定本提交开始release库测试宿主构建，再以同原官方包和原Start30／Status5／Stop10运行唯一新双轮验证，带真实登记分段和完整移动／关闭／原件／监听／清理证据。开始构建不算通过；此前11681ms失败与较早失败保持。根中断／缺Ready／一般客户端图路径和完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／官方交付验证与固定候选等IMP34全范围不缩减，目标active，未发布。
