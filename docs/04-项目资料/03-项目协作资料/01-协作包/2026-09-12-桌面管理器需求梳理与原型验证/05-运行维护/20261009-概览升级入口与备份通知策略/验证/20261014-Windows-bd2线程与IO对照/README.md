# Windows bd2 自有进程线程与 I/O 成功对照

2026-10-14（UTC+8），用户授权接续本机诊断，固定 `bd2f6eb10d58115235e35d575507bb69f44666da`。本轮没有修改产品源码、原夹具、截止、断言或并发，没有再次触发CI、候选打包、安装器或桌面实例。

**结论：取得一次本机成功场景的原始线程/WCT/I/O采样和夹具文件通知；没有复现历史CI失败，没有取得失败场景调度/文件I/O ETW。**原PR CI38537166560失败与Mac性能failed继续保留，不能由本轮成功对照撤销。

## 来源与单次执行

独立push CI38537161445的实际Windows checkout日志记录完整bd2 SHA。API制品归属、ZIP摘要/CRC与runner中的test-executable-sha256一致；测试EXE26782720字节/SHA2709775efbce5a08a02827c61a74abb1adfb3170653898b8c614eebf9c367400。测试EXE不含提交号字面量，初始准入因此停止；随后以实际checkout、同源API制品和runner测试EXE摘要绑定来源，未将其混同PR合成检出或桌面候选。

CI Node为24.21.0，本机下载官方对应Windows x64 ZIP，只提取Node；官方SHASUMS、ZIP CRC、Authenticode Valid/OpenJS Foundation一致，EXE SHAba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32。未安装npm、执行生命周期脚本、改变全局PATH或预热Node。

17:33:37.903至17:33:48.585执行一次原完整库测试，无筛选、无 `--test-threads` 或 `--nocapture`。534项中529passed/1failed/4ignored，exit101；七个version/metadata success/nonzero/overflow/timeout查询夹具均passed。唯一失败 `commands::workspace::tests::local_documents_are_frozen` 的 `license.opened` 依赖编译时CI仓库路径，本机没有该路径；不创建假runner目录或改断言求绿，不计作查询失败。

测试根PID11640/creation FILETIME134364440179039668，外层保留句柄，最大120秒诊断保护未触发；原2秒查询、4秒清理、5秒observer/8秒watchdog保持。临时文件仅落本线程runtime，测试输出原capture行为保持。

## 原始观察

证据：`.adg/evidence/OCX-0110-20261014-WIN-BD2-TRACE/`。

| 类型 | 数量与范围 |
| --- | --- |
| 原始JSONL | 836条，保留原字节及SHA256 |
| 进程采样 | 554条，根测试进程与六个核验Node后代 |
| 采样轮次 | 80轮；每次读取后等待50ms，最长读取1459.0111ms |
| 等待链WCT | 78次成功，2次1444；只存第一个节点，不保存外部进程身份/对象名 |
| 夹具文件通知 | 274条，自有TEMP下创建/更改/重命名/删除通知；无buffer overflow |

每个已采样Node经父系、自有新创建时间、完整EXE路径与保留句柄核验；进程记录含Wait0、退出码、CPU FILETIME和GetProcessIoCounters，线程记录含状态/WaitReason、线程创建时间、CPU、cycle及首次WCT结果。没有按名称结束任何进程，也没有启用SeDebugPrivilege或改防护。原测试自身清理结束后，官方Node和desktop数量均0。

## 不能据此证明的事项

- 这次七查询夹具成功，不能解释或替代旧runner五秒active/标记缺失失败。
- 原libtest捕获隐藏成功场景eprintln，case与PID完整关联未建立；只有六Node被观察，短寿命进程会漏采。
- 观察器在测试启动后约312ms才完成初始化，读取/WCT有额外开销；不作性能验收样本。
- 线程状态和WCT是动态快照，不能替代调度/context-switch ETW；第一节点之外的等待链没有保存。
- 文件通知是通知交付时刻，不是文件读写开始/耗时；进程I/O是累计量，从首个观察点计算，不是全生存期逐文件追踪。
- 不归因为Defender、loader、CPU抢占或产品代码；原失败根因未识别。

下一证据应来自**实际失败时间窗**的自有新PID/creation与适当权限、限定范围的调度/文件I/O录制，或后续真实失败runner的原始跟踪。本机重复跑成功直到求绿没有意义，本轮不重跑或扩大采样。WPR只读status仍not recording，未发start，不能写成“ETW录制被拒绝”。

## 目标模式与平台交接

用户继续授权已接收；本轮单次正式 `thread/goal/set` 仅传active/origin=user，不替换objective或预算，但仍被当前active writer拒绝，立即读回paused且原目标/累计用量不变。Windows诊断优先指令已准备但未应用，不能宣称自动续行已恢复。未绕锁、清除/重建目标或标complete。

原desktop在14:58:12后不再有活进程，旧current-session.json只是最后快照。17:19预检和17:37终检本机desktop均0；本轮未正式移交Mac原生窗口，Mac继续后台。日常EXE及原隔离preferences/data-root/backup-policy/schedule摘要前后相同，auto=false，未启动新的24h自动周期。

`verify-evidence.py`无参数重验冻结清单。该提交可供Mac读取**成功对照证据**，不能回答“已经取得bd2失败原始跟踪”；完整Windows门禁、原CPU/RSS失败及预算、TASKrevision2/in_progress/pending与空验收、FZ50缺口保持，不合并、发布、签名或关闭。
