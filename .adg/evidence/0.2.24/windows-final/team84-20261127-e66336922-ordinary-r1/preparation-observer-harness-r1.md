# TEAM84 Windows 准备页时间线采证工具

- 基线 main `e663369226337c9a9cad594ccd3cafd1719c3711`，TEAM84 `3f1b7da702f9e255bcc1f3071d9cc49f3139da75`，既有证据 `94a921ec5a5d44f89526ea2d59f93a84f3739c1e`。
- 本轮代码 `codex/024-windows-preparation-observer@b6f37e07e6129e243b1f5a2e1570cedc7cf90fc5`，仅含 Windows 冒烟采证入口和测试。
- 待证伪判断：只采到终态会被误记为看到本地准备进度，或后续尝试会继承前次尝试的进度。预期第一次终态保留证据，终态单样本不报进度，变更尝试后重新等待并独立记录。

### 运行证据

- Node 24.22.3。三份模块通过语法检查。Windows 观察器、CDP 回环发现、版本化目录准入共 20 项定向断言通过。序列化 WebView 函数在模拟页面核对了只读 IPC 和可见元素筛选。详见 [原始 TAP](raw/preparation-observer-tests.tap)。
- CLI 用两份已存在的回执模拟复用输出目录，返回 EEXIST，原内容留存。探针首错是 PowerShell `New-Item` 不支持 `-LiteralPath`；初次没有建出目录。改用 `Directory.CreateDirectory` 后复验通过，临时夹具已清理。详见 [首错](raw/preparation-observer-guard-first.txt) 和 [复验](raw/preparation-observer-guard-retry.txt)。
- GitHub Actions 的 main 回读仍指向既有构建 `45973489918`。沿用已归档的 MSI/NSIS 摘要。详见 [Actions 回读](raw/preparation-observer-actions-readback.txt)。

### 新候选采集方式

在已经获准的测试启动入口打开隔离窗口，使用仅监听回环的 WebView2 CDP。传入窗口实际绑定的绝对沙箱根和新建的输出目录。在启动后尽早运行观察器。

```powershell
node test/smoke/windows-webview.mjs http://127.0.0.1:<port> <new-output-dir> <isolated-root> observe-preparation
```

`preparation-timeline.jsonl` 逐次记录发生变化的后端准备快照、页面上「无需联网」和准备步骤列表的可见性，并保留归属尝试的终态。浏览器先读准备快照再读 DOM，页面自身独立更新。两者同时采到只证明该时段的同帧观察，不能证明 UI 与后端处在同一修订。采集耗时从观察器开始计；早于附着或处于采样间隔的渲染可能漏记。采到终态也不证明离线、没有 HTTP 请求、首次 Status 时长或可安全 Start。缺包确认仍需另外保留用户动作与同次下载的证据。

本轮未新增实机验收结果，`accepted:false`。完整包离线、缺包确认下载、首次 Status 与确认空闲后的 Start → Status → Stop 待同源候选与相应前提。沿用 Start／Status／Stop 的 30／5／10 秒预算及未知拒绝。保留历史失败与实机现场。
