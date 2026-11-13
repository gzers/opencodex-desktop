# 0.2.24 客户端恢复的官方写者锁覆盖

revision: 1；2026-11-12。归IMP-34 P24-04，承接[原生协调](native-client-coordination-r1.md)、[父与服务退出](macos-launcher-exit-r1.md)。main2f3c7dbfd；accepted=false、非冻结候选，MutationPermit与实际apply仍未开放。

## 源码反证与实际缺口

从固化bundle中读取官方tgz，SHA256 b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c精确匹配；只在自有临时根展开src用于审计，不修改官方包。K的activePermits运行时登记负责catalog/cache/audit写入，回调结束先撤销permit再释放锁；但client/connect.ts的连接、同步和断开路径还会直接atomicWriteFile/unlink默认catalog，这些操作受withClientLifecycleSync的用户级L及当前HOME的C保护，并不取K。因此“持有claim/N/K就排除全部官方目录写者”被源码反例推翻，不能只增加PID缺席就签发许可。

当前产品NativeLease缺L是具体待修项；官方L位于effective OS-user runtime root/client-desktop-lifecycle.sqlite，独立于HOME/OPENCODEX_HOME。N→L→C、N→K→C及L→K→C的已有顺序要求，采用claim→N→L→K，不从K反向取得L。L按OS用户区分；同一用户不同HOME的相关连接提交会短暂相互拒绝busy，不改成无界等待或服务抢占。

## 产品修复

Desktop helper只导入原样官方client/lifecycle-lock.ts，在已取claim/N后进入withClientLifecycleSync，再进入K；安全预检查覆盖L及原有锁/sidecar，坏权限/链接不先让官方函数修复再冒通过。回执协议增为ocxd-client-coordination/2，固定四组路径/设备/inode；旧协议及缺L三组数据明确拒绝，无旧兼容读取。

原helper正常释放/exit0后，Rust按同顺序重获原四个inode上的SQLite事务，claim仍EXCLUSIVE、N/L/K仍IMMEDIATE且busy_timeout=0。NativeLease.check/RAII现要求四连接及所有pin/VFS/owner/上下文保持；回调异常或超时不会让外部定时helper提前解锁。更新不会给任意磁盘JSON赋权，原完整镜像前后核验和原动作deadline均不变。

该锁组证明所检查官方K/L回调在当前原生临界区内不能执行，不能证明非合作文件写者、历史逃逸进程/转移socket或任意API的完整性；原three-file恢复的竞争保全/冲突拒绝继续，不能以持锁代替文件归属/原件/正常退出及完整恢复判定。Windows平台未执行此Mac调用，不改其独占文件。

## 定向验证

17个相关函数通过：官方协议6、原生协调11（含辅助探针入口）；两个显式原包手动入口ignored不计通过。新增L忙锁、权限变宽和同字节inode替换反例，旧协议/缺L拒绝；四个数据库都通过真正另一OS进程的SQLitebusy探测。编译/格式/全目标严格Clippy通过。

新增test/official299-native/catalog-writer-exclusion.mjs以固定官方发行原函数做更强检查，不起服务/不连网络/不读凭据：

- NativeLease持有四锁时，withCatalogWriteSerialization的callbackRan=false/busy，catalog/cache/audit均不变；L下同client/connect的直接目录写入callbackRan=false/busy，均未执行。
- 释放后，同一新Profile中原K的catalog/cache/audit mutator和L的直接写入全成功，三项摘要均变化。故不能把错误路径/导入失败当排他通过。
- 伪造K permit拒绝；正常K回调返回后泄漏permit也拒绝。原官方源码/原生算法不降断言。
- 明确等待2100ms超过旧helper窗口，再检查原件和四数据库锁仍保持；原生操作4059ms过本入口原10秒，锁释放后可重获。此项不是普通Stop完成。

## 原失败与隔离边界

第一新Profile在L阶段拒绝；精确官方L函数于原sandbox复现client_lifecycle_lock_failed/EPERM。只允许既有0700用户运行目录file-write-mode后可取得/释放，产品代码与权限断言不改。首次诊断误猜包内Bun路径execvp ENOENT也留存，后继用真实dependencies/bun/bun核验两策略。

第二新Profile已取四锁，但test-only probe导入了不存在的src/config/index.ts，原模块导入错误保留，不能计writer被阻断；修为官方实际src/config.ts并重编准确binary后，第三新Profile才完成上述真实正反例。前两轮目录/摘要/日志与失败保持，不复跑旧现场。

原用户级L文件在测试前已存在，记录inode/字节/0600/owner及sidecar存在状态，测试只借用官方锁；最终独立EXCLUSIVE重获释放，并核对身份/字节/权限及其0700父目录均与原值相同，没有删除该旧文件或留新sidecar。没有以更改日常配置绕过L。新fixture各自claim/N/K只在测试根或精确自有新命名空间产生。

## 普通产品链受影响复验

最终准确binary21446da0ca008be971d12630fbaf2878ab716104b7d682560d9a35c948f091f8的17项与原包执行副本匹配。再用新Profile完整供应/核验同一官方2.99：Start8784ms、Status234ms、Stop4996ms（内部4992ms）；原服务和父normal0、4项捕获TCP缺席、四frame/独立索引RestorePrepared/revision6/plan/stage/35项内容引用匹配。重开collector360ms Stopped+Required、无PID/port且三门槛false。

原30/5/10不变，四锁新增路径在该样本未造成超时。catalog/cache/audit仍未实际补偿，原完整assert保持exit101/accepted=false；不把这轮原件计划已准备当恢复成功。没有构建本批新release/app/DMG或借旧候选代签。

## 证据、清理与下一步

证据.adg/evidence/0.2.24/official-writer-exclusion-r1/含先写假设、固化官方包/源码锚点与摘要、所有原失败/诊断、协议/锁测试、真实writer正反例输出、原件/plan/index读回和manifest。没有runtime secret/admin token或业务请求。

三个native协调Profile和一个lifecycle Profile全部自有helper/Bun/Node/test进程镜像缺席；服务原主端点经完整Darwin TCP表framing核对，附加TCP缺席仅引用准确产品trace。9个精确新外部协调DB各重获/释放EXCLUSIVE后清理，四Profile/程序副本/原件blob/index/stage/测试binary、只读展开官方src与重复回执根移除。既有globalL及其父目录完整保留。失败证据已复制逐摘要验证，不以成功覆盖。

后继在已明确的K/L临界区中继续验证完整生命周期集合、原件归属及持续写者边界，再接实际补偿和成功关闭会话；仍须显式中断恢复/全客户端范围及一般图，旧revision4首错、WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/固定候选保持。局部组件通过不关闭目标，不发布。
