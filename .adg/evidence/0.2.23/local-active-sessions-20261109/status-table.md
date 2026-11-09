# 本地双活动会话诊断（2026-11-09，Asia/Shanghai）

accepted:false。基线2c6edebc32909c9a16e6666836c22ca6cf05037d；程序构建42547758411，ZIP302488994字节，SHA256 b182dc5877d995fbfc8ee856285a4f62fb35489a22630ae7325813510db4cb8c，6个内部成员核验。程序apps树db4ac662f6216626c6dae2aae7f1d49866871052，与R33相同。运行包保持2.97.0-desktop023.names.2及fcd2740ecc32ae1e11c1c79d84c801b2c5d5447ae89e070ab060548d61f75ee6，未更换字节。

这是一轮真实SID/会话/提升状态下的隔离Sandbox诊断；不能替代真实OS Profile下的用户根、DPAPI/IPC或正式安装验收。15119属于管理员组但实际进程未提升，不声称它是“非管理员组标准账户”。Administrator实际控制台SID尾号500/会话1/提升true/type1，15119实际RDP SID尾号1001/会话2/提升false/type3。两会话同时Active，原生PID/映像/FILETIME/CIM SID及实际令牌分别采集。

|项目|实际结果|结论与证据|
|---|---|---|
|R33历史全表|原26过/2失败不变|原Start30134ms、补充Stop10059ms失败保留；本轮不覆盖它。|
|双方程序集成来源|通过本轮准备|sessions-04双方原生离线供应exit0；实际runtime_source与model_paths均managed/consistent，环境ready。不是服务ready。|
|RDP实际Start|本轮通过|15234ms，原30000ms门，PID40612/自动端口53147，实际HMAC/health/配置路由写入；进程SID…1001/session2/未提升。|
|RDP实际Status|本轮通过|384ms，原5000ms门，PID40612/53147及healthy；后续observe同代次。|
|控制台固定端口冲突|本轮通过|360ms返回typed port_conflict/starting_failed/result null/canStop false/canRestart false；原30秒内本侧记录缺席。RDP仍同代次、原监听和HMAC健康。|
|控制台自动端口Start|失败|原Start30011ms；自己record/HMAC=PID10172/49500，Desktop status却显示peer40612/53147；55次官方就绪观测unreachable。迟到ready不补判。|
|控制台补充Stop|失败|fresh H/record/HMAC后独立Stop，Manager191ms报lifecycle_stop_unknown；驱动观察2258ms。不是正常Stop通过。|
|RDP正常Stop|失败|Manager5143ms报open recovery process identity/拒绝访问，driver manager_stop_not_confirmed。随后PID/端口消失不能补判预算内全部证明/客户端还原通过。|
|双方服务互不干扰|未完成|固定端口冲突阶段RDP保留有证明；完整双服务成功重叠及一方正常停止后peer保留未取得。控制台服务随后由夹具错误清理，不能归因RDP跨SID误停。|
|控制台托盘隐藏/重开|本轮通过|用户明确回报成功；同PID51540/同HWND重开后的原生visible检查及app.log tray/menu事件。|
|RDP托盘隐藏/重开|本轮通过|用户明确回报成功；同PID98792/同HWND与app.log事件；日常PID41396未作为测试对象。|
|RDP托盘正常退出|本轮通过|用户操作，reply-5原句柄exited true/exit0；finished.normalManagerExitVerified true。|
|控制台托盘正常退出|独立补验通过|原sessions-04被夹具失败清理，不算正常退出；替换后的console-exit-only02同基线/同SID/同提升/session1，PID19696原句柄退出true/exit0，有用户确认，临时任务移除。仅管理器退出，不补服务停止门。|
|原预算和恢复退出断言|保留|原驱动仅3处Sandbox路径绑定替换，原30/10/5秒、HMAC、SID/映像/代次、还原与退出检查保留。原134离线合同通过，新入口12检查通过；不是整版验收。|

## 失败原因与已实施范围

