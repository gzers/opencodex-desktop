# 恢复撤销与重新准备的产品确认接线

revision: 1；2026-11-12。归IMP-34 P24-04，承接[原生回退与重准备](client-recovery-rollback-r1.md)及[显式继续](client-recovery-continue-r1.md)。后端main026529269、界面main522d042e1。accepted=false、非冻结候选；本批把既有已验证执行器接产品命令/真实组件及唯一原型，不冒整个中断恢复范围或签名候选已验收。

## 后端和方向确认

注册client_recovery_rollback、client_recovery_rearm两个Tauri命令，均复用现有SharedRuntimeInstall排他租约、安装中/待重启更新拒绝及binding-ready门，blocking工作持有租约至返回。用户参数仍仅attempt/revision/digest，无任意路径/内容/权限选项；其它平台明确unsupported，无Windows权限/服务行为替代。调用原rollback::rollback/rearm::prepare，未修改原生publication、原OS/进程/port/claim-N-L-K/完整镜像/读回/终态算法和原30/5/10秒。

Preview新增canRearm/rollbackDigest/rearmDigest；RestorePrepared可同时有继续和撤销两个真实选择，各自用Decision派生的不同摘要；RestoreRolledBack只能重新准备且使用rearm前缀摘要。已完成文件恢复分支仍只收尾。未知、坏原件/计划/回执或上下文无法核对时不发任何能力。新增session断言证明preview撤销/重准备摘要与各原生review完全一致，继续摘要不能代用撤销摘要。

两方向摘要共享一次整组只读核对结果派生，避免将同一批plan/目标/暂存/归档/余项树重复读取；序列化绑定内容/顺序及各自decision与此前单方向算法保持一致，不减少任何不同事实核对。preview只是当前可审查的确认材料；执行器仍在真实排他锁/当前原生权限下重核，不由展示布尔签发许可。

## 界面与原型的实际流程

诊断中心同一UiDialog内：部分中断状态保留“确认继续恢复”，旁加“撤销本次恢复”。点击撤销进入同弹窗明确说明：回到恢复前的停止状态、启动前原件仍保留/尚未恢复、之后可重新准备；可“返回”，主操作“确认撤销恢复”。不叠第二个全局dialog。处理中关闭仅detach，不取消原生动作，不再次提交。

真实rolled_back回执须同attempt且revision+1；界面继续保留恢复限制。只有新live同D/H、Stopped/Required、无PID/port、三动作false、operation idle及preview同attempt/revision/rolled_back阶段核对，才显示重新准备入口；不调用清除latch的成功方法。

“重新准备恢复”另有同弹窗确认，说明只是准备、客户端配置保持不变、之后仍需点击确认继续。真实restore_prepared回执同attempt/revision+1后，同样新live Required及精确preview核对，展示新计划和“确认继续恢复”，不自动Continue或Start。最后明确Continue返回Restored后，沿原fresh live scope核对才能解除限制；前两步永不当NotRequired。

Pinia保持唯一执行状态和action/receipt，关闭重挂/重复点击不重发；操作成功但状态/preview刷新失败保留成功回执，按钮只“刷新恢复状态”，只能重读，不重试mutation。错方向status/attempt/revision拒绝并清旧preview；真实执行失败亦要求新检查且保留latch。重新打开已成功核对的非终态操作会重新检查CURRENT计划，不沿用历史成功承诺。选择/取消状态只在当前dialog，不会从关闭弹窗反向触发操作。

唯一总原型只改client-recovery-024.js，以固定内存展示同确认、返回、busy、关闭重开、失败、状态读取失败和完整撤销→重准备→另行Continue。复用原有modal的onCancel合同，不覆写全局cancel事件；临时额外按钮在render/close移除，不污染其它弹窗。总index及其它并行原型未改未提交；没有真实API、文件、持久化或系统调用。

## 本批证据与复用界限

- 22个后端session函数通过；新增方向摘要/能力、重新准备阶段证明，既有只读观察锁/原件/损坏拒绝/终态证明保持。编译/格式/全目标严格Clippy通过。没有重跑未变的普通启停/原生回退服务链；上一批8130/9557/9434ms原生核对和其完整来源/manifest摘要引用固化于reused-native-evidence.json。当前Tauri命令注册被真实编译，未构建新.app/DMG或执行签名候选GUI→IPC原生整链，不冒这部分已完成。
- 44个前端用例通过：专用无路径命令、不同direction摘要、返回零mutation、成功后新scope/phase/revision、两操作各关闭重挂单promise、错误回执、矛盾DTO/共用摘要/未知回执拒绝、成功后只读刷新、busy/另一scope拒绝及完整三次明确确认。最终新增busy用例误填不存在的starting操作枚举，ui-build-final.log保留；仅改为产品既有applying后44项/类型/生产build通过，不放宽状态门。
- WebKit和Chromium各22个软件/原型场景通过，覆盖撤销/重准备/慢返回/执行错误/状态读取失败以及既有Continue/Finalize、明暗1180×760和900×600；用户返回零mutation、在途关闭重开只有一次、前两步恢复限制保留、无自动Continue。补完整三确认流及原型慢操作关闭重开后，两引擎各12个受影响检查通过，包含各自软件/原型三次确认链；最终只有Restored+新live清限制。pageerror为空，真实consumer为LogsRoute+ClientRecovery+既有UiDialog；截图实际检查，不把browser mock当原生文件/服务验收。
- 生产dist排除DEV handle/固定合成UUID/mock文本，含真实rollback/rearm命令及两确认文案。既有原型样式保持；不为掩盖差异修改并行总index或其它原型。

证据[本批目录](../../../.adg/evidence/0.2.24/client-recovery-rollback-ui-r1/)及manifest保留原build失败、最终准确源、原型index观察摘要、四轮browser原输出/截图和复用来源摘要。四轮browser/Vite均terminal，自有audit脚本终止、unit TMPDIR为空；本批无原生服务/外部协调DB或Profile，归档逐字核对后删除重复临时根，正常dist/target保留，无日常/官方/系统配置或真实业务修改。

Windows远端只读核对仍b2e89665d/0eef59a01，无新贡献；本批不重复W0或W2。下一步更早Preparing/Captured/StopPrepared等中断恢复、首次缺根/一般客户端与路由图/完整进程集合/旧revision4首错按IMP-34推进；Windows集成、NAME、复杂System代理、官方交付验证/README与固定候选仍未完，目标active，不发布。
