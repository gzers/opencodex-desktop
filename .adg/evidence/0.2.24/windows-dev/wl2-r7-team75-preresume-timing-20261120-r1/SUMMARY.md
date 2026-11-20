# TEAM75／IMP218 Windows 首次 Resume 前恢复计时

`accepted:false`。MSVC 定向断言 12/12，测试构建和非测试库检查均 raw0。新普通桌面序列尚无运行回执。首次 Explicit 的 10 秒超时阶段仍为 Unknown。

待证伪假设是服务端在原 10 秒内完成登记、原 worker 已接手，随后完整扫描、源围栏或 V2 持久化首先耗尽期限。上轮只读证据可定位到 dispatch 返回超时。本轮只在合成样本验证了观察能力，尚无同次真实请求前缀用来判定该假设。

| 输入或产物 | 标识 |
| --- | --- |
| Windows 源码基线 | `9a49ea219e8f34dfa1867ecebfc6050fa6dcb756` |
| 主线基线 | `cfb3c8aa79de85e5a4ecdbf613acc668945ed0d2` |
| 治理合同 | `cfa4e6f0fbd0a6edd1e63cf2bd2a7432d945559f` |
| 上轮证据 | `ba54ebf76f7fff7a438606a329505da237118778` |
| 本轮源码 | `6532cbb6e612f2cc4e21684f685b28c92279ee51` |
| 编译／测试 Git 树 | `7ee052909df0c1e478cad78c8d33a98b00e667e6` |
| MSVC 测试 EXE SHA-256，104,755,200 字节 | `83fec3f05a6c356c25e0585b2199d15914baee30d4ad481080a9c5669adbd802` |
| 合同固定官方运行包 SHA-256，本轮未暂存或运行 | `14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7` |

| 阶段 | 本轮定向证据与计量覆盖 | 真实桌面证据 |
| --- | --- | --- |
| Host 原 30 秒／独立进程代次 | 保留原入口与预算，新配对尚未通过原生路径门 | 新轮未运行。旧轮九次扫描、计划内 raw31、20,820 ms、Node 未 Resume 的回执沿用 |
| DiagnosticBefore 原 5 秒 | 无新请求 | 旧轮 114 ms，原 ThreadId(51) 的连续 claim/N/L/K 回执沿用 |
| 原窗口、请求、宿主绑定和准入 | 原 QPC 起点、截止 tick、频率与请求、动作、broker 代次绑定。错绑、换钟、重放、恰到期限和未知拒绝断言通过 | 仅旧轮 intent/result 已绑定。新轮无回执 |
| dispatch 入口和返回 | 在原结果与错误旁记录进出，并覆盖意图日志和准入复查 | 旧轮已有 dispatch 返回 TimedOut，内部节点 Unknown |
| 注册、排队、原 worker 接手和首次 observe | 写入 worker 线程、失败动作、会话及入队／接手标记。争用后及时结果、确定性过期、raw5 与 LostWorkerUnknown 断言通过 | 当前超时样本的注册和接手 Unknown |
| 原件隔离、准备全量读、三道完整 source 围栏 | 准备读取与三个围栏分别计量。合成镜像前两次扫描通过，第三次内容变动保留 runtime_image_unconfirmed | 当前超时样本 Unknown |
| V2 Intent/Pending 新建、刷盘、独立读回 | 现有写集覆盖 persist 总入口、返回或错误。内部记录与 I/O 阶段待合同补齐 | 当前超时样本 Unknown |
| 结果观察、最终重验、reply／ACK | 各段记录进入、结果或错误。reply／ACK 错绑与拒绝 raw 断言通过 | 旧轮服务端 result 为 refused／TimedOut／raw=null。客户端未验证回复或 recoveryActionId |
| DiagnosticAfter 原 5 秒 | 依赖同一已验证回复和动作 | 无回执 |

前缀采用每请求 128 个槽位，仅读当前连续发布部分。合成 worker 持续阻塞时旁路读取成功，放行后保留 raw5。空洞、溢出、QPC 缺失各有标志。服务端在原期限观察和返回时分别快照，客户端在交换结束时给同请求前缀。快照 QPC 是实际采样点，线程调度可令采样晚于原截止点。按原截止 tick 审阅逐行时间，空洞或未收到回执仍为 Unknown。计量仅供诊断。

上轮原始首错为 `Manager transport or refusal authority differs.`。客户端 9,999 ms 后为 TimedOut、raw=null，测试 unwrap panic raw101。上轮 Explicit 回执 SHA-256 `f06070f4f474b94289323872db63f73657f7a39b1eccc571d07d80326967648b`。此次定向验证无用例失败。新轮首个阻塞在启动前的原生固定配对与内部计量覆盖，尚无 Host 或 Explicit 退出码。

## 集成线待接手的最小接口

1. 请提供或纳入允许写集的新固定 Profile／Public／bundle 配对。原生 `windows_client_originals024/partial_capture_fixture.rs:37–40` 只接受 range-r1，`partial_capture_fixture/isolation.rs` 依此执行 exact bootstrap、固定路径、重叠排除及句柄 pin 检查。最小改动是授权在前者固定一个新代次，现有隔离实现继续复用；仅在需要同时支持两代时扩大到 isolation.rs。有了原生配对才能在已允许的 launcher／观察器／核验器文件中固定新路径、新 out01、摘要和成员／ACL 清单。
2. 请纳入 `windows_client_originals024/before_resume_record_v2/pending_attachment.rs:118–128` 和 `before_resume_record_v2/storage.rs:313–385` 的测试观察边界，或提供同等的测试回调。需要目录创建、Intent/Pending 各自的记录新建／写入／刷盘及独立读回的进入、返回、原始错误。复用同一请求前缀和绝对 deadline，保留原 I/O、校验、顺序和格式。

确认前置后执行一次新 Host，独立核宿主代次，再在原 5 秒内由原 worker 证明 claim/N/L/K。齐备才发一次 Explicit。DiagnosticAfter 以验证回复及同一 recoveryActionId 为前提。2026-11-20 检查时 marenben 的 session 7 断开，待新配对验齐后再重连。

## 原始证据与清理

`CHECKS.json` 是脱敏后的命令与结果核对，`SHA256SUMS.tsv` 是本机封存原件的字节数与摘要。编译时工作树含上述六文件改动，提交树与测试／库检查记录完全一致。原日志及 EXE 封存于 Windows 源码工作树下 `test/windows024-native/target/r7-team75-timing-20261120-r1/`，EXE 封存副本重新验 SHA-256。合成镜像临时目录已独立检查缺席。新桌面现场、账户、Profile、PATH 和 ACL 无本轮变更。旧 range-r1 和官方 r2 没有运行新动作或清理，原现场与证据保留。构建产物与原日志按索引留置，原有未跟踪 schema 未提交。公开 Start 保持拒绝，正常 Stop/Closed、生产原件归还、普通用户 MSI、生产单 SID 和签名候选独立验收。本轮未跑 W2、双 SID 或发布。