1. 42545767548原生只读原型实际失败，原15671907字节ZIP/SHA256 d17068cc56edd3da1c0cc8064e7c81b099bc624229306c33aeaf235e357edd50及28成员核对；原stderr token-pointer保留。固定Bun1.4.4本机原样复现同错。小Uint8Array被取.buffer时底层存储物化导致API写入的SID指针相对原地址有效、相对新地址越界；对照日志不含实际内存地址或token内容。
2. 0eb42ebc0只修test原型，先显式分配ArrayBuffer/DataView再调用API，检查返回长度及地址不变，保留SID/FILETIME/完整命令行边界。三个自有cmd精确命中/相似后缀/另一HOME三轮完整通过，23.76/18.05/14.64ms，无unknown、无终止API；仅技术验证，不是维护包采用或真实Stop修复。
3. 4fff1d5ca修Desktop的未认证CLI兜底：managed live必须走本H的受保护record/HMAC/ready链；没有本H记录时，仅接受明确本H、无PID、无矛盾live字段的Stopped诊断，并重新检查本H记录仍缺席。外账户监听发现不再给当前Manager运行身份。正常Stop仍需原OS归属/还原/退出证明。
4. 普通Stop在枚举全机runtime形态进程时碰到不可读对象。只读对照证明某PID在0x101000及仅0x1000均error5，WTS同对象SID null。未采用“跳过拒绝访问”“按会话推断SID”或把未知当缺席；没有修改停止归属或运行包来伪造通过。后续需要按H保护租约/退出证明的完整方案、反例及用户事先知悉的维护修订。
5. 控制台提升启动受保护record为何不可达尚未证明；不得只按推测把Administrators owner当作实际证据。保留原启动/补充Stop/迟到ready/错误peer状态。

## 精确返修验证

原型普通令牌CI42552474525成功，原ZIP15672139字节/SHA256 a852f012d8197e2fbb27ef83dbcb71500e9596ecf736406ba9e01bff0e76e544、28成员与真实exit0/账户Profile移除/ACL恢复核对；完整三轮4.55/8.19/3.54ms，未知0。原42545767548失败不覆盖，未把技术验证写成运行包已采用。

状态保护首轮42552474670的精确Windows源码aa559ddb8：3新增状态准入+8原认证状态/停止+5归属测试通过，1原专用进程用例显式忽略不变。整个job失败于执行器错误预期“6 passed”，严格Clippy尚未执行；3640字节原ZIP/SHA256 4dc30c280c56153f7fcfbfae01661eaaca69cbbb77e0da11596dceb60d791250与日志在status-first-ci。77a641618只改实际计数5并核对1原忽略，apps树与4fff1d5ca相同；后继42553369183定向回归/严格检查另附。不声称首轮job成功或返修已完成真实双服务验收。

## 夹具失败与清理

sessions-01：公开恢复准备分别报备份文件不存在、进程身份拒绝访问，且驱动退出/CIM采集竞态使整项失败；没有服务Start。sessions-02/03：先供应D导致产品拒绝“nonempty unrecognized directory”，原退出101/stderr保留。sessions-04先由真实产品初始化D、停止该自有初始化代次，再由原供应函数首装，不改产品目录保护。f19bafcb6补此顺序、短命驱动身份握手、原句柄及还原检查；transport握手不改产品时限。

sessions-01/02/04实际自有进程已退出，私有目录移除，临时控制台计划任务移除，双方daily配置前后摘要一致。sessions-03控制台亦已清理；RDP WebView cache journal曾被占用，原清理失败保留，未生成虚假finished。D:/ocxd023-r34-03/rdp/private的两次精确自动删除被审批blocked by policy拒绝，没有绕过；用户第一次报告删除后10:58实际核验仍存在，operator-cleanup-confirmed.json如实为false；用户再次按准确路径删除，10:59:34实际核验缺席，operator-cleanup-final.json为true。原失败、审批拒绝、首次不一致与最终人工清理全部保留。日常程序、真实业务同步和发布未操作。

正常退出补验首个console-exit-only在用户操作前被执行器30秒观察包络超时清理，属于执行器失败，不冒认产品失败或通过；原件保留。替换console-exit-only02改为人工作业等待包络10分钟（不改变任何产品启动/停止/状态预算），用户操作且原句柄exit0。两个退出夹具私有目录与清理任务均在console-window-cleanup.json及console-window-cleanup-task.json实证移除。

本轮原始记录与evidence-manifest逐项SHA在同目录；最终定向CI结果另附，不用构建成功/局部回归替代以上失败。
