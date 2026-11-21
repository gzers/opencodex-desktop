# macOS ARM64 Release 两代正常启停定向验证

2026-11-21。假设为固定源码在 ARM64 Release 原生宿主中能完成两代正常启停，两次均达到独立读回的 root_absent，前代归档保持原样。实际结果支持该判断，测试进程退出 0，官方 OpenCodex 2.99.0，整版 accepted:false。

| 步骤 | 第一代 | 第二代 | 预算 |
| --- | ---: | ---: | ---: |
| Start | 7,278 ms | 9,049 ms | 30,000 ms |
| 运行中独立 Status | 220 ms | 715 ms | 5,000 ms |
| Stop | 5,317 ms | 8,152 ms | 10,000 ms |
| 停止后独立 Status | 1,025 ms | 1,614 ms | 5,000 ms |
| 恢复终态 | root_absent rev9 | root_absent rev18 | |

两代均恢复原本缺席的客户端目录状态，独立查询确认无运行记录。第二代启动及停止后，前代绑定事件与保管归档未变。测试夹具、进程、监听和本轮新建的三项协调器清理完成。原 Debug 超时保留在 [IMP221 的失败证据](../mac-root-stop-archive-observation-r1/result.md)。

[机器可读结果](summary.json)固定源码基线、补丁和制品摘要、原始证据摘要与清理结果。原始证据保存在本机受限目录，实屏与正式签名候选另行记录。
