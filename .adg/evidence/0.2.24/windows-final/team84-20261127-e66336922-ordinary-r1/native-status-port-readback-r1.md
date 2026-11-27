# TEAM84 Windows 原生定向复验

固定源码 e663369226337c9a9cad594ccd3cafd1719c3711，在新分离工作树以 x86_64-pc-windows-msvc 和 Cargo 离线锁定依赖构建。沿用既有 UI 构建目录作为编译输入。测试未启动桌面管理器、正式运行时或原包 Doctor。

待证伪判断是原生 Status 会在没有 Owner 时给出可信状态，或者 Start 在 PID 记录拒绝或端口写锁超时时写入端口设置。生命周期定向测试 6 项通过、4 项按现有标记跳过，端口设置定向测试 1 项通过。Status 使用传入的截止时间，已过截止时间返回 Timeout，无 Owner 返回 Unreachable。PID 文件拒绝和写锁超时均未发布端口设置，自动端口设置测试保留原始文件内容。见 [Cargo 输出](raw/native-status-port-readback-r1.log)。

本地组件计时不计入普通交互桌面的首次 Status 或 30／5／10 秒实机验收。实机可信空闲、首次 Status 计时、缺包同次确认下载证据仍待补齐，Start 未执行。Doctor 共享故障见 [源码复核及最小交接](doctor-source-readback-r1.md)。

只读复查时，历史 Full 管理器 PID 103156 仍在 marenben 会话 3。新组件夹具的 scratch 目录已空，测试进程已退出。工具自动审批拒绝了删除本轮目录及生成文件的清理命令，结果为 blocked by policy。保留分离工作树 team84-native-readback、其空 scratch、两个自动生成 schema 文件状态及用于编译的 UI dist。原现场保持取证状态。accepted:false。
