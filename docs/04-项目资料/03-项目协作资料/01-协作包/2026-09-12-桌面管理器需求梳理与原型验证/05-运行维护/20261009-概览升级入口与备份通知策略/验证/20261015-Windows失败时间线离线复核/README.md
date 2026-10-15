# Windows 查询失败时间线与等待字段边界

2026-10-15，Asia/Shanghai。前台窗口结束后只做离线复核。源码仍`feature/0.1.10-windows-query-etw@493da80675e6508e46e0fd83d86ee7216abf96fe`，没有修改产品、夹具、采集器或工作流，也没有新Node、原库、ETW、原生实例或安装执行。

## 五轮原日志对齐

`.adg/evidence/OCX-0110-20261015-WIN-FAILURE-WINDOW-REPLAY/`按显式路径读取11份既有输入，逐项摘要并独立核对解压日志与原回执的摘要。重算5轮库运行、35条夹具时间线；同一control的三时点PID、创建FILETIME、Wait0=258、exit259/exitFILETIME0和API无错误均核验。Windows夹具源码自bd2至493无变化，但不以源码相同推定不同run的EXE相同。

| 历史run | 库通过/失败/忽略 | 原始控制标记事实 |
| --- | --- | --- |
| bd2 PR38537166560 | 523/7/4 | 七组到observer结束5.051–5.657s仍缺control标记，累计CPU0–62.5ms |
| bd2 push38537161445 | 530/0/4 | 七组query返回前已读到标记，发布距实际创建72.592–629.753ms |
| 38572890687 | 527/3/4 | 三组version查询返回时缺标记，observer返回时已有；标记距创建4.182–4.252s |
| e6 PR38583689702 | 527/3/4 | 三组version同类失败；标记距创建3.213–3.334s，早于约5s observer结束 |
| e6 push38583679031 | 530/0/4 | 七组query返回前已读到标记，发布距创建55.518–489.534ms |

两轮后续失败中，version三个场景都是Timeout，query返回读取的父进程/子进程/ownership-ready等阶段标记全缺；metadata四组已有control标记。它们与原bd2七组到约5秒仍缺标记是不同观察结果，不统一写成“所有失败5秒仍未启动”。保留原两秒query、四秒cleanup、五秒observer、八秒watchdog及默认并发，不以control稍后写出标记将查询超时改成通过。

时间轴用夹具内嵌unix-ms、GetProcessTimes创建FILETIME和原elapsed，不用GitHub日志行前缀当执行时间：libtest会缓冲捕获输出，前缀对应日志交付时刻。标记缺失不能单独区分Node加载、JS require、线程调度、文件写入/重命名等阶段，也不能认定系统防护或产品根因。

## 成功ETW的等待字段解读限制

原CI诊断run38662786466的事件原字节SHA `a4f4260a3a58d812a1e17a436364c2616e1a9e8fb8c9c40086cf116c4f76b1d8`重核。3,175条scheduled-out均为CSwitch version5，其中402条waitMode没有固定解码器枚举名称，原字符串为16/17/80/81/96/97等数值，原值完整保留。没有掩码、重映射，也不把96解释成KernelMode。若仅凭枚举标签认定等待原因，会超出已有证据。

只读核验Microsoft PerfView固定v3.1.21源码（565,319bytes/SHA `5a1ff963eb4c5b6dbe8845843c841e8466b5c4855e72b454c7a0cf31ea4a708f`）：该getter从偏移13取字节，ThreadWaitMode只命名0/1。Microsoft Learn公开CSwitch表描述Thread_V2，不能据此认定本轮version5字段语义已经验证。来源、定位、摘要与解读边界记录于`decoder-source-review.json`，没有复制第三方完整源码或修改固定DLL。

成功录制里7个不同control PID均有本地control-started.pending写事件，说明现有采集范围可以保留这类文件进展；它们属于成功对照，不是原失败进程的原始I/O。当前分析保留所有原线程/调度/I/O记录，新增的是解读边界，不宣称修复了采集器或原产品问题。

## 下一次失败取证与当前状态

下一次有界方案须从自有进程/线程/映像创建开始覆盖，持续到原五秒observer，不只看query返回；按PID+创建时间/原始QPC区分control、查询worker及标记I/O，保留未知等待字段数值与版本。应在自然可能失败的首次执行路径取得失败原件，不自动重复已经成功的固定制品回放，也不预热、改变并发或放宽截止。此页没有派发新CI或授权额外采样。

`analyze-failure-windows.py`与`verify-archive.py`通过，新增8项归档清单和11项输入摘要。原提交019dd6c2的35项verifier重验通过，仍为本机七query成功、没有失败调度ETW；其输出goalStatus=paused/native0是历史快照，不是当前目标/实例状态。

13:22:54独立检查点仍是原管理员PID78512、创建12:47:46，native1，退出码未知。未强杀、未启动第二实例或Mac原生交接；普通CIM对提升映像路径为null如实保留。日常EXE、候选EXE、旧隔离四摘要、原TASK3878bytes/SHA bc7965b4…和493干净代码均保持。

实际失败窗口ETW及根因仍未取得；原bd2/e6失败、Mac CPU/RSS超预算、完整原预算/样本数、安装与Profile环境、更新/备份/树表/事件/真实退避周期及全部八门禁不变。TASK/IMP继续in_progress，不关闭、合并、签名、晋升或发布。
