# 后续窗口的就绪绑定（本轮未执行）

这份方法只准备控制顺序，不提供前台授权。新的人工开始信号及约定窗口到达后，再在诊断 worktree 的 `test/runtime` 下创建全新 RunDirectory。填写真实 `human-window.json`，保存同一轮 CI 的完整成功读回，再执行 fixture → baseline019 → candidate0110。没有新的开始信号、完整 CI 或至少 620 秒剩余时间时不启动下一段测量。

`launch.ps1` 仅负责一次指定版本的新进程及真实普通令牌绑定；`observe.ps1` 仅调用 f227 已提交的 `test/smoke/windows-idle-observe.ps1`，不复制或修改生产采集器。动态原件全部写到 RunDirectory，冻结准备目录不再接收运行结果。结果另建 `.adg/evidence/` 档案，不改此前 019 无效轮或 d94 中断轮。

Computer Use 按技能重新读取当前返回窗口，按启动记录的完整 EXE 路径筛选，必须唯一。激活后观察，不在测量中反复抓屏或注入输入。至少真实稳定 60 秒后，另取一次就绪状态，确认概览准备态；用返回窗口及原 AX 记录写 `<version>-sky-settled-ready.json`：

```javascript
// 先初始化 @oai/sky，并通过 list_apps/list_windows 得到实际返回窗口。
// returnedWindow 仅来自本次选择；runDirectory、version 已绑定真实执行记录。
globalThis.idleReadyState = await sky.get_window_state({
  window: returnedWindow, include_screenshot: false, include_text: true
});
globalThis.idleObservedAtUtc = new Date().toISOString();
nodeRepl.write(String(idleReadyState.accessibility?.tree || ""));
```

停止并检查该 AX 后，在下一调用保存同一观察的 `observedAtUtc`、`window`、`accessibility`。时间必须在观察取得时记录，不能延后写入一个新时间冒充已稳定。原生入口会再次核对真实进程代际、普通令牌、窗口归属、2360×1520 客户区及至少 60 秒稳定期。

观察身份与原件字段固定为：

```javascript
{
  observedAtUtc: idleObservedAtUtc,
  window: idleReadyState.window,
  accessibility: idleReadyState.accessibility,
  screenshots: [],
  screenshotPayloadPublished: false
}
```

若需调整窗口，先通过新的 Computer Use 观察与单次动作完成，再重新取就绪记录；不在计数开始后调整。600 秒按首末完整读数跨度计算，任一读取、代际、前台、可见性或单实例失败立刻作废，不拼接旧片段。两版各一次，不能为挑选通过值自动重跑。

每段完成后先核对 `terminal.json`，启动 `observe-exit.ps1` 保留对应进程句柄，再通过刚观察的真实窗口 Alt+F4 正常退出。确认 exit0 和全机 native0 才启动下一版。失效段单独记录中断与正常关闭，不能调用完整对照分析；30 分钟窗口到期后不启动新原生动作。

预定时序：0–2 分钟准备及 019 稳定，随后约 10 分钟测量；正常退出后用约 1–2 分钟启动/稳定 d94，再测量约 10 分钟；余时只核对与退出。冷/热各 20、真实周期、安装器等另安排，不在此空闲子集中声称通过。
