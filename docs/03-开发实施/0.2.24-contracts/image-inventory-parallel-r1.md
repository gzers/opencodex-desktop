# 完整运行镜像的有界并行读取与交付配置定向验证

revision: 1；2026-11-13；IMP-34 P24-04/05；main 97a3bf93b。accepted=false，非固定候选。承接[停止收尾](client-root-stop-tail-r1.md)。固定原官方2.99的串并行清单/内容摘要相等；release配置的首次缺根Start/Status/Stop两次满足原30/5/10预算，工程debug停止超时原结果保留。下一次Start、根中断恢复、RootAbsent状态UI和整版候选验收尚未完成。

## 先测量再选择并行范围

新隔离用户Profile仅展开固定官方运行镜像，不启动服务。串行基准15646项（13393文件、2254目录，root manifest另计检查但不入自引用摘要），内容645549518字节，总774801us；目录路径检查74085151ns、条目检查65736277ns、文件准入/打开248459580ns、读/hash266304299ns。原包及descriptor摘要复核：official b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c，descriptor e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3；完整树仍a29dedc203311ca4d6d4005db94b4a10d4fb84409283669e25272f080756b790。

最终同图像对照先serial→parallel为1970408us→617526us，反向parallel→serial为655807us→694484us。两顺序逐个Entry/BTreeMap及树SHA完全相同，13393文件的worker计时全部汇总。缓存冷热/顺序会显著影响倍率，不能宣传稳定3倍；暖缓存反向只说明有限壁钟收益。worker时间为各线程耗时之和，不当作主线程壁钟。

## 保持 fresh 完整内容准入

inventory每次重新枚举所有条目，原root唯一manifest排除、嵌套manifest普通内容、深度64/100000项/单文件512MiB/总2GiB、空目录/可执行位和原摘要序列化规则保持。收集时统一seen覆盖文件与目录，防规范化路径名碰撞；预检meta大小和总量，记录最早目录metadata，保留每目录枚举末尾检查。

最多4个scoped worker，每job紧邻读取前重新check_path(root,path,false)，随后仍走当前file_entry中的NOFOLLOW/opened regular/single-link、实际文件长度和读前后mtime/mode/nlink检查。所有实际字节仍重新读取并计算SHA，不复用旧digest/先前准入SelectedProgram；helper前read_complete和动作末尾完整来源复验不变。

每个已打开文件以before.len在AtomicU64 checked fetch_update中预留内容预算，**在任何字节读取之前**保证累计≤2GiB，失败不退额度而整轮拒绝。Take(before.len+1)保留原EOF增长探测，读到额外字节立即拒绝，未靠事后长度判断代替。成功inventory字节≤2GiB；坏在途任务最多4个各1探测字节，单文件仍≤MAX_FILE+1，不出现任务全读完才累计导致无界I/O。读后真实size/mtime/权限及单链接仍核对。

Builder::spawn_scoped创建失败、worker Err或panic都取消后续领取；取消竞态中的在途任务仍受预算约束。所有已创建handle的join和内层AppResult逐个检查，全部退出后才决定成功/失败，成功还须结果数量等于jobs；没有detached I/O或错误过滤后的partial success。BTreeMap合并不得覆盖同名条目。所有读取结束后，以最初metadata对root/空目录/每级目录再次check_path/inspect/is_dir和mtime，Unix另核dev/ino/mode/uid/gid，Windows额外creation/attributes。没有从中途metadata建立新baseline。

原路径检查到实际open不是内核原子操作，原TOCTOU边界仍在，不能声称消除临时祖先转向；每job路径检查和最终全目录检查是额外防护。Windows creation/attributes不是file ID，不据此冒强身份后端；本批未触Windows独占文件，最终受影响共享扫描仍须Windows候选检查。

## 负控与旧行为对照

最终10项镜像函数（7个并行新增、3个原镜像测试）、11项来源选择、5项repair、7项removal共33个去重debug函数通过。新增负控包括1/4/超大请求worker数的清单逐行相等；祖先被转成symlink时在file_entry之前拒绝，未开始foreign payload读；读取中目录新增后整体拒绝且外部内容保留；4线程竞争最后2KiB预算仅2项读取，失败预留不触发hash；worker panic/Err全部unwind且并发≤4；规范化名碰撞、链接、稀疏增长拒绝。

EOF负控在admitted后真实增长1字节，read-complete阶段若到达则会恢复长度/mtime；必须因额外探测字节先拒绝，grew=true/restored=false。注入unwrap/assert独立成功，不用注入本身失败冒负控通过。恢复、卸载、complete_admission_never_reuses_prior_content等旧消费者回归保持。

