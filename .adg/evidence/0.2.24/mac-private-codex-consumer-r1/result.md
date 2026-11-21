# Mac 私有 Codex 后端消费验证

本轮为隔离组件诊断，整版 accepted:false。使用原样官方 2.99.0 和 ChatGPT 26.1111.71704（build 29030）随包的 Codex CLI 0.169.1。应用和运行包入口摘要见 [inputs.json](inputs.json)，试验脚本见 [probe.py](probe.py)。使用合成渠道及全新的私有目录。沙盒拒绝对夹具默认 Codex 和 Grok 目录的读写，并拒绝对外连接。[防护回执](guards.json)

| 阶段 | 自动名称（同一 slug） | 人工名称 | 官方刷新 / 消费回执 |
| --- | --- | --- | --- |
| A | 备份渠道 · vendor／custom | 人工名称 | [保存读回及目录刷新](refresh-1.json) · [model/list](codex-1.json) |
| B | 修改后渠道 · vendor／custom | 人工名称 | [保存读回及目录刷新](refresh-2.json) · [model/list](codex-2.json) |
| 回退 A | 备份渠道 · vendor／custom | 人工名称 | [保存读回及目录刷新](refresh-3.json) · [model/list](codex-3.json) |

每轮刷新后由独立 stdio app-server 执行 initialize 与 model/list。三个进程报告同一私有 CODEX_HOME，并以 0 退出。合成输入的模型 ID 与路由见三份刷新回执。试验范围为本地保存、刷新和列表读取。

官方 CLI 启动耗时 1,726.808 ms，Stop 耗时 927.577 ms，停止命令与启动器退出码均为 0。四个监听端口已消失，进程与夹具已清理。清理回执记录夹具自有协调器，既有全局协调器保持。[启停回执](run.json) · [清理核对](owned-coordinator-cleanup.json)

默认两棵树在四阶段快照中一致。私有主配置启动后改变，停后恢复字节。原件身份和保护属性恢复、实际 Electron 独立实例及 ChatGPT 桌面模型选择器仍待验。[启动前](original.json) · [就绪](ready.json) · [停止前](pre-stop.json) · [停止后](stopped.json)

归档启停回执保留 PID 与进程组，移除含即时启动证明的进程命令行。原始日志与协调数据库字节记录保存在本机证据目录。
