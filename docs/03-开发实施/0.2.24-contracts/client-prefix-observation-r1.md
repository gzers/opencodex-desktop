# 帧入口祖先链合并与计量 r1

revision: 1；2026-11-13；IMP-34 P24-04；main e782afb6b；accepted=false。承接[双轮原包失败](client-root-two-cycles-r1.md)。本批没有运行官方服务，没有重跑原包失败轮；原第二Stop12878ms/RestorePrepared16仍是当前有效失败。

## 最终改动

唯一生产行为变化是 frame 入口 data/profile/client/vault 四份目录 pin 的共享祖先合并。PinnedDirectory::recheck_set 按完整路径形成调用内唯一集合；相同路径的预期 identity 或全部Protection不同，聚合阶段立即拒绝。每个唯一对象从持续借用的原parent fd做nofollow fstatat，与retained fd fstat及完整owner/mode/flags/ACL/xattrs比较，整个集合执行两遍。data/directory的private断言在当前完整Protection已与pin相等后核原leaf属性，profile/client原准入不放宽。

raw父fd由输入pins借用持续持有，不对SQLite inode新开/关闭辅助fd。不同字面路径不按inode合并；空格、中文、反斜杠仍为原字面路径，点路径/相对路径拒绝。没有跨调用cache、来源学习或持久格式变化。

每帧完整verify_indexed_anchor、全部来源文件读取/hash/解析、namespace检查、frame SHA/predecessor/schema/root身份与权限、Original baseline、tree/Grok archive核验及末尾namespace保持原样。没有采用scoped-provenance guard，也没有singleframe特判。

## 测量与被拒绝方案

局部test-thread metrics记录open_at/capture_file/pinned open/recheck/Protection/frame/prefix/root provenance/plan load的调用次数与inclusive时间；嵌套inclusive时间不能相加，也不是全机syscall追踪。reference模式仅cfg(test)编译，thread-local RAII封闭，无产品env/serde入口；生产固定false，reference仅切回原帧入口四目录独立检查。

同一夹具/同一最终binary的正反顺序 reference→union→union→reference 测量：reference90.75/89.93ms，union87.79/86.30ms，均完整验证八帧；来源核验10→10、capture64→64、pinned open56→56，Protection6683→6299（减少5.75%），该小样本均值约3.65%变化。只支持这个合成debug观察内的小幅收益，不能推导原包Stop通过或通用速度承诺。

曾尝试首尾完整来源认证加逐帧retained-fd freshness、namespace pins及union。但四帧观察局部收益不能抵消单帧Original路径开销，串行capture约4.35–4.54s/3.86–4.06s，参考约2.99–3.05s；因此整套来源复用实现与专属新测试已从最终代码撤下，从未提交或运行原官方服务。相关编译/fixture失败、负控、参考和退化计量保留，不能引用它们声称最终生产代码具有不同的认证边界。

最终祖先union的相同串行capture约3.02s，原10s创建保管与全部断言不变，完整来源计数1089与参考相同。独立回归确认未知/权限/身份/路径/摘要等原拒绝仍保留。

## 验证边界与后续

最终186个唯一受影响函数通过（含显式执行的1个ignored诊断函数），strict all-target Clippy/fmt/diff通过。此前拒绝方案的失败不计为最终通过。

新增三项union直接负控：同路径旧/新inode或同inode不同Protection两种输入顺序均准确断言聚合冲突；首遍已扫共享祖先后注入mode/rename/symlink/xattr，第二遍拒绝；父子兄弟/重复roots排列、中文空格反斜杠、非法点或相对路径以及跨调用变更。原每帧动态属性与archive/恢复负控继续运行。

并行回归中出现创建保管期失败；按开跑前假设独占同source/binary原deadline复核，祖先用例和一项捕获通过，但多场景捕获仍失败，故没有归因成环境通过。随后同code reference诊断证明来源复用造成退化，拒绝该方案。最终祖先union的参考/捕获/受影响回归另留日志，原失败不覆盖。

证据 .adg/evidence/0.2.24/client-prefix-observation-r1/ 保留开跑前假设、最初基线、所有被拒绝实现与失败日志、最终source/binary/计量和清理。未执行官方runtime/service/全局协调DB，无日常配置/Shell/已安装app或真实WebDAV/模型请求；没有固定候选/发布。

下一步继续对真实完整旧归档在同一观察内的重复打开/重复frame-prefix做计量与有界优化。本批小幅收益不足以证明能消除第二Stop超时，不盲重跑失败轮，原30/5/10及Restart30/Stop子预算不变。根中断入口、缺Ready、一般客户端范围/进程集合/刷新、NAME/复杂代理/Windows W1W2/发行与固定候选等IMP-34全范围保持。