同源码release宿主另跑镜像10＋来源11项负控全部通过；最初release复制宿主也单独跑相同21项。实际fault-injection线程panic为期望拒绝路径，join后测试通过，不当作丢失错误。all-targets strictClippy、fmt/diff-check通过。基准thread-local指标经worker-owned guard在所有退出路径汇总，未漏掉工作线程内容读取计数。

## debug与release的真实原包结果分开

工程debug首次Start8877ms/Status228ms；完整Stop达到RootAbsent9和readback，却在外层完整来源复验后返回client_native_coordination_timeout，summary10754ms（manager10352ms）。原失败保留，不改成通过；root终态本身不能替代整体Stop。

追加交付配置诊断采用仓库原Cargo release codegen-units1/LTO/opt-level=s（未修改构建策略），相同生产源码/官方包/原Start30/Status5/Stop10断言，所有关键Stop/owner/native/image/selection代码未发现debug_assertions分支或debug_assert绕过；discovery只切计时输出，preferences的frozen-schema debug_assert另作明确范围。库测试panic处理与最终app panic=abort可能不同，仍需固定app验收，不能将library test称为安装包。

| 来源 | Start | Status | 完整Stop | 独立结果 |
| --- | --- | --- | --- | --- |
| 第一个release复制宿主，源码相同但内嵌提交仍46952673e | 5991ms | 225ms | 7836ms | 原Stop成功、RootAbsent9/完整读回 |
| 当前提交97a3bf93b的release宿主 | 5979ms | 224ms | 7913ms | 同上，含协调release、最终完整来源复验及原期限 |

两轮runner exit0，summary result=passed但accepted=false，均只覆盖首次缺根完整启停。服务和launcher真实正常exit0、原4监听缺席，原根Absent schema5/独立origin、完整四帧/计划/临时marker释放和Retire读回SHA与独立completion一致。原工程失败不被release结果覆盖；本次没有修改10秒预算、不跳真实OS事实或凭据/来源校验。后继最终候选还要核对打包资源与actual app来源，release定向通过不能关闭P24-04/05或本版。

首次release构建花6m12s，源提交更新build.rs注入来源后，负控命令触发另一次6m11s relink。沿原handle等待，未重启编译。source-final分别记录debug、首release复制宿主、当前release库binary与对应源码；不把相同路径或后来rebuild的hash冒前一来源。

## 下一次Start仍未完成

当前Stop回执canStart=true不能证明RootAbsent后继可用，实际reserve/初始marker与vault历史仍有首代约束，status_source仍可能映射Required。已做只读分析并留next-generation-analysis：建议每次后继Start创建新root generation和新attempt，保持代内origin.id==attempt、create-only custody、旧archive不变；必须先区分当前缺根和历史闭合验证，在独立index/events原子追加generation来源/前驱/Preparing并用本代解析，再让状态和Start消费同一readonly后继资格判断。不能简单放开reserve_start或回移旧root冒新Original。分析未实施，后续至少两轮原包Start/Status/Stop与中断/CAS/新用户根负控俱要完成。

## 证据与清理

[140文件manifest及verify_all](../../../.adg/evidence/0.2.24/image-inventory-parallel-r1/)约311KB，保存原假设、三种镜像对照/原输出、debug失败和两release通过、源/binary/构建配置边界/33debug＋21release负控、原件与SQLite无损字节、静态复核和清理。验证脚本逐帧核独立SHA/前驱、schema5原Absent/RootBinding、scope/live、plan source/marker、完整释放树和Retire receipt/RootAbsent9；副本不冒活体证书。旧capture脚本给release输出failed命名及StopAccepted:false的通用标签有cleanup-scope-correction，实际测试结论按真实runner/summary，原记录不篡改。

全部tool/test/native/agent句柄均终止；精确9个新N/K/history DB按三轮各自独立exclusive重获后移除，逐inode原删除明细保持分开。预存global L identity/权限/字节保持且重获释放；独立清理核导出主端口，4secondary产品证明不冒额外全监听清理再验。一个镜像基准根及三个native Profile共4自有根、复用runtime/resources/复制binary在原字节逐字归档后删除，unitTMP空，工程debug/release target保留。无日常配置、真实WebDAV/模型请求、官方源码修改、新.app/DMG/Windows候选或发布。

下一步接后继generation历史/创建/索引/Start/状态同源，再处理明确中断恢复；NAME、缺Ready/一般客户端图路径/完整进程集合与刷新/复杂代理/Windows W1/W2/发布验证/README/固定候选等完整版本目标不缩减。
