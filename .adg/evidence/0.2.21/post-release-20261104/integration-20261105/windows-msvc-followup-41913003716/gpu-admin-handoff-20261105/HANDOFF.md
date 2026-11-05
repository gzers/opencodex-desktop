# GPU/DWM管理员协作准备（未执行采集）

用户已明确“有管理员或测试环境可协助”，当前待确认本机管理员手动运行，还是另一台独立Windows环境。此前0xc5585011录制失败原件保留；本轮没有自动提权、没有修改策略/驱动/日常配置，没有启动录制或新的测试应用。

代码提交及两个工具的精确SHA见preparation.json；工具均在test/smoke/post021-windows-final，产品apps树不变。collect-gpu-trace.ps1与write-gpu-target.ps1为可审阅的本机方案；另一台测试机需先明确该机仓库及临时根，再适配路径准入，不将当前主机目录假定到其它主机。

## 本机协作流程

1. 由本对话按已有隔离方案启动已核验MSVC候选；确认唯一原生窗口、用户前台可用时段和实际进程。当前没有存活测试进程，不得沿旧PID采集。
2. 在普通测试账号生成一次性目标身份：write-gpu-target.ps1读取显式测试launch.json，核对实际PID/路径/EXE SHA，写CIM创建UTC ticks、session、owner。只允许两份已核验MSVC哈希和固定测试根；过期PID拒绝，文件不覆盖。
3. 运行collect-gpu-trace.ps1 -PlanOnly校验新身份和新输出目录。它读取WPR状态并保存拟执行参数，不开始录制；计划输出与实际采集输出使用不同新目录，不能覆盖旧证据。
4. 管理员从自己打开的shell手动运行相同脚本，指向那份新身份与新采集目录，-DurationSeconds 30（允许5～45秒）。脚本不会替管理员打开提权提示或自动授权。具体绝对路径命令须在确认协作方式与新目标存活后生成，不能用占位/历史身份冒充可运行实例。
5. 只采GPU及DesktopComposition调度/合成，不请求FileIO/Network/Registry profile。先保留collector-start-intent.json与恢复命令，成功启动后写collector-started.json；普通测试会话保持目标前台，按约定场景采样并记录时序。采集时间与30秒验收样本没有完整重叠时，不认证对应整段GPU证据。
6. 正常/异常结束后只stop本次独立WPR实例并写gpu-dwm.etl、摘要和结果。-instancename为随机ocx-post021-gpu-UUID，按本机wpr -help advanced要求置最后；默认/他人命名会话都不归本脚本。没有cancel、进程kill、配置恢复或日常实例操作。既有默认录制、未知状态、开始拒绝或目标变化都失败并保留日志；WPR命令20秒未返回时保留未知状态及精确命令PID，不能盲停其它会话。
7. 原始机器级ETL只留指定本机临时目录待审阅；先分析目标PID、其WebView2后代、目标session的DWM/GPU等待，并回交精确摘要及过滤后证据。不自动将其它程序元数据或完整机器轨迹推到仓库。

## 已实际检查

PowerShell解析通过；5项路径准入检查通过：合法子路径、边界自身拒绝、同前缀邻目录拒绝、..逃逸拒绝、目录联接祖先拒绝。junction测试夹具留在受控.toolchain目录，未递归删除未知目录。旧MSVC launch.json对应PID已退出时，身份生成器实际拒绝，未写身份文件、未操作应用或WPR。

有效存活目标的PlanOnly、管理员实际录制、GPU/DWM分析与性能修复均未执行，不能计passed。也不因此关闭正式Codex默认adapter的一次性账号/会话隔离缺验。原性能预算、玻璃配方、档位和日常进程边界保持，TASK in_progress、RCP总体failed，无CERT。
