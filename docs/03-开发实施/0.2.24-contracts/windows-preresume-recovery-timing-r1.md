# Windows 首次 Resume 前恢复超时定向诊断

当前 revision: 3（TEAM78／IMP223）。

revision: 1；TEAM75／IMP218。Windows 源码基线 `9a49ea219e8f34dfa1867ecebfc6050fa6dcb756`，证据基线 `ba54ebf76f7fff7a438606a329505da237118778`，共享代码 `cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2`。Windows 源码基线包含共享代码，同时保留尚未进入 main 的 Windows 实现。Mac 的后继准备取消改动与本轮诊断无文件重叠。

## 要证伪的判断

服务端在首次 Explicit 的原 10 秒窗内完成 worker 注册，原 worker 在超时前开始处理，随后某个完整扫描、源校验或 V2 写入及读回阶段耗尽原期限。以同一请求无间断的阶段前缀证伪，区分 dispatch 入口／返回、注册／接手、等待／observe、扫描／持久化和回复阶段。缺失观测保持 Unknown，计时记录不转化为恢复权威或验收通过。

## 输入与动作

- 从上述 Windows 源码分支追加诊断提交。使用新隔离配对及新 out01，普通未提升 SID、原生交互桌面、无外层 Job。运行前固定新源码 SHA、新 EXE、官方运行包、CMD、launcher、观察脚本与核验器的实际 SHA-256、隔离路径、完整成员与权限清单。官方运行时沿用 r7 已核版本和摘要 `14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7`。已有 range-r1 宿主、其失败证据及旧官方 r2 三个保管原件只读保全。
- 先取得单次 Host 的计划内 raw31、完整入口与 worker 扫描、Node 从未 Resume，以及原宿主身份的独立观察。再由同一原 worker 在 5 秒内证明 claim/N/L/K。前置齐备后只发一次 Explicit。仅在取得可验证回复和同一 recoveryActionId 时做一次 DiagnosticAfter，独立读回须在原 5 秒内。Host 30 秒、Explicit 10 秒保持原绝对期限。无前置、超时、拒绝、丢回执或未知都保留首错，禁止重发。
- 使用同一请求的单调起点、截止 tick、频率以及请求、宿主、worker 绑定。有界记录进入和返回及首错，至少覆盖 dispatch、worker 注册和首次 observe、完整扫描及三道完整 source 栅栏、V2 Intent/Pending 新建／刷盘／独立读回、结果观察、最终重验和回复 I/O。超时分支读取已发布的有界前缀，避免等待 worker Drop 或阻塞 I/O。采样丢失或容量不足必须显式记录，保持 unknown 时拒绝。三个 source 栅栏原样保留，原持久格式、传输断言和权限范围保持。

## 本轮允许写集

仅限测试构建计量和取证，生产语义与持久格式维持现行合同。按以下路径排他写入，新增辅助文件限所列位置。

- Windows 控制与 worker 接线，限 `apps/desktop/tauri/src/infrastructure/` 下 `windows_owner_host024/recovery_control/{server,client,wire,wire_tests}.rs`、`windows_official_lifecycle024.rs`、`windows_official_lifecycle024/start_worker.rs`、`windows_client_originals024/recovery_worker{.rs,/tests.rs,/canonical_trait.rs}`。
- 扫描及附件阶段计量，限同目录下 `windows_client_originals024/{recovery_timing.rs,adapter/before_resume.rs,adapter/before_resume/attachment.rs,coordination/retained_start.rs}`。只在调用边界观测，完整镜像扫描器和共享 trait 由主线保留。
- 启动与回执核验，限 `test/windows024-native/` 下 `launch-r7-partial-owner.{cmd,ps1}`、`launch-r7-private-native-empty.{cmd,ps1}`、`observe-r7-readback-timing.ps1`、`verify-r7-{host-receipts,scan-sections,partial-sections}.py`；证据在 `docs/024-windows-dev` 的新不可覆盖证据目录与 `windows-dev-handoff.md`。

先验证及时返回、确定性过期及超时时回执前缀可独立读取，验证错绑、过期和未知仍拒绝，再执行上述一次真实桌面序列。回交源码／证据 SHA、原始首错与退出码、计时前缀及阶段表、同次允许确认和仍未知的事实，以及证据清单与任务残留清理结果。原始证据独立封存，Git 仅提交脱敏可核材料。本轮 `accepted:false`。正常 Stop/Closed、生产原件归还、普通用户 MSI、生产单 SID 与签名候选单独验收。禁止 W2、双 SID、正式发布及旧现场复跑或强制清理。

## revision 2 · TEAM76／IMP219

后继基线固定为 Windows 源码 6532cbb6e612f2cc4e21684f685b28c92279ee51、证据 32b0483e33fef37370bc13d9c0807edb151c9eb5，测试树 7ee052909df0c1e478cad78c8d33a98b00e667e6。沿用上文官方包及原 30／10／5 秒绝对期限。合成测试验证了观测链，首次真实 Explicit 超时所在阶段仍为 Unknown。

