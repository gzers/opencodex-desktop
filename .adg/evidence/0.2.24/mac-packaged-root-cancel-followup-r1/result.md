# Mac ARM64 Release 打包 GUI 定向验证

源码 `57073c4e5df13294e9c067ee86bf4535ded7f9b8`，官方 OpenCodex 2.99.0，隔离合成配置。包内可执行文件和四项运行包资源摘要见 [summary.json](summary.json)。包经过严格深度 ad hoc 验签。离线供应回执记录 `codeExecuted:false`，实际启停由以下 GUI 场景执行。

| 场景 | 索引变化 | 界面观察到取消 | 正常 Stop |
| --- | --- | ---: | ---: |
| 普通准备取消 | `preparing@1 → root_cancellation_absent@3 → ready@7 → root_absent@12` | 5,854 ms | 7,565 ms |
| 部分捕获取消 | `capture_reserved@2 → root_cancellation_absent@4 → ready@8 → root_absent@13` | 6,621 ms | 7,860 ms |

本轮检验两个预测。Ready 锁短暂争用后重试，应在原 Stop 期限内得到终态。写租约结束后刷新状态，应在一次显式只读复核后允许后继启动，正常停止后恢复 Start。两条路径均达到预期。Stop 记录确认原生进程退出码为 0，四个已捕获监听均已消失，完成卡自动收起。部分捕获的三份快照字节相同，覆盖内容、摘要、文件身份、模式和所有者。此轮未单独计量完整 Start 耗时。

两轮测试根、进程和监听已清理，每轮三份测试协调器文件归档后移除。管理器测试进程收尾为 -15，服务正常 Stop 的退出证据单列。首轮打包因 zsh 的 `path` 变量覆盖 `PATH` 在资源校验中退出 127，失败记录与脚本保留。已清理约 276 MiB 的不完整副本并保留通过验证的候选包。既有 IMP220 GUI 失败、IMP221 Debug 11,642 ms Stop 超时及 IMP222 Release 原生宿主双代通过分别保留。

定向 Rust 检查 3、10、21 项通过，协调检查 1 项忽略，独立续接检查 1 项忽略。UI 构建通过，116 项 UI 测试通过。原始文件留在本机，摘要见 `localRawEvidenceSha256`。本轮 `accepted:false`。Developer ID、notarization、更新签名和最终候选绑定待完成。
