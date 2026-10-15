# Windows 独立测试 Profile 只读准入

2026-10-15，Asia/Shanghai。上一段[备份与本地事件实测](../20261015-ca26备份恢复轮换与事件/README.md)已经正常收尾；本段全部后台执行，没有新的产品原生实例、前台输入、安装或性能样本。完整 IMP-20 / TASK-182 目标、八项检查及原预算保持。

## 本段新增的可执行工具

代码提交 `493da80675e6508e46e0fd83d86ee7216abf96fe` 已推送到 `feature/0.1.10-windows-query-etw`，相对 ca26 仅有：

- `test/smoke/windows-acceptance-profile.ps1`：独立 Profile 的只读准入函数。
- `test/smoke/windows-acceptance-profile.test.ps1`：纯输入合同检查。
- `.github/workflows/ci.yml`：在现有 Windows smoke evidence job 中运行上述合同检查，沿用原制品上传目录。

apps、原查询夹具、collector 三文件及全部预算未改。此前 ca26 的候选与本机证据继续只证明原精确制品；本段没有生成 493 的新桌面候选，也不将 ca26 EXE 改称新 SHA 制品。

既有 20261012 Known Folder 实验已证明，仅覆盖 HOME / USERPROFILE 不能隔离当前账户的 Profile；此前官方 0.1.9 的 EXE 与 MSI 来源核验不等于可以安全启动。此处把原执行方案中“新普通本地账户 + 实际系统目录核验”落实为可执行准入，而不是再放宽环境变量隔离。

函数每次实读当前进程令牌、当前 SID、本地账户、线程模拟状态和三个 Known Folder；必须是指定的非日常账户、未提升且非受限管理员令牌、现有目录、全部路径及祖先无重解析、Profile 精确相等且两种 AppData 均在该 Profile 内、本机原生实例数明确为 0。相邻同前缀目录、点段、设备/UNC/相对/流路径、字符串冒充布尔或令牌类型、未知状态均不准入。

这是本期原独立普通测试账户方案的约束，不改变产品普通模式对 false/type3 的既有实测结论。Profile 工具拒绝当前受限管理员令牌，不表示先前普通权限产品验收失效。

Known Folder 调用为 `SHGetKnownFolderPath(flags=0, token=NULL)`，不使用 CREATE、SetPath 或绑定重定向；内存路径及 SID 只用于判断，公开回执保存摘要。函数没有账号创建、环境/ACL 修改或产品启动代码。纯谓词只供测试，不能将传入合成对象或旧 JSON 当原生准入证书。

## 实测及严格边界

本机 31 项纯合同检查、原有 9 项令牌/全部 smoke PS 语法检查均通过。两项合成正例只检查判断逻辑，不证明新账户存在。测试在提交前执行，原令牌回执如实记录父 HEAD ca26；最终方法 SHA `b75bbf4202c51fd49db63a7694deed97ffba9fd6e36d30097efb4e51c08011fb` 与提交后真实探针所用文件一致，三文件 Git blob 与工作区摘要另存。

10:24:50 在已提交 493 的干净代码 worktree 下运行一次实际日常账户负向探针。DailySid 和 ExpectedSid 明确都绑定当前真实账户，以验证即使期望 SID 相等也不能将日常账户准入；没有捏造已准备的新账户身份。结果为 rejected：

- 真实 token elevated=false / type3，当前账户与 DailySid 相同。
- Profile / RoamingAppData / LocalAppData 均未位于候选 `C:\Users\ocx0110_accept`。
- 候选目录未完成现有目录核验，本地 `ocx0110_accept` 不存在。
- 本机 desktopCount=0；无账户/Profile 创建、目录绑定、环境、权限、产品或前台输入变更。

此负例只证明当前账户会被拒绝。独立普通测试账户的真实正向准入仍未执行，官方 0.1.9 新启动数和配对性能新样本均为 0。即使将来 Profile 准入通过，也仍须新的人类前台授权、精确两个制品/产品令牌、受控场景、测量与正常退出；不能将 Profile 准入当完整性能通过。

## CI 与草稿 PR

493 的 push run `38670771363`、PR run `38670774817` 独立跟踪。10:29:53 只读快照中，两轮 Windows smoke / frontend / release-tools / macOS backend 均 success，Windows backend 仍 in_progress；完整 CI 结果待终态核验，旧 ca26 绿不代替新 head。

本段仅据 API 记录上述 job 状态，未将它描述为完整原始日志或制品归档；本机 31 / 9 计数来自实际保存的本机原件。未 dispatch、重派诊断、构建新候选或重复原库求绿。观察 API 的首次传输失败未推定远端停止；随后标准证书校验的 urllib 只读取得同两轮状态。

草稿 PR #5 标题/正文已改为当前工具范围，经本机已认证 GitHub 凭据更新后独立读回相同；head=493、base=bd2、open/draft/unmerged/auto_merge=null 保持。连接器首轮传输失败、次轮仅 integration 权限 403，以及本机首次只读 GET 的 TLS EOF 均保留说明；没有禁用证书校验或盲目重复 PATCH，没有合并、改 base 或发布。

## 下一步所缺的具体条件

官方 0.1.9 同物理机配对需要由用户准备并首次交互登录一个独立普通本地账户，建议沿用 `ocx0110_accept`。用户自行设置/输入密码，不向代理或证据提交密码；不把日常账户切为测试身份，不加入 Administrators。当前 ETW 管理员录制授权不自动覆盖新增 Windows 账户或 Profile。

该账户准备后，在其实际普通令牌下将真实 SID/实际 Profile 交给本工具重做新鲜准入；所用源码及二进制仍须复核。新账户可能没有当前私有 worktree 的访问权，后续须将已核验测试载荷复制到明确自有的测试位置并逐项校验，不放宽日常 Profile / 整个工作区 ACL。本段未执行这些准备，也没有要求用户现在打断日常工作切换账户。

安装门禁另需没有日常产品注册/真实数据的独立 Windows 安装或专用设备。新账户不能隔离机器级 MSI UpgradeCode；不要在当前宿主或仅换安装目录下运行原安装器。真实安装、默认 data/自定义根/升级卸载保留仍待验。

原性能预算不变：官方 0.1.9 同机，冷/热每版各至少 20 样本；启动可交互 p95 增量同时 ≤100ms 和 ≤5%；每版 600s CPU 增量 ≤0.5pp、RSS ≤10MiB，主线程任务 ≤50ms；freshcache 100 routes / 10 restarts 零自动网络/query 子进程。原 Mac CPU/RSS 失败、bd2/e6 失败与实际失败时间窗 ETW 缺口、真实 30 分钟退避、stable24h/beta6h、休眠/跨启动以及全部剩余原生/事件/更新门禁保持。

证据 `.adg/evidence/OCX-0110-20261015-WIN-PROFILE-ADMISSION/`。10:31:32 日常 EXE、ca26 候选、旧隔离四文件、原 TASK 共七项摘要匹配，native=0；TASK 原 3878 bytes / SHA bc7965b4…保持。用户原删除不入提交。此工具不是 required_check 通过回执，不关闭、合并、签名、晋升或发布。

本段 25 项原件/方法清单独立冻结，`verify-profile-admission.py` 通过；范围仅为工具合同、真实日常账户拒绝、PR 元数据及所列 CI 历史快照。后续 CI 终态/日志/制品另建证据目录，不能向这份冻结清单追加观测或重跑 PATCH/探针来覆盖原件。
