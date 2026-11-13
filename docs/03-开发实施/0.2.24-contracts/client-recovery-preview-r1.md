# 0.2.24 客户端原会话的只读恢复诊断

revision: 1；2026-11-12。归IMP-34 P24-04。代码main 2ce09eb97d9fdfc1bb46b9b3876ee8d2ada18797。accepted=false；本批是显式恢复的诊断/查询前置，执行器、继续/回退界面和产品整包验收尚未完成，不把只读查询写成中断恢复已交付。

## 真实入口与合同

注册Tauri client_recovery_preview命令，无请求路径/候选摘要/决策/凭据参数。只从当前SharedDataRoot/ProcessContext取得当前绑定，run_readonly在工作线程执行；不取得写操作许可、不初始化索引、不启停服务、不申请官方写锁或自动恢复。前端client-recovery.ts提供严格类型和唯一preview调用，未接新UI组件或虚构按钮。

Mac输出status为not_required/active/pending/restored/unknown，phase为原索引阶段，revision/attemptId仅来源于独立已校验索引；originalEvidence、planEvidence、completionEvidence分别说明未记录/已核对/未登记/未知。phaseCount只记录独立阶段数。active表示索引原会话Ready，不是新的真实服务就绪证明；restored表示原登记完成链读回通过，不替代当前OS运行状态。canContinue/canRollback本批一律false，不能凭预览执行原apply。其它平台固定unsupported，无Mac证据代签Windows。

## 同一观察租约内完整读取

旧inspect抽成inspect_map，兼容原inspect只克隆Head的调用语义；新project回调在同一shared action锁内执行，读取后再核对data/client pin、index pin与作用域/动作锁身份。Busy在打开SQLite inode前拒绝，避免同进程关闭额外index FD释放另一个writer的POSIX锁。原SQLite immutable/query-only/schema3、无sidecar、current/event一致、目录权限/原生身份保护保持。

有References时，用其独立original/phase摘要打开原件，核对完整档案/内容对象、startup-scope与当前data/client/profile/home身份；有Live时另验独立Live摘要/Ready关联及有界字段。这里只核对持久证据，**不把手写Live的PID或错误码当内核Ready/退出证明**。有plan时Pending::open按原独立id/SHA验证原计划与各phase。计划缺失/篡改、原件/Scope/Live错误、坏权限/目录变化返回保守unknown，固定DTO不带原错误详情/配置/审计/密钥/令牌内容。

若completion文件存在但原Head不是Restored，仅标unregistered，不读取其observed摘要来补登记，也不发继续/回退权限。Restored仍沿原独立completionSHA/outcome/Scope/Live/plan全链核对；预览不是修复原件或自动关闭旧索引的入口。当前没有confirmationDigest或执行命令，不用假审批字段暗示恢复功能已完成。

## 检查与边界

49项后端去重函数通过：session21、index22、completion6（含已有辅助入口），4个manual入口ignored不计执行；新增4项覆盖缺会话不初始化、Preparing/Captured/Ready投影、动作锁忙/整个回调锁不提前释放、原件与Scope破坏/plan缺失拒绝、未登记完成文件不变、输出无私有目录/Live原文及continue/rollback始终关闭。

首session组20过1失败是新夹具误认为manager-state不存在，实际来源初始化已有通用目录；仅测试改为比较真实前后条目且无新index/action文件，原stdout/stderr保留。原产品/保护规则不因夹具改动放宽。

前端2项调用合同测试通过：调用不带路径/凭据/决策，IPC错误原样传播且无自动重试/写入。首轮beforeEach简写返回mock函数被框架当cleanup调用，导致错误路径测试清理失败；改为void初始化块，产品API不变，原失败保留。UI类型检查、cargo编译/格式/全目标严格Clippy通过。

准确最终binary SHA 1450e47872fe31cafeabcf3276fe633d548dd319a5a4df894ec249f19fe4e6d0绑定49项检查。Tauri注册入口经编译，文件/SQLite/锁投影为真实原生夹具；前端invoke传输用mock，未做真实GUI调用或新组件渲染。本批不重新跑无新假设的原包启停，未启动任何官方服务/模型/业务同步，不把纯诊断验证冒恢复执行验收。

## 证据、清理及后继

证据.adg/evidence/0.2.24/client-recovery-preview-r1/保存假设、准确源/build/binary、首错/49+2结果和清理回执/manifest。4个新增诊断与锁测试另只为清理核验在精确自有TMPDIR执行，结束目录确认为空后删除；总检查数按去重49，不将这次清理重复记成新覆盖。旧组TempDir按RAII回收，未伪造未记录随机目录的逐根清单。精确测试helper命令均缺席，无本批外部协调DB/副本程序/Profile或官方包，没有日常配置/系统修改；逐SHA保存证据后清理重复临时根，无新.app/DMG/发布。

Windows远端本轮仍b2e89665d/0eef59a01，未凭已推合同代签新贡献。下一步继续可审查的显式恢复执行/确认合同和相应UI，先逐阶段确定新的OS停止/锁/原件/冲突事实，不把旧normal-exit布尔反序列化成许可。一般客户端/配置图/完整集合与旧revision4首错、WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README/固定候选仍未完，目标active。
