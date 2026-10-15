# Windows 原构建后单次 ETW 诊断

2026-10-15，Asia/Shanghai。按新目标先后台诊断、阶段集中归档。源码`feature/0.1.10-windows-query-etw@8e266abac78ff6d1b78cb05e680c7c5dbe1f0e56`，父493。只修改6份test/工作流；`apps/desktop`整树仍`bf4365586d874d1a9268636e5227efad4621bad1`，与原bd2相同。没有新桌面候选，旧493 EXE不改贴新SHA。

## 改变的诊断条件

原CI先正常构建，再首次执行库，使用系统TEMP；此前固定失败EXE回放则使用工作区TEMP及重定向HOME/USERPROFILE。本轮新增显式`post-build-runner-temp`模式：仅临时Actions Windows runner、workflow_dispatch/attempt1准入，验证原应用tree后运行常规npm build/fmt/clippy/cargo test --no-run，不运行测试或额外预热固定Node，再单次执行原失败EXE。

保持临时runner用户配置，仅在原系统TEMP盘新建独占自有子目录；序列化路径白名单仍限定该子目录、固定Node和System32，不包含整个系统Temp。保留原默认并发/capture、2s query/4s cleanup/5s observer/8s watchdog、原库120s外层保护、64MiB/32768队列/32768 IRP。作业45分钟仅供构建准备，不是放宽测试截止。无人工CPU/I/O压力、循环重跑或宿主环境修改。

采集器新增纯环境合同及CI-only门禁，旧隔离模式默认行为保持。调度记录增加等待/状态原解码数值，显式`versionSpecificWaitSemanticsVerified=false`，不把未知数值重映射或据此推断根因。

## 验证及实际执行

- 本机11项Python准入、101项采集器纯合同、10份Windows脚本/11个工作流块语法通过；未本机启动原库、固定Node或ETW。没有本机YAML parser，远端工作流实际成功单独核验。
- 同SHA push38699864356、PR38699869579全部成功；每轮Mac900/Windows627passed、忽略6/7，各35组。原日志ZIP/CRC/选择日志逐字节核验；push实际8e266aba，PR实际02de6ed6、整树73c4af0f相同，不推断二进制相同。
- 原失败run38537166560仍failure；artifact11881042603未过期且API摘要匹配。本机固定失败EXE497814ca…、官方Node24.21.0/ba4e6d11…及两DLL摘要/Node OpenJS有效签名重核。
- 15:59:09单次dispatch HTTP204，实际run38702455024/attempt1，query-etw成功/package跳过。常规构建准备15:59:51–16:04:45；原库及录制16:04:51.364–16:05:53.730。

原完整库exit0，530passed/0failed/4ignored，七query全部通过。测试根PID6208/creation`2026-10-15T08:04:51.4337703Z`，实际elevated/type1为runner未拆分令牌，不混同宿主管理员type2。runner为4逻辑CPU、win25-vs2026/image20260925.250.1、rust1.99.0，与历史记录逐项保留；`exactHistoricalEnvironmentReproduced=false`，不声称重建全部宿主时序。

16,865条原始ETW/6,089,687bytes，SHA `b9a044718f8ab98d707ee3a2c056204f1c6a5a16091ca757ea3e7847bc95b1fe`。31个process-start/stop全部对应：test1、Node22、cmd-basename8；957对线程开始/结束、ready2090、scheduled-in3126/out2865、file-create2193、operation-end2466、read/write273。丢弃、kernel loss、decoder errors与已知映像覆盖缺口均0；consumer/writer排空、自有session原生STOP0，未外层强制清理。根高频调度237673、文件I/O initiation303353仍按原scope省略计数，不宣称根热点完整覆盖。

artifact11957286829/351969bytes/SHA `055eeed1a8cdc7e64c128de4f1353ff3f465fb6c4c409139cc71157251052b7d`匹配API，14份包内原件逐项校验。完整证据`.adg/evidence/OCX-0110-20261015-WIN-POSTBUILD-ETW/`134项清单及`verify-evidence.py`通过，包括嵌套清单、实际checkout、原日志和原始事件。

## 结论与下一步

**目标失败未复现，实际失败窗口ETW和根因仍未取得。** 构建后/系统TEMP条件下的成功不能排除加载、调度、文件I/O或其他环境因素，也不撤销bd2/e6失败。此条件仅执行一次，不自动追加相同录制。后续应针对普通CI首次回归的自然失败建立留证路径，而非继续重复独立成功回放；具体实现须保留原断言、预算和源码/制品身份，不先承诺下一轮必能复现。

16:09保护检查本机native0；日常EXE、旧隔离四文件及原TASK3878bytes/SHA bc7965b4…保持，代码8e266aba干净。独立验收账户和WindowsSandbox.exe仍不存在，未创建账户或正式Mac原生交接；前台未操作。PR5终态说明已更新并独立读回，仍open/draft/unmerged、无auto_merge。

TLS/API读取失败及checkout清理日志匹配误拒绝原件/边界保留，修正离线提取后复用原ZIP，不重派CI、放宽证书或转发跨host凭据。文档只在阶段结束集中回写。Mac性能failed、原性能/真实周期预算、安装/Profile环境及剩余原生/更新/备份/事件八门禁全部保持；不合并、关闭、签名、晋升或发布。
