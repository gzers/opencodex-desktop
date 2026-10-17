# Windows 查询与帧原始观察入口

这些工具只采集，不授予前台、不启动产品、不发出业务查询、不改变生产响应或截止，也不自行判定完整门禁通过。只接受已准入 8e MSI 载荷或原官方019精确映像，保留原配对与固定场景；产品启动/退出、普通令牌、配置隔离及还原由原生运行控制器负责。

`windows-cache-query-state.ps1` 为当前0.1.10缓存场景提供只读准备检查和纯内存偏好生成：缓存/原成功锚点/24h截止需覆盖整个拟用窗口，自动检查必须开启，关闭需真正退出，自动面板/联启/启动清理关闭，显示条件为浅色、高档、mesh及100%界面比例。该入口不改截止或缓存内容；准备报告不证明来源真实性、前台授权或零查询通过。每个被计入的重启必须实际观察到原45秒启动评估成功完成；快速启动后退出不能抵扣该场景。019帧对照沿原方法单独准备。

## 两条互补记录

- `windows-query-counters.ps1` / `.cs`：管理员后台 ETW，只启用 Process 和 NetworkTCPIP。普通产品由另一个控制器在 `ready.json` 出现后启动。记录整个测试进程后代的启动/退出、原始 QPC、进程世代、TCP/UDP IPv4/IPv6 传输事件及内核大小字段，避免每秒快照漏掉瞬时子进程。只标记双方均为回环地址且端口精确匹配本轮调试端口的采集通信；其它回环代理仍计入。没有导出对端地址、端口、数据载荷、完整命令行或 SID。
- `windows-scheduler-observer.mjs`：独立 Node 进程连接该普通产品的 WebView。对原生 `postMessage` / `runCallback` 设置始终返回 false 的调用观察条件，记录三个既有调度/查询命令及完成事件，不替换只读桥。独立 RAF 采样和 PerformanceObserver 记录原始间隔/longtask；不替换 RAF、计时器或结果。按增量输出，外部汇总，避免每秒在 WebView 中序列化整个历史。

ETW 的事件/字节不是逻辑 HTTP 请求计数；CDP 的查询命令也不是后端 HTTP 字节。必须结合两条原件及生产状态核对，不能用某一侧的零代替另一侧，不能漏掉附着前区间或把调试通信记成生产联网。RAF 间隔不是渲染回调成本；帧预算仍按原方法与实际场景判定。

进程事件保存派生调用类型，区分浏览器、控制台、npm view远端查询、本地版本参数及未分类命令；原命令行只在已归属后代的回调内检查，不导出。未分类项需保留审阅，不把所有node/cmd一概当成远端查询或一概忽略。

## 管理员录制的正向校准

`windows-query-calibration.ps1 -MechanismOnly -NodeExecutable <现有Node实际exe> -OutputDirectory <全新test/runtime目录>` 使用该已安装Node的独立字节一致副本，隐藏运行20个短命cmd子进程并完成4次固定回环请求，随后只通过自有stdin正常退出。它不启动产品、不访问外网，也不启动ETW。

实际校准另需同一入口的 `-RecordApproved -HumanWindow <新的人工窗口> -DecoderDirectory <固定解码器>`。普通控制器通过标准RunAs启动隐藏的管理员观察器，等待其ready后才启动普通诊断目标；在实际ETW根事件出现前保留目标，避免根过早退出导致无法核对映像。观察器只录自有进程族，停止排空后核对20个精确子世代、至少一个实际亚秒生命周期以及TCP发送/接收正向记录。该证据只校准所观察的进程/TCP范围，不声称已校准UDP/IPv6或产品场景。

默认管理员入口和过期窗口都必须在录制前拒绝。托管CI若本身为管理员，仅运行保护检查并明确跳过普通机制，不将管理员目标冒充普通令牌；本机普通机制单独保留真实令牌和exit0。产品不受这个机制跳过逻辑影响。

## 单次原生缓存场景控制

`windows-cache-query-run.ps1` 默认只显示准备状态。实际入口需已验证的新人工窗口、明确允许的原生UIA路径、真实ETW正向校准，以及已经保存原物理配置目录/原指针的私有事务；源EXE仍固定8e。普通应用与管理员录制器分别启动，产品令牌始终单独核验。普通原生窗口用WebView2的环境参数启用回环调试，应用smoke端口变量保持未设置，避免进入沙箱配置。

