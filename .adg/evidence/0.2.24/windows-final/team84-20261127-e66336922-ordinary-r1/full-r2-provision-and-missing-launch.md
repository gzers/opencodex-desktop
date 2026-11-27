# TEAM84 Full 准备与 Missing 启动回执

旧 r2 采集入口在普通账户运行时到达第 39 行，因 PowerShell 变量名不区分大小写，赋值 $home 撞到只读的 $HOME，返回 VariableNotWritable，未创建回执。本次未重跑 Full。新建只读采集入口，原启动器及其摘要保留。[终端原文](raw/collector-r2-failed.txt)

更换采集入口前，用 Windows PowerShell 5.1 复现旧错误，核对空目录、存在的文件、允许共享读取的打开日志、独占日志出错时保留独立字段，以及禁止覆盖已存在的回执。预检初次误用当前 PowerShell 版本，被版本断言拒绝。第一次 5.1 夹具在带 PowerShell 扩展属性的文本参与 JSON 序列化时未及时完成，仅停止自己的合成探针。把指针改用 .NET 读取纯文本后，5.1 定向检查通过。合成目录已清除。采集脚本对 marenben 为读执行，回执目录可写。[5.1 定向结果](raw/collector-replacement-powershell51.json) [ACL 与清理](raw/collector-replacement-cleanup.txt)

11:21:35Z，marenben 的普通交互会话 3 成功采集。Full 仍是 PID 77372。随包归档存在，运行指针报告 2.99.0／win32-x64，管理器日志记录 integrated runtime provisioned and verified。管理器启动 11:09:44.294Z 至准备校验完成 11:12:34.299Z，墙钟 170.005 秒，不能将整段归为 Status 或解包。该回执没有看到准备过程中的界面，当前测试桌面联网。TEAM84 的断网项仍按约定跳过，目标层的离线实机项仍缺独立桌面。[同账户原始采集](raw/full-r2-collector-receipt.json)

11:21:47Z，另一个沙箱 Missing 使用相同源候选，省去一项随包运行归档并核对其余九项。普通交互会话 3 启动 PID 40624。采集回执先于 Missing 启动，其中 Missing 的缺席字段仅是启动前基线。界面暂缓、重开及用户确认后的下载待核验。[Missing 原始启动回执](raw/missing-r2-launch.json)

采集时 Full 的 ocx.pid、runtime-port.json 未见，仍需同次 Status 的状态与耗时才能判断是否确认空闲。未执行 Start。两处产品现场保留，清理待完成。产品首错待实机状态确认，accepted:false。

2026-11-27 范围确认：用户重申跳过断网实测。按 TEAM84 将目标中的断网桌面项记为 skipped，前文所述环境缺口仅保留为历史。完整包随包准备、缺包确认下载、首次 Status 及确认空闲后的启停继续验收。保留原始回执及 30／5／10 秒预算。
