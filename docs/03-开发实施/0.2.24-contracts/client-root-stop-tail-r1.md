# Stop 根恢复收尾定位与索引提交去重

revision: 1；2026-11-13；IMP-34 P24-04；main f771c5e2b、46952673e。accepted=false，非固定候选。承接[根许可成本](client-root-stop-cost-r1.md)。本批保留原有逐帧动态检查和helper执行前完整内容准入；索引同次传递/提交去重落地，原官方首次Stop10仍失败，整版/nextStart/Windows最终验收均未通过。

## 收尾位置证据

测试构建增加固定ROOT_STOP_TAIL阶段和root-final-absence计时，记录intent、native_gap、rename、writer settle、before/readback、guarded proof、完整absence等白名单及微秒，不输出路径/配置/Key。trace只随测试构建，不修改release产品语义。

首prefix-context实验在未接受的共享上下文实现下到达guarded-proof-returned和final-absence-confirmed，full OS absence约248.702ms，含前后完整authority共388.683ms；随后root-close超时。独立事件保留Prepared8→Absent9→Prepared10，completion最终null，说明过期后的独立关闭回退并未因盘上回执而误放行。**该实现随后被否决，此计时只帮助定位，不作为安全等价或产品验收证据。**

## 两项未采用方案

1. 用整段前后完整root context代替每帧动态root/pin/ACL/xattr/origin检查，被只读审查指出可漏“Original后改0755、PreStop前恢复0700”的窗口。已完全撤回生产Originals变更；最终Originals源码与前一检查点逐字相同。中间额外增加的prefix末尾完整fence也撤回，它是新增工作且放大嵌套耗时，不是原有校验，不能把它当性能优化必须削掉的原门。
2. 拟在helper启动前复用同action已read_complete的SelectedProgram，因verify_current不重算实际Bun/官方模块/依赖字节而在实施前被否决。动作中可改模块后执行、末尾恢复内容，final scan无法证明中间执行过哪些字节。写锁helper前的完整内容准入与外层最终完整比较均保留，不能靠action局部cache替代。

首实验原日志、源hash和二进制摘要保留，但未提交的被否决源补丁未单独留存，不宣称可由归档重建其完整源码，也不拿它给最终实现签字。

## 最终采用：同次完整观察与提交检查合并

root_binding::resolve_originals在原完整resolve流程内返回刚核验的Originals；已有resolve包装器仍只消费client/origin。Index.checked_head_originals保持独立binding、原root来源/当前实际root、完整原件/phase链、data/action/DB pins/VFS和Head前后相等检查。返回的原件只在该次调用局部消费，没有存入Index或跨动作缓存；registered_root_originals不再完整head检查后立刻重开相同原件再验一次。

record_root_retirement删掉紧邻registered_root_originals完整观察之前的重复entry check；typed record_first_root_phase只删除写事务会完整比较的入口head复读，保留origin ticket、私有phase proof当前native/signed-ready/normalStop及提交前后require。过期/错误代次不能通过后续CAS。

Index::write在BEGIN前核对实际data/action/pin/DB VFS；BEGIN内首条INSERT/UPDATE前仍完整checked_head与binding、expected代次比较。写后完整check、失败ROLLBACK、COMMIT后完整head读回并比较next都不变。close_root_absence仅删除紧邻write之前的一次重复check，原proof.require前后、原Stop期限和提交后过期demotion保持，未减少任何实际引用变更前后的完整边界。

## 负控改进

两个最终测试固定原有逐帧语义：Original checkpoint成功改xattr/mode/替换root后，下一帧必须拒绝；另一个测试安排Original改755、到PreStop恢复700，必须在Ready之前拒绝，restored=false，实际mode保持755证明确有注入。所有注入操作先unwrap或assert syscall返回0，再断言校验失败，避免把注入本身失败误算成拒绝。没有添加测试专用通行条件或改生产权限规则。

## 本轮结果仍未通过

[127文件证据及manifest](../../../.adg/evidence/0.2.24/client-root-stop-tail-r1/)保存四个独立新Profile的原假设、源码/实际binary、原日志、无损原件/SQLite字节及核验。原30/5/10预算、真实normal-exit/捕获4监听缺席与未知拒绝保持，沙箱阻断外网和日常HOME；只使用loopback合成模型发现，无真实模型/业务同步。

| 轮次 | 来源范围 | Stop与终态事实 |
| --- | --- | --- |
| prefix-context | 已否决实现，仅原始定位 | 11027ms；读回在盘，曾Absent9后期限回退Prepared10/null completion，不可验收 |
| commit-coalesced | 原逐帧已恢复，但带新增末尾fence的中间版本 | 10379ms；目录已移动/官方writer已释放，Prepared8、无readback |
| entry-coalesced | 同次Originals回传+新增末尾fence的中间版本 | 10255ms；RestorePrepared7，原root仍在 |
| original-context | 最终生产源码/实际binary对应 | 10493ms（manager10253ms）；Prepared8/null completion，原root仍在，未开始根移动 |

最后一行真实manager日志为duration_ms=10253，不以较早实验的进度替代最终来源。最终Stop仍恢复待处理/timeout而非Stopped成功；Start<30秒事实不代替完整恢复、下一次Start或中断恢复。

## 回归、复核与来源

最终87去重函数通过（28原件快照/36索引/23根移动，索引4ignored不计），fmt/diff-check与all-targets strictClippy通过。子审查对同次Originals来源传递、原件/动态权限、BEGIN前后和CAS规则未发现直接遗漏，不执行测试/native或编辑，不代签性能。所有子审查已关闭。

最终5个提交文件与original-context失败轮source逐文件SHA、实际执行库binary完全对应，source-final固定两提交；最终生产Originals源码无改动。被否决实验的27项通过仅属当时来源，不能冒最终87项。verify_native从原字节逐项核摘要、Original缺根schema5/RootBinding、scope/live、phase前驱和准确事件链；verify_all另核最终source/binary与断言。离线副本不是正常Stop或活体权限。

## 清理审计边界

所有test/native/agent句柄终止。四轮原件/索引/合成HOME按实际现场完整捕获后归档，逐字核验后清四个自有Profile、复用运行包/资源/执行fixture和重复临时证据根，unitTMP空，工程target保留。原官方资源摘要复核未改，无日常配置、真实WebDAV/模型、新.app/DMG/Windows候选或发布。

首次清理工具回执确认删除12个精确新N/K/historyDB，均先exclusive重获；其详细逐inode清单因复查脚本误重跑被空列表覆盖，**归档中不具备该逐删除inode明细**，仅保留原工具计数说明、每轮准确目标路径和复查全部Absent事实。cleanup-audit明记此证据缺口，不补造明细。预存global L在两次检查中identity/权限/字节不变并exclusive重获释放；独立核对导出主端口，产品4监听证明不冒独立清理全监听再验。

## 下一步

不继续盲跑Stop。已有完整计时显示有限去重无法稳定满足原10秒，下一步需在保持helper每次完整内容准入、每帧动态权限与全部mutation边界的前提下，测量完整内容inventory各项成本并评估同等核验的并行/批量I/O实现；不是复用旧摘要或跳过来源检查。只有提出可证伪且不降判定的改动才开新native轮。Stop通过后仍需nextStart、RootAbsent状态UI和明确中断恢复；Windows W1/W2、NAME、一般客户端图路径/完整进程集合与刷新/System代理/发布验证/README/固定候选等全范围不缩减，目标active。
