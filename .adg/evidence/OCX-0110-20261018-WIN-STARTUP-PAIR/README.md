# Windows 原生冷/热启动配对

启动子项通过：官方 0.1.9/de6862d4 与已准入 0.1.10/8e 同机、同真实普通令牌，两模式各 20 次/版本，共 80 样本，全部正常 exit0。01:24:14（+08:00）恢复原配置并归还前台，早于 01:40:33 正常收尾目标及 01:45:33 授权截止。

| 模式 | 0.1.9 p95 | 0.1.10 p95 | 增量 | 原增量≤100ms且≤5% |
|---|---:|---:|---:|---|
| WebView 磁盘配置冷 | 1354.5511ms | 1273.1190ms | -81.4321ms / -6.0117% | 通过 |
| 正常退出后进程热 | 1111.7454ms | 1103.4023ms | -8.3431ms / -0.7505% | 通过 |

用户批准独立原生 UIA 驱动及新 30 分钟窗口。20 轮交替版本顺序，每版冷后热；端点为 Process.Start→可用设置按钮→UIA Invoke→设置路由文本出现，仍受原 30 秒上限约束。采用原 nearest-rank 分析器；未清系统缓存、不称操作系统冷启动，不删初轮较慢样本，不重跑整套取优。仅一实例顺序运行，轮间核 native0；无沙箱、CDP 或强杀。

采样本身全部完成；收尾偏好回滚因 0.1.9 JSON 格式归一化触发严格字节保护，宿主 exit1。`original/cleanup-result.json` 保留该失败。`original/preference-semantic-verification.json` 核两份完整 JSON 语义相同，候选仍为精确 seeded 字节；没有覆写变化字节。随后 native0 下完整保存新夹具，再恢复两份原物理目录、四逻辑路径及私有原指针，独立核验原文件 ID/创建时刻、偏好/指针/TASK 摘要和 native0。

- `original/01-*.json` 至 `20-*.json`：80 份原始样本，包含进程代际、普通真实令牌、真实端点、WebView 路径匹配、耗时和正常退出。
- `original/comparison.json`：原分析器输出。其完整性能门禁和旧内存失败字段保留原义；CPU/RSS已另获用户接受，本轮只判启动子项。
- `original/human-window.json`、`measurement-plan.json`、目录保存/还原及独立复核：授权、精确映像绑定和配置保护证据。
- `original/ci-*.json`：辅助代码 26840e1b 首次 CI 39050729429/attempt1 整轮 success；产品仍复用 8e，未新构建。
- `method-references.json`：方法绝对路径、大小和 SHA；实现脚本保留在代码 worktree，不复制到文档分支。
- `record.json`、`manifest.json`：结论边界及逐文件完整性清单。

原真实配置内容、指针原文、SID 和数据载荷不发布。原失败及变化的测试夹具保留。本轮不验收完整性能或 Windows 门禁；帧/50ms、原始请求与短命进程计数、100 路由/10 重启、休眠/周期等原缺项继续按 IMP-20 固定表处理。TASK revision2/in_progress/pending 保持；不合并、关闭、签名、晋升、发布或执行 FZ50。
