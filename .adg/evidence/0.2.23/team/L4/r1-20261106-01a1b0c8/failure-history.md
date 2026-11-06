# R1 首批失败与修复记录

保留原始失败，不把修复后重跑改写为首次通过。

| 顺序 | 原始记录 | 结果与处理 |
| --- | --- | --- |
| 1 | native-helper-first.log | 初始4项通过，但目录子ACL测试最初只读内容，随后加强为真正的安全描述符比较。 |
| 2 | native-helper-second.log | 3过/2失败：SetSecurityInfo普通exclusive handle仍改变子ACL；LocalAppData未显式传token时可受子进程环境影响。 |
| 3 | native-helper-third.log | 改为MAXIMUM_ALLOWED句柄的不传播策略、显式token KnownFolder；5项通过。 |
| 4 | product-startup-first.log | 生产crate完整GNU测试编译成功；原始libtest EXE缺Common Controls v6 manifest，载入0xc0000139，测试未执行。 |
| 5 | 初次GNU测试副本调用的工具返回 | RT_MANIFEST修正后副本缺相邻WebView2Loader.dll，以0xc0000135退出；补拷同一构建的loader后运行。此载入失败无测试结果，不能计通过。适配范围和最终hash见gnu-test-manifest.json。 |
| 6 | product-ipc-first.log | 3过/1失败：响应客户端仍持handle，max_instances=1的重建返回231；后续客户端得到pipe不存在。 |
| 7 | product-ipc-second.log | namespace仍有原handle时创建一个替换instance，避免busy与命名空窗；匿名拒绝/恢复及原用例5项通过。 |
| 8 | product-final-build.log | 新增“拒绝后不残留thread token”测试中的raw pointer局部跨await，不满足tokio::spawn的Send；属于本线新增测试编译错误。 |
| 9 | product-final-build-retry.log及final原生日志 | 将同步token断言限定在await前的独立作用域；完整GNU libtest编译成功，最终97项通过。 |

独立driver的5项和其后生产crate的10项有重叠，不叠加计入97。各完整日志包含源码根与工具链位置，属于本次实际环境，不表示其他主机已复现。最终文件摘要及编译/运行范围归verification.json和manifest.json。