每代先绑定真实普通产品和WebView监听器，再由 `windows-cache-query-observe.mjs` 只读核对活动根、内部HOME、实际自动检查偏好和既有缓存。控制器对当前所有者核验过的窗口执行UIA侧栏动作；每个动作由实际页面路由回读确认。等原45秒启动评估真正返回成功后才停止观察并正常退出该代，随后确认原生和WebView均结束。原生帧间隔/长任务随场景保留，独立帧预算与019对照仍沿原方法判定。

默认请求原十代，每代十个真实路由变化。`generationStart/generationCount`仅用于继续经审计确认的缺失序号；总体十代/100次和完整原始计数仍需汇总核验，控制器不自行宣布完整门禁通过。失败不自动重启或覆盖旧目录，普通产品不强杀；已启动的只读观察助手若无法停止，单独记录其强制结束，整个尝试保持失败。

退出/停止排空后先恢复原目录与指针，再审核内核丢失、世代闭合、全部命令型子进程及排除精确调试端口后的实际网络事件。调用分类只提供证据，不自动豁免本地查询或未分类项。动作与完成标记先完整写入后原子发布，已有原件不得被另一写入者替换。默认、过期窗口、未完成启动评估、重复路由及记录溢出均不得抵扣原100/10场景。

## 运行顺序

1. 新人工窗口内，用新私有根准备原配置保护、已验证的缓存和普通产品启动参数；不能执行旧已过期窗口。原第25分钟正常收尾目标、第30分钟截止保持。
2. 管理员 ETW 入口默认只编译及运行纯规则，不启动会话。实际录制需 `-RecordApproved -Plan <本轮计划>`，计划绑定精确 EXE/来源、`runDirectory`、`debugPort`、独立 `stopFile` 及新窗口。`ready.json` 之前不得启动产品。
3. 普通控制器保留启动句柄，调用 `Save-WindowsObservationBinding`，将真实令牌、进程创建时刻、单实例和调试监听器到该产品的 WebView 祖先链写入新回执。观察器要求绑定该文件及摘要，不把 CDP 自身当作原生 PID/令牌证明。
4. `node test/smoke/windows-scheduler-observer.mjs --observe <本轮计划>` 只连接已运行目标。计划包括 `authorization`、本 worktree `test/runtime` 下全新 `outputDirectory`、`endpoint`、`stopFile`，以及 `binding.source` / `exeSha256` / `retainedHandleIdentityFile` / `identitySha256`。每次重启使用新目录和新绑定，不覆盖旧样本。
5. 原生 UI 控制器执行原100路由/10重启等固定场景；工具自己不导航、不点击、不手动发业务查询。观察停止、移除只读调试条件后正常退出产品，等待 WebView 收尾，再写 ETW 的 stop 文件。录制覆盖正常退出后的子进程。
6. 复核 stop/drain、内核丢失、队列溢出、解码、进程起止匹配和观察区间；任一不完整都不能用零计数通过。只停本轮唯一命名会话，不用通用 TraceEvent Stop 修改全局采样间隔，不强杀产品。原配置按运行控制器既有方法恢复。

本轮只完成编译、纯规则和隔离测试目标的真实协议接线；尚未执行管理员内核事件正向校准或真实产品场景。`complete` 仅说明录制完整性，不是业务/性能验收。

## 方法验证

```text
node --test test/smoke/windows-scheduler-observer.test.mjs
pwsh -NoProfile -File test/smoke/windows-query-counters.ps1 -DecoderDirectory <已核验的TraceEvent3.1.21目录>
```

真实浏览器方法检查另设 `OPENCODEX_OBSERVER_TEST_EDGE` 为已安装 Edge 路径，再执行 `windows-scheduler-observer.browser.test.mjs`。它使用独立无头夹具和新 profile，缺变量时显式跳过，不冒充产品通过。首次夹具就绪竞态与 DevTools evaluate 未形成真实浏览器任务的失败原件保留；修正后的夹具使用事件循环任务验证 longtask。常规 CI 执行平台无关的 V8 协议和保护检查。

协议依据为 Chrome DevTools Protocol 的 Debugger.setBreakpointOnFunctionCall、Runtime 与 Chromium Long Tasks；内核字段以已固定的 Microsoft TraceEvent v3.1.21 解码器为准。现有代码中的解码器 SHA 必须匹配，未升级或自行下载替代版本。