Windows 增加以下排他写集，承接上文原写集：

- `apps/desktop/tauri/src/infrastructure/windows_client_originals024/partial_capture_fixture.rs` 增加唯一一组新固定 Profile／Public／bundle／out01 代次。仅在保留两组固定代次确有需要时调整同目录 `partial_capture_fixture/isolation.rs`。复用 exact bootstrap、重叠排除和句柄 pin 检查，保全原 range-r1 定义与现场、旧官方 r2 现场及其三个保管原件。
- 同目录 `before_resume_record_v2/{pending_attachment,storage}.rs` 增加仅测试构建可用的观测。分别记录目录创建、Intent 与 Pending 各自的新建、写入、刷盘和独立读回的进入、返回及原始错误，纳入同请求有界前缀。测试放在允许写入的模块内。保留生产 I/O 顺序、持久格式、原期限、完整镜像与三道完整源校验。按原起止 tick 和频率解释。缺行、空洞、溢出、缺钟或落在截止之后的观测无法确认期限内进度。

开跑前固定新源码、新 EXE、官方包及脚本／核验器摘要，固定四路径、成员与 ACL 清单，并验证原隔离定义和新配对的重叠拒绝。写下本轮要证伪的判断：原 worker 在首次 Explicit 的原 10 秒内确已接手，随后某段完整扫描、源校验或持久 I/O 首先耗尽期限。以完整前缀与原始 tick 对照，无法定位时回交 Unknown。

在普通未提升 Windows SID 的交互桌面、无外层 Job，仅做一次新隔离 Host。30 秒内取得计划内 raw31，证明 Node 从未 Resume，并独立核对同一原生进程代次。由同一原 worker 在 5 秒内证明 claim/N/L/K，齐备后做一次原 10 秒 Explicit。仅在验证回复且取得同一 recoveryActionId 时做 DiagnosticAfter，实际独立读回严格小于原 5 秒。前置缺失、首错或超时即保留证据，禁止重发。本轮 `accepted:false`，公开 Start 保持拒绝。回交 SHA、各阶段原始 tick／首错／退出码、隔离范围与清理情况。保全旧现场，禁止旧样本复跑、强清、W2、双 SID 与发布。

## revision 3 · TEAM78／IMP223

Windows 推送后继续待命。收到用户转交的准确 Git 输入后，从 `codex/024-mac-integration-candidate` 的 `57073c4e5df13294e9c067ee86bf4535ded7f9b8` 开始。该提交包含 Windows 源码基线 `6532cbb6e612f2cc4e21684f685b28c92279ee51`，保留 Windows 证据基线 `32b0483e33fef37370bc13d9c0807edb151c9eb5`。测试目录树 `09f81a66a37090f36d49d39d86d076824757be92`，`build-wl2.ps1` SHA-256 `003354d8c932ffeea41391fda5b34fdc20caea20b3e9dcca72d8deab66509154`。先核对输入，再按 revision 2 测试专用写集准备新配对并记录派生提交与脚本摘要。生产逻辑、权限和持久格式沿用现行实现。

先从实际 Windows 构建产物证明 GUI 探针隔离。普通包的 `custom-protocol` 构建须证明诊断钩子缺席。诊断构建由 `build-wl2.ps1` 的 `actual-gui-build` 增加 `windows-actual-gui-probe`，须证明钩子存在。保留两侧确切构建参数、产物摘要及钩子存在性证据。环境或产物无法证明时标 Unknown。此检查只验证构建隔离。

本轮要证伪的判断是原 worker 在 10 秒内接到了 Explicit，随后完整镜像扫描、完整源栅栏或 V2 持久 I/O 耗尽剩余期限。原 Explicit 9,999 ms 返回 `refused / TimedOut / raw=null`，无 action ID，首个耗尽阶段为 Unknown。旧官方 r2 的 Node 从未 Resume，三个原件进入 custody。两处原证据独立保留。

只在全新隔离普通未提升 SID、真实交互桌面且无外层 Job 的配对执行一次。先固定源码、EXE、官方包、核验器、脚本和夹具摘要，以及目录绝对路径、成员和 ACL。按 revision 2 先取单次 Host（计划 raw31，Node 从未 Resume）并独立核对原生身份，再证同一 worker 的 claim/N/L/K，随后发一次 Explicit。只有已验证回复包含同一 recoveryActionId 才做一次 DiagnosticAfter。保留原 30／10／5 秒预算，独立读回严格小于 5 秒。采集绑定同次请求的原始 tick、有界事件前缀、首错、退出码和阶段表，并写清可确认与 Unknown 的事实。首错后停止，不重发、不复跑旧现场。强制清理不能计为正常 Stop 或 Closed。回交新证据清单、源码和证据 SHA、实物隔离检查及残留清理结果。保持 `accepted:false`，不跑 W2、双 SID，不发布。
