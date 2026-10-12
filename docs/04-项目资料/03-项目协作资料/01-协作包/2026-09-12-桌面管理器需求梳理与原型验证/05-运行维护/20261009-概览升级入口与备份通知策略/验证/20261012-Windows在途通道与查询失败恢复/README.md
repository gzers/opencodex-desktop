# Windows 在途通道与查询失败恢复

2026-10-12，原根唯一 `main@afff490` 候选 PID29936。通过生产 Pinia store / 原生命令调用真实 npm 元数据来源；没有替换响应、事件、时钟或 Tauri 桥。这些是命令驱动的真实原生场景，不冒充鼠标交互或实际安装升级。

## 面板查询在途切换管理器通道

证据：`.adg/evidence/OCX-0110-20261012-WIN-INFLIGHT-CHANNEL/`。

16:54:58.705 开始真实 `loadRemoteLatest('user')`，约100ms后发起管理器 stable→beta。原生切换完成时仍观察到面板 loading=true、promise未完成；随后面板于16:55:00.460完成，单调耗时1754.8ms。

切换前后在途旧缓存均为已取得的2.83.0，查询最终成功、loading=false/error=false，管理器仍为beta。落盘管理器stable截止完全不变，面板查询成功更新自己的last_success和86400秒截止。最后恢复stable并重新加载偏好store，auto-check仍false，偏好文件前后SHA一致。只覆盖这一次在途元数据查询，不扩展为在途安装/确认/重启或全部方向与错误矩阵。

## 实际连接失败与手动恢复

证据：`.adg/evidence/OCX-0110-20261012-WIN-PANEL-EVENT-RECOVERY/`。目录名保留最初持久事件观测意图，实际结论按本节收窄。

隔离HOME原本没有`.npmrc`；临时写入仅测试用`registry=http://127.0.0.1:9/`、零重试、1000ms fetch timeout及ignore-scripts=true。真实npm查询失败后，前端error=true/loading=false，上次有效remote和原生缓存保持；调度落盘failures=1并保留原last_success（该规则对应5min退避，不是已等待5min的周期证明）。

同期真实管理器查询返回up_to_date；恢复原本不存在的`.npmrc`状态后，再次手动面板查询成功，error=false、failures=0，next_due=last_success+86400。偏好字节不变，没有执行包生命周期脚本。

没有生成持久面板失败通知。核对精确生产注册表，panel-check-failed/succeeded为active，但policy=null、resolves=[]；四个通知样本完全相同。因此本次只证明查询错误与缓存/截止恢复，不作为“持久通知解除”通过。未擅自增加通知政策。

两目录分别冻结原始文件清单，并无参数重验7/12个文件通过。完整通道、持久事件、更新安装、原生双权限/完整物理DPI、配对性能与真实周期仍开放；原TASK/脚本二次确认、预算和既有性能failed不变，IMP in_progress，不关闭或发布。
