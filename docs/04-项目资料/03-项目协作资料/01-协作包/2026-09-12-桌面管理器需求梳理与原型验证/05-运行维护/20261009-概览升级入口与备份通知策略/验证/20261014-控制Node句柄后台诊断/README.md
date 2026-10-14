# 控制 Node 句柄后台诊断

2026-10-14，用户分工保持Windows验收、Mac后台。代码feature/0.1.10-integration-candidate@bd2f6eb10d58115235e35d575507bb69f44666da，父51a35eff。此前f545 PRCI三个版本夹具失败原件见[Windows向导路径与父提交CI失败](../20261014-Windows向导路径与父提交CI失败/README.md)：query assign/resume在十几ms完成，约2秒返回Timeout，control和parent启动标记都缺失，Node/宿主/文件写入具体原因未确立。

## 本次有界取证

只改cfg(test,windows)共享查询fixture文件。ControlProcess是此夹具刚创建、std::process::Child保留的独立Node句柄，不加入生产QueryJob；trace仅调用WaitForSingleObject(handle,0)、GetProcessTimes、GetExitCodeProcess，并读取自有control-started.json。记录query前、query返回、原observer.join返回三个时点：PID、elapsedms、0等待状态、进程创建/退出FILETIME及user/kernel累计CPU100ns、exitcode及API错误；即使observer5秒后panic，第三快照也在expect前记录，不失去controls实际状态。

数据可以区分“控制进程已退出/仍活动且CPU几乎未增/晚进入JS标记”等下一线索，不能单独证明加载器、线程挂起、磁盘延迟、杀毒或资源抢占的根因。GetExitCodeProcess STILL_ACTIVE=259；实际原始exitcode必须结合WaitForSingleObject返回一起解释；不得仅把某个数值当存活。代码不把任何快照判为通过，不更改生产query返回/读输出/退出码分类。

没有预热Node、等待其就绪或派新control，没有额外sleep或改变5秒observer、8秒watchdog、2秒生产截止/<4秒清理、默认并发、父子image/creation验证、无关control存活断言。保留原观察与Drop自有清理顺序，observer仍panic导致失败。观察发生前/后有少量API与日志I/O开销，这轮不能作同机性能采样或用后续绿结果推断原因已解。

JS夹具源常量与51a逐字节相同，所有产品代码/前端/注册/配置也相同；本地fmt/diff通过，但Windows专属API不在Mac编译/执行，完整结果只能由新SHAWindowsCI证明。本轮没有编造Windows单测通过，没有启动/激活/退出Mac或Windows原生实例、真实安装、脚本生命周期或性能采样。

## CI与证据边界

new push38537161445/PR38537166560 head=bd2最后in_progress，frontend/release-tools/token job已success、后台Rust仍执行。父51a push38535387208/PR38535398225最后仍in_progress，各实时job和run身份留存；不把观测超时/网络EOF当job终止，也没有取消/重派/重跑求绿。旧f545失败继续封存，若控制快照可进一步定位才按据实原因修正夹具或产品，不降低门禁。

最新产品未打包源码51a与测试诊断headbd2等价产品内容；已验证候选run38524086177/artifact11875775415仍归aae，不含f545真实阶段和51aWindows路径修复。没有新候选dispatch；Windows可继续固定旧aae已有矩阵，新的界面必须新SHA制品复验。PR4经ownerCLI更新回读draft/open/headbd2/merged_atnull，main仍eae，不自动合并。

CLI审阅确认Windows传输层虽已实现namedpipe，get_preferences仍投射cliEnabled=false、save_preferences仍拒绝Windows开启；设置禁用与当前命令行为相符，不能只改UI声称完整CLI已启动，lib.rs也只在启动时按已存偏好bind。能力扩展须有完整运行态/注册/权限证明，本轮没有放开开关或改变偏好。

原TASK3878字节/revision2/in_progress/pending与空验收保持；原macOSCPU+1.817629pp/RSS+117.008965MiBfailed、冷热各20/p95≤100ms且≤5%/10minCPU≤0.5pp和RSS≤10MiB/50ms/有效缓存100切页10重启零自动查询/真实周期及双平台安装备份事件门禁全部开放。FZ50未许可不执行，日常0.1.7/0.1.9及真实数据不操作；Mac原生先协调Windows实例0。goalactive/RCPoverallfailed，无核心晋升、自动合并、签名、发布、stable晋升或完成证书。
