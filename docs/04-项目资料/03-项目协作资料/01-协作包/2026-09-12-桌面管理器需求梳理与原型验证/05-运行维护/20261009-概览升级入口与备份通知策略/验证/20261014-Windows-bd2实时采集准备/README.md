# Windows bd2 失败制品与实时采集准备

2026-10-14（UTC+8），用户已在宿主设置新完整目标；工具读回 `active`，原createdAt/预算保留。本轮按原IMP/TASK优先准备有界失败取证，没有重跑原库、启动Node/desktop或ETW，没有以准备完成关闭版本。

## 原覆盖纠正与失败制品

Mac b14a0f6复核指出：019dd6c的六Node首次采样时全部退出，474条Node记录均Wait0=0，Node活跃线程/WCT均0；全部2664线程记录和80次WCT来自测试主进程。Windows本轮按path-keyed映射复核35份原件摘要，并独立重算确认；原件/旧清单/方法不改。它只有成功对照与测试根线程，不能解释Node启动等待。原verifier在Mac的失败来自路径列表排序，不是字节漂移，后续清单按相对路径字符串排序/映射校验。

重新取得失败PR CI38537166560的实际测试EXE，而不是重复使用成功push的EXE。实际checkout为31e7748d9e82c79bef97fc1b73233f08bc1f1b3d，关联head为bd2f6eb10d58115235e35d575507bb69f44666da；两者整tree均753be1611f9e96e6077a4a164937b5bd6f42f0cc。原checkout日志、API归属、ZIP摘要/CRC和runner测试EXE回执绑定。

失败轮次测试EXE SHA497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3，与成功push2709775e…不同；源码tree相同不等于二进制字节相同。未运行失败EXE或据此推断差异是根因。官方Node仍固定24.21.0/SHAba4e6d11…，没有预热。

## 准备工具

`feature/0.1.10-windows-query-etw@5fa36a60a6867124cf13f27e1e1856ae495ef29b`只新增 `test/smoke/windows-owned-etw.cs`、`.ps1`、`.test.ps1`，基于bd2，不改apps、原夹具或CI工作流。草稿PR#5以integration-candidate/bd2为base，未合并。

固定Microsoft TraceEvent3.1.21通过NuGet官方metadata SHA512、ZIP CRC核验，仅解出本线程runtime的DLL和XML；没有全局安装或执行包脚本。包内仓库签名存在但没有独立密码学验签，不混同SHA512验证。v3 catalog返回404后从NuGet v2官方metadata取得摘要，HTTP gzip响应显式解码；准备过程错误保留，不记作产品失败。

本机Add-Type编译通过、38项纯生命周期/时间戳/PID/TID重用合同通过；默认准备、摘要不符拒绝、输出越界拒绝、普通权限在任何会话/进程创建前拒绝均通过。QPC API被库标为discouraged（不是不支持），只在原时间戳辅助函数局部处理该警告；没有更改原Rust/产品警告或预算。

首轮CS0618停错仅在工具输出可见，PowerShell重定向文件实际为0字节；保留空文件并另记错误摘要，不称已有完整stderr归档。首轮证据校验错误期待该日志含文本，已按真实文件情况纠正，未重造失败日志。

**尚未执行实时事件采集和排空**，上述检查不证明实际事件覆盖。新分支自动CI状态读取失败，不能声称CI通过。没有手动重派CI/候选或求绿循环。

## 可审阅的管理员操作

完整入口位于 `.adg/evidence/OCX-0110-20261014-WIN-BD2-CAPTURE-PREPARATION/record-request.ps1`。

默认执行该文件只显示方案；只有用户明确批准、手动取得管理员PowerShell7且传 `-ApprovedByUser` 才可能录制。它核对collector HEAD/无局部改动、测试EXE/Node及准备DLL摘要、本机desktop=0和新的输出目录，不自动提升、不使用Run对话框或更改执行策略。

拟录制只执行一次原完整测试库，默认并发/capture、2s query/<4s cleanup/5s observer/8s watchdog均不变，不预热。原库外层最长等待120秒，额外排空/停止有短时收尾；这个外层保护不是延长原查询预算。

- 在原测试启动前启用唯一命名实时system logger，不替换其他会话、不写整机ETL。
- 系统内核事件会短暂进入内存解码；持久化只接受自有测试/命令/精确Node家族及原始QPC生命周期，不宣传provider本身已经按PID只生成这些事件。
- 只存自有线程调入/调出/ready、加载和文件I/O字段，省略其他调度参与者ID、完整命令行、SID及无关路径；超出夹具/Node/系统加载目录的路径会省略。
- 每次Node生命周期尝试保留实际句柄并记录映像/创建/退出时刻；回调到达时已退出或无法核验必须如实标注，不能补造活进程证据。
- 不串行WCT，不开网络/registry provider，不抓文件内容；输出上限64MiB、队列/待完成IRP均32768，丢弃、解码错误、缺覆盖都不能算完整证据。
- 最终只停止自己的会话并排空消费者/写入线程。丢事件计数目前是stop前快照，不能写成最终关闭全过程零丢失。

管理员控制器的令牌会由测试子进程继承，实际令牌和原始结果须记录。用户批准仍待，没有Start-Process RunAs、自提升或实际录制动作。内核短暂内存筛选与仅保存自有家族的范围应让用户确认；若不接受这个范围，应改用专用隔离录制环境，不能在日常桌面启动全系统文件录制。

## 完成与未完成

本轮完成原覆盖独立核验、原失败测试EXE准入和可审阅采集器/命令准备。失败场景调度/文件I/O原始跟踪仍缺，根因未识别。未来必须按实际失败case/control PID、创建时刻与ETW时间窗联合审核；仅LICENSE路径失败或全部查询成功不算复现，不能无限重跑直到求绿。

Windows desktop/官方Node当前均0，没有正式Mac原生交接。原完整门禁、性能failed/预算、FZ50许可缺口、日常版/真实数据保护及原TASKrevision2/in_progress/pending保持；未关闭、签名、合并、晋升或发布。采集批准后将结果写新runtime，再独立封存，不能覆盖本轮准备证据。
