# Windows 查询与帧原始观察入口

这些工具只采集，不授予前台、不启动产品、不发出业务查询、不改变生产响应或截止，也不自行判定完整门禁通过。只接受已准入 8e MSI 载荷或原官方019精确映像，保留原配对与固定场景；产品启动/退出、普通令牌、配置隔离及还原由原生运行控制器负责。

`windows-cache-query-state.ps1` 为当前0.1.10缓存场景提供只读准备检查和纯内存偏好生成：缓存/原成功锚点/24h截止需覆盖整个拟用窗口，自动检查必须开启，关闭需真正退出，自动面板/联启/启动清理关闭，显示条件为浅色、高档、mesh及100%界面比例。该入口不改截止或缓存内容；准备报告不证明来源真实性、前台授权或零查询通过。每个被计入的重启必须实际观察到原45秒启动评估成功完成；快速启动后退出不能抵扣该场景。019帧对照沿原方法单独准备。

## 两条互补记录

- `windows-query-counters.ps1` / `.cs`：管理员后台 ETW，只启用 Process 和 NetworkTCPIP。普通产品由另一个控制器在 `ready.json` 出现后启动。记录整个测试进程后代的启动/退出、原始 QPC、进程世代、TCP/UDP IPv4/IPv6 传输事件及内核大小字段，避免每秒快照漏掉瞬时子进程。只标记双方均为回环地址且端口精确匹配本轮调试端口的采集通信；其它回环代理仍计入。没有导出对端地址、端口、数据载荷、完整命令行或 SID。
- `windows-scheduler-observer.mjs`：独立 Node 进程连接该普通产品的 WebView。对原生 `postMessage` / `runCallback` 设置始终返回 false 的调用观察条件，记录三个既有调度/查询命令及完成事件，不替换只读桥。独立 RAF 采样和 PerformanceObserver 记录原始间隔/longtask；不替换 RAF、计时器或结果。按增量输出，外部汇总，避免每秒在 WebView 中序列化整个历史。

ETW 的事件/字节不是逻辑 HTTP 请求计数；CDP 的查询命令也不是后端 HTTP 字节。必须结合两条原件及生产状态核对，不能用某一侧的零代替另一侧，不能漏掉附着前区间或把调试通信记成生产联网。RAF 间隔不是渲染回调成本；帧预算仍按原方法与实际场景判定。

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
