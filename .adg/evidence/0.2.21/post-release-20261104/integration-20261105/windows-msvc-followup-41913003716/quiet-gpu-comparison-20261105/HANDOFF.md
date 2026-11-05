# 协调空闲时段对照：改善，但原性能门仍失败

用户“现在就可以”后，本地2026-11-05约15:50开始同一存活MSVC候选fa9df83a/manager75988/GPU76176/RDPsession2对照，哈希重新核对，测试根/高档/暗色/动画/玻璃配方及字号布局保持，无GPU启动开关、drawcap或低档替换。原QA静止和滚动各30秒、弹窗30秒全部真实前台；p95分别84.8/45.4/78.2ms，max174.8/81.7/174.8ms，over6B20/0/6。B=16.6667ms/p95≤2B/over6B0门保持，三场景均failed，失败JSON原件随附。

补采第一次在用户回信之前提前开始，nativeforeground不满足，只有unpaired失败scene；录制正常stop，363855872字节原ETL留本机，不算有效配对。第二次scratch入口加明确release文件，绑定新采样PID/create，收到用户回信后exactpath原生窗口置前、实际稳定foreground≥3秒才写release，随后标准collector再次核身份并45秒录制。scene16:00:57.622至16:01:27.643全部30秒在录制内，p95=78.4ms/over6B16/foregroundtrue，高档保持。正常stop、367001600字节ETL SHA aa8c78619fdd20b3c2178e562011b69c87656f4f874672e17c8cf8ce1fd200f8，仅本机保存。开始时间已经超过最初“约10分钟”窗口；用户对第二次运行及60秒前台明确回信，但其它会话保持空闲的连续性未证明，不写成严格空闲GPU验收。

原生ProcessTrace3223711事件exit0，头EventsLost0；原配方/非分页/无系统调用栈采集保持。按sceneUTC裁切，Device/Context绑定owner，硬件Queue178/PacketType0的submit→firstDMA p95由639.5639降至160.0615ms，submit→complete由668.4418降至189.189ms；DWM DXGI Present壁钟p95由226.3144降至49.9225ms。15079组自有角色相对时间CSV无地址，独立重分组复算、dur=end-start、窗口边界和重复start0检查通过。原对照baseline为14:50同一EXE/存活manager及GPU进程，精确区间见comparison，不把其它二进制旧滚动样本冒充严格对照。

重要限制：所谓协调空闲并不等于硬件节点真正空闲。其它会话node0 DMA墙钟区间并集仍22.975秒/30秒，旧baseline18.386秒；这是墙钟区间，不是GPU硬件利用率，不能据此称负载已降低，更不能将帧时延变化唯一归于其它用户暂停。两次时间之间新增Administrator正式测试客户端等也不是严格控制变量。当前证据证明本应用排队/呈现等待改善但仍显著，不证明已排除环境争用或生产实现已修。未停止其它会话程序、断开用户或修改策略/驱动。

另采8秒EXT_disjoint_timer_query诊断40个可用区间，p95=8.466406ms/max9.767187ms，原gl.drawArrays已恢复。只说明这段诊断GPUelapsed，不替代30秒门，也不当作GPU利用率或单独着色器成本证明。GPU计数器初次路径转换失败原件保留，通过系统枚举路径重采3组，时间15:53:38-40，原始机级counter仅本机，不能冒充与三场景全程配对。额外settled30秒helper发生换行引用解析错误，未进入执行/未采样；node--check复现错误保留，不编造样本。

两张已显示的ComputerUse原生Windows窗口JPEG按原格式归档，分别为初始三场景前及最终放行前；不是headless/DOM截图，也不称产品修复前后图。录制于16:01:52确认stop后已告知用户可恢复桌面，后续离线分析没有再占用前台。源码产品apps仍a8bb18f9ff2781dbbfe67f91473a298f3cd74ae6，本轮没有生产材质改动或新性能通过项。

TASKin_progress/RCPfailed、无CERT。未完仍为性能生产修复及原预算矩阵、官方2.84默认10秒进程探针修复后的实际默认重启和读回；不合并main、不发布、不真实同步。历史误停事件原件继续保留，不作全批零副作用认证。

采集开销补充：初始滚动场景可能与系统GPU计数器集合枚举重叠，未独立测其CPU扰动，三场景初采不冒称无扰动实验。最终配对静止段与baseline使用相同显式WPR配方，未与counter枚举或WebGL timer诊断重叠；主要队列对照以该段为准。
