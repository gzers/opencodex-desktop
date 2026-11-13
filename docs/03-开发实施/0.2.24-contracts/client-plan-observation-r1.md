# 历史归档单次计划认证收敛 r1

revision: 1；2026-11-13；IMP-34 P24-04；main 350bc0309；accepted=false。承接[帧入口计量](client-prefix-observation-r1.md)。本批只减少同一历史只读观察开始阶段的重复 Plan 加载；没有执行原包服务或改变当前恢复/移动路径。

## 实现与边界

原 open_closed_history 经 open_archive_reference 的 Pending::open/load，再次 Pending::open/require_source，最后 verify_historical_close 的首尾 load，共认证同一 Plan 六次。前四次之间没有写入或外部回调，不构成独立的 before/after 观察边界。

新增私有 open_archive_shell 仅固定 data/profile/vault/retired-root pin，不对外返回“已认证历史”。唯一历史调用者必须随后通过同一个 Pending::open_with_plan 的原完整 load 获得 owned Plan；完整比较 initial.source 与独立 source Head，才进入本次观察的私有 verify_historical_close_from_initial。

首次 load 仍核规范UUID、完整文件私有保护、独立 Plan SHA、strict JSON/字段/schema、origin、source状态/Original绑定、execution tree及所有writer marker约束；origin 仍从 owning Originals 继承仅内存 namespace。不存在只验hash后另造Plan的捷径，也不经序列化重建 namespace。Plan按值仅在本调用内消费，不放入 Pending、不缓存到后续动作。

历史读回的 checkpoint 后，仍独立 self.load，核 Plan 编码相等，再完整读取第二次archive prefix、tree、Grok blob及intent/readback记录。每帧来源/内容/pred/root/protection/Original、scope/live与真实released tree检查不减；两次receipt重建与独立completion SHA保持。上层完整index/events、每代来源、scope与vault membership审查保持，且原前后各调用历史入口一次，故该层计划加载为12→4，不是整次索引只读两次。

公开 Pending::open 仍执行原完整load后返回Self；当前 open_archive_reference 仍核完整独立source，current/Stop/移动许可继续走原路径，未接历史shortcut。测试reference模式生产固定false，只有cfg(test) thread-local RAII开关可走原六次路径，不能从env/serde使产品采用其它判定。

本批保留前后完整认证与持续变化拒绝，不承诺捕获任意外部并发瞬时改坏又恢复（ABA）。减少无写入/回调间隔的重复读取不等于所有不合作并发调度下接受集合逐时刻相同；原实现也不能保证捕获所有瞬态。

## 验证

同一binary/同一完整schema2四帧历史夹具，reference→current→current→reference正反顺序：reference约86.89/86.32ms，current约81.10/82.19ms，计划load6→2；八帧、两个prefix、十次完整根来源检查不变，capture64→60、Protection6299→5927。仅说明这份debug合成观察的小幅收益，不宣称原包Stop10已通过或通用加速。

104个唯一受影响函数通过（新增3、fullphase比较1、其余历史36、索引56、多代8；5项ignore不计），strict all-target Clippy/fmt/diff通过。

三项新反例：准确计数2且后续调用污染仍拒绝；身份/Original链合法时，仅改独立scope或source revision、精确断言完整source比较拒绝，另核digest/id/completion错误不写盘；首次完整读回checkpoint后改Plan字节、权限或换成未封存文件，结束认证必须拒绝且不修复。replacement-unsealed验证的是0600访问准入，不声称同字节合法400 Plan文件的inode必须连续。

原archive/来源/receipt/provenance/frame/blob/scope/live、当前新根与历史分离、native gap及多代namespace负控继续验证。证据 .adg/evidence/0.2.24/client-plan-observation-r1/ 保留开跑前假设、源码/实际librarybinary、原与当前计数/正反对照、所有测试和review/清理。最初source负控会被前置身份检查截断，已加强为scope/revision及精确错误；旧通过日志不冒加强用例已执行。

## 后续

上一原包双轮第二Stop12878ms/RestorePrepared16仍为有效失败。两批局部收益尚未证明能消除超时，不盲重跑原失败轮。下一步将计量上层两次历史入口与内部前后archive prefix是否重复覆盖同一观察；若合并，必须由封闭局部观察对象保留原件/来源/完整before-after字节与保护、投影后的独立重读，不靠磁盘学习或跨动作cache。

原30/5/10及Restart30/Stop子预算、根中断/动作回执容量边界、缺Ready/一般客户端图路径/进程集合与刷新/NAME/复杂代理/Windows W1W2/发行与固定候选等IMP-34全范围保持。无日常配置/Shell/已安装app、真实WebDAV/模型请求或原官方2.99改动，未发布。
