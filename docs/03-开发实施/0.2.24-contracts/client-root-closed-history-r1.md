# 已关闭根代次的独立历史核验

revision: 1；2026-11-13；IMP-34 P24-04；main 0ac19863e。accepted=false，非固定候选。承接[并行镜像与首次release启停](image-inventory-parallel-r1.md)，这是安全后继Start的历史证据前置；没有创建新generation、放开reserve_start、改变状态映射或运行第二轮服务。

## 当前终态与历史事实分离

当前open_archived_originals/verify_retire_readback继续要求实际profile/.codex缺席、当前Grok匹配已恢复原件及准确归档树。新增open_closed_history只依据独立索引应提供的ClosedRootReferences（source Head、closed Head、originalRoot）核对旧关闭事实，不读取新一轮.codex或当前Grok作为旧baseline。

source/closed各自严格validate，closed必须RootAbsent并有独立completion，plan/sourceRevision、ticket、refs、原restorePlan和origin.id必须连接一致。按准确attempt/plan槽打开原归档，不扫描/采用目录里的新摘要；原root identity/Protection、原创建intent/created/published及原件schema5/完整历史帧/blob、scope/live、不可变plan与Grok原归档字节均按原摘要核验。准确Retire intent和readback必须存在、私有权限保持，实际完整归档树必须精确等于released execution tree，临时config写锁不得残留。receipt由原plan/source/树重新编码并比独立completionSHA；投影后再次核计划、归档与两记录，变化拒绝、不修复。

历史Grok验证读取不可变frame/blob和plan原件关系；新用户Grok配置可以不同，不能替代旧归档。新用户.codex即便是symlink，也不被历史路径跟随或接管。此事实入口没有MutationPermit/PublishedRoot/Transfer/AuthorizedReadback构造，不产生Start、move或正常Stop权限。当前根检查与reserve_start仍拒绝新路径/未集成后继。

## 从实际独立Index审计全部历史

Index.closed_root_history借用已有SQLite连接与action fd，不额外开关同inode句柄。先核data/action/pin/VFS、当前完整expected Head和原binding，校验origin与初始Preparing1，然后读原有有界events（连续revision、canonical原文、每文档64KiB/总4MiB/最多4096），最后再次核Head/全部events/binding和存储代次。

当前仅支持原首代历史，所有事件ticket必须等于该独立origin；首Preparing1、末RootAbsent，完整相邻阶段连接、原件refs/live前缀和已注册restore/rootRetirement决策不得丢弃或变换。无refs的Preparing/CaptureReserved/Failed或未知generation同样参与判定，不能因latest map只收有refs事件而漏过。

每个closed对应movement必须在source+1有准确初始Prepared登记，closed-1也须为准确Prepared；两者state、ticket、sourceRevision/plan、references、restorePlan和null completion都须完整相符。source事件与不可变计划逐字段比对。最新事件不是完整关闭则拒绝；vault实际目录集合必须恰好等于已索引关闭attempt集合，额外未登记目录不被收养。核验目录投影后，再复查全部旧归档与目录集合；未知/改写事实保留失败。

后继generation还未注册，所以这里故意拒绝其它ticket，不把首代origin挪用给新代。下一批必须扩展为各代独立origin/scope/首Preparing与前驱关闭关系，保持每代id==attempt及旧归档不变；当前入口不能当完整多代历史已实现。

## 验证和审查闭合

[30文件证据、manifest与核验脚本](../../../.adg/evidence/0.2.24/client-root-closed-history-r1/)约1.32MB，73去重相关函数通过：14个新增历史场景、根恢复总37（含14历史）、索引36；重复child行不重复计数、4ignored不计通过。fmt/diff-check、all-targets strictClippy通过。本批没有官方runtime、服务或全局协调DB执行，所有文件/进程仅自有合成夹具。

历史正例在新codex/auth和新Grok配置出现后仍能核对同一旧archive/原件，data vault字节不变；当前head/Start仍拒绝。旧Grok config blob篡改不因当前Grok不同而被接受；空目录内容完全相同但换inode拒绝。错source revision/state、错plan/origin/completion、Prepared冒Closed、extra vault、原件/scope/intent/receipt篡改、必要provenance/plan/blob删除均拒绝且不修复。

投影中实际改receipt字节或仅mode、同连接修改binding/event、替换action文件、归档新增外部文件都拒绝。注入操作先独立成功，不用注入失败满足外层is_err。完整Ready/PreStop/Stopped/live以及schema2临时写锁释放场景用明确的合成phase/PID及storage callback验证引用链；它不进入生产正常Stop factory，不代签官方原包或第二次启动。

只读审查最初发现“refsNone未闭合事件被跳过”和“中间Prepared未连接”两点，均在提交前补齐，新增反例实际通过；早期7/8个通过日志保留其当时范围，不冒最终14场景。子审查最终未发现当前首代历史事实范围的新阻断项，不执行测试/native或编辑，现已关闭。

最终6源文件SHA与实际库binary固定在source-final；verify_raw_history从原日志rawBase64逐字重建只读SQLite，核source3/Prepared4/Absent5事件、原RootBinding/schema5 Original/scope、plan/source/readback/treeSHA及新逻辑root/Grok和旧planAbsent分离。该正例是合成storage close，副本不作活体身份或权限证书。verify_all校验manifest、最终通过函数与原字节断言，始终accepted=false。

## 清理与后继

全部测试子进程实际wait终止，TempDir原现场清理、unitTMP空，原字节归档逐字核验后删重复临时根；工程target保留。没有日常配置/真实WebDAV/模型请求、官方源码/全局DB修改、新.app/DMG/Windows候选或发布。

接下来在保留初始binding/events与每代旧archive的前提下实现新generation create-only custody、当前RootAbsent/read-only后继资格、前驱关闭摘要与本代Preparing原子登记，状态与实际Start共用同一资格判断。然后验证至少两轮原包release Start/Status/Stop及新用户目录/中断/CAS/创建fd/原期限反例。首次release30/5/10通过证据不替代此后继验收；NAME、Windows W1/W2和其它完整IMP-34范围保持未完。
