# 早期原件保持会话的产品确认入口

revision: 1；2026-11-12。归IMP-34 P24-04，承接[早期原件保持原生核心](client-early-unchanged-r1.md)。后端main7d6ecdf13、界面main5b3d9e797。accepted=false、非冻结候选。本批接真实命令/预览/真实组件及唯一原型，不把完整早期恢复、最终native候选或Windows验收冒为已完成。

## 预览与执行合同

Preview新增canFinishUnchanged/unchangedDigest，核验独立原件/scope/frames和完整客户端现有原字节/权限/原缺席、最新frame的0-2稳定锁身份。适用阶段仅Captured/Ready/StopPrepared/Stopped/带原引用Failed，无restorePlan；不完整Preparing、未知原件/内容变化、其它恢复能力或摘要矛盾均不发本能力。Ready的持久引用本身不证明当前运行：若原件完整且无plan，诊断预览为Pending并提供“重新核对结束”的选择，执行时仍必须原生证明当前服务停止；不是把仅文件证据当Stopped或让活服务结束会话。

只读核对现在也检查同revision已写但未登记的unchanged-readback：缺席合法，已存在只能逐字匹配同原Head/完整当前树/锁身份，坏值拒绝Unknown并保留，不冒正常未记录。此为更早拒绝，原执行readback本来已拒绝覆盖坏回执；没有减少原生门或改变原10秒。

注册client_recovery_finish_unchanged，仍只接attempt/revision/confirmationDigest。沿已有SharedRuntimeInstall租约/更新安装及待重启门、binding-ready和blocking worker进入原unchanged.finish；无IPC路径/任意写入参数，其它平台unsupported。沿原完整官方准入、原生当前停止/记录和可选锁释放/完整树读回/原索引CAS，结果明确unchanged，不能用restored/rolled_back替代。原生文件收尾算法未重写，无自动Start。

## 用户流程与状态门

复用诊断中心ClientRecovery及唯一UiDialog。已核对可用时显示“启动前原件保持完整，原会话尚未结束”，操作“核对并结束会话”；同弹窗确认说明需重核服务停止、原件完整未变、客户端配置保持不变、不自动启动，可“返回”或“确认结束会话”。无文件恢复/回退按钮混入这一路径；Preparing/原件Unknown/有plan或矛盾能力均不显示确认。

Pinia独立finishUnchanged action，只有当前明确choice才执行；不同动作或旧结果不能代替。处理中关闭仅detach，重开显示同一promise/进度，不重发；正确回执必须同attempt/revision+1/status=unchanged。成功文案“原件保持完整，原会话已结束”，不能声称重新写入恢复。只有新live同D/H/Stopped/NotRequired及无PID/port、既有lifecycle状态核对通过才解除失败latch；另一scope/仍运行/未知不放行。

操作失败清除过期preview并保留限制，要求重新检查。操作已成功但状态读取失败则保留unchanged回执，关闭重开不会重复执行，只“刷新运行状态”重读。原件完整不变分支与已完成文件恢复的metadata收尾仍各自确认/文案/命令，不从预览布尔给写权限。

唯一总原型只更新client-recovery-024.js，固定early/early-slow/early-error/early-refresh-failed内存状态，入口/确认/返回/进度/关闭重开/失败/成功与软件一致。新发现busy时原型继承view模式会隐藏底部关闭，已在本模块显式使用confirm模式并隐藏primary，保留关闭，未改总index或其它dialog消费者。无真实文件/API/服务/持久化操作。

## 实际验证与证据边界

- 24后端session去重用例：初组23pass、旧Ready断言期望Active的1项失败保留；按上述当前合同修为Pending+canFinishUnchanged后只重验失败用例。新增无plan预览摘要与原生review一致/DTO无配置原文、坏未登记readback保留Unknown，定向重验early用例通过；其它已过函数按依赖复用，不因局部断言重跑全组。编译/格式/全目标严格Clippy通过。
- 前端61项/类型/生产build通过：无plan原件证据准入、独立path-free命令、强制明确choice、返回零mutation、在途关闭重挂单promise、错误status/attempt/revision拒绝、scope/运行态矛盾保持latch、后台失败需重查、成功后状态失败只读刷新。生产dist含真正命令与确认/成功文案，不含DEV handle/固定UUID/mock文本。
- WebKit首轮7个软件场景与prototype-early通过后，prototype-early-slow关闭按钮隐藏而失败，visual-webkit/原失败目录保留，不冒原型整组已过。修本模块模式后，WebKit和Chromium各12个软件/原型场景通过，含明暗1180×760/900×600、返回/取消、关闭重开只有1mutation、执行错误保留限制、成功后读取失败只读刷新、现有Finalize键盘和布局回归，pageerror为空。截图实际查看软件暗色确认与总原型；browser证据为mock界面，不当原生执行或整版验收。
- 原unchanged.finish执行/完整树/OS/锁/预算算法不变，明确按摘要复用上一批独立2415ms收尾/index2→3/完整两树及7路径/真实最终状态证据，引用见reused-native-evidence.json。本批未重跑native服务/文件操作、未构建新.app/DMG或最终GUI→IPC候选整链；新命令接线已真实编译，但不借旧binary宣称包含新IPC。

证据[本批目录](../../../.adg/evidence/0.2.24/client-early-unchanged-ui-r1/)及manifest保留首后端断言失败/原型busy关闭失败、最终代码和原型index观察SHA、双引擎原输出/截图、复用native证据摘要。三轮browser/Vite均退出、unit TMPDIR空，自有audit脚本terminal，本批无native Profile/服务/外部协调DB/日常或官方修改，无真实模型或业务请求；归档逐字核对后删重复临时根，正常dist/target保留。

Windows远端核对仍b2e89665d/0eef59a01无新回交，本批不跑W0/W2。下一步Preparing无独立引用/未完成捕获及已改写但未有plan的早期恢复，连同首次缺根/一般客户端与路由图/完整进程集合/旧revision4首错、Windows集成/NAME/复杂System代理/官方交付验证/README/固定候选等仍继续；目标active，不发布。
