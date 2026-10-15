# ca26 Windows 候选后台准备与回放差异

2026-10-15，Asia/Shanghai。前台窗口已结束，本轮没有本机桌面、原库、Node 或 ETW 新执行。保留 IMP-20 / TASK-182 原范围、失败和全部预算。

## 回放不能代替原失败当次证据

从旧 run38537166560 的原始 backend-windows 日志和新 run38662786466 原件独立比对：

| 项目 | 原失败 CI | 成功 ETW 回放 |
|---|---|---|
| 测试 EXE 字节 | 原制品 SHA497814ca… | 相同 |
| EXE 位置 | `apps/desktop/tauri/target/debug/deps` | `test/software/ci-bd2-query-etw` |
| Node 位置 | `C:\hostedtoolcache\windows\node\24.21.0\x64\node.exe` | 任务 `test/software/ci-bd2-query-etw/node.exe`，官方字节核验通过 |
| Temp | `C:\Users\runneradmin\AppData\Local\Temp` | collector 覆盖到任务 runtime |
| HOME/USERPROFILE | 原回归继承的环境，原日志未完整导出 | collector 显式覆盖 |
| 执行/输出 | cargo workspace 原回归，`--show-output` | 完整原 library、零 CLI 参数，默认 capture |
| 成功夹具阶段原文 | 原 CI 会输出 | 回放 stdout 不含 control-snapshot/control-marker-at |

旧日志记录4逻辑核、RUST_TEST_THREADS/RUST_TEST_TIME_UNIT 为 null。新回放没有证明相同 CPU/负载/编译后工作负载或完整环境，不能把差异归因为具体失败原因，也不能反过来说某一个路径已导致失败。

新成功回放的31进程均有start/stop、Node/cmd家族范围和零丢失结论保持；但缺少七个场景与ETW PID/creation/阶段的逐项关联，不能把总数齐全扩大成七个失败场景逐项完整覆盖。必须取得**实际失败窗口**及对应场景/进程身份，审计真实路径、参数与环境；不改截止、不预热、不施加人为争用，不以另一次成功对照解除失败。本轮不再 dispatch 同类 ETW 回放。

`audit-replay-context.py` 只读固定日志、现存原件及 ca26 的 collector Git blob，输出来源摘要和差异，不复制实现源码、不执行二进制。原先“未复现”结论不改，只补齐其适用边界。

## 下一前台候选提前准备

代码 `ca26e771b0808b8b1b58c7ffadf5d1bad46189ff` 已有同 SHA push38660621496/PR38660625647 成功和原始 checkout 证据。本轮提前制作该精确 SHA 的候选，避免等到下一前台窗口才开始打包；候选制作不构成发布。

只 dispatch 一次 purpose=candidate，HTTP204，实际 run **38663871103** / attempt1。已区别旧 run38662786466 的 query-etw/package-skipped 模式，未错误复用诊断结果作安装候选。新run实际package进行中、query-etw skipped，09:09:58快照仍in_progress/conclusion=null，未计通过、未下载或在宿主执行安装器。

准备快照固定12份方法/请求/实际run绑定/只读差异与保护状态；`verify-preparation.py` 只验证此历史时点。独立观察器最多观察600秒；观察到时不代表远端终止，更不能重派。后续必须继续同run，终态后核验API digest/ZIP CRC/安装载荷/版本/source和runner原始回执，只有准入后才可等待用户新的前台开始信号。

### 候选终态及本机只读准入

第一观察器608.296s到时仍为构建中，第二观察器继续同run，271.078s后于09:22:32读得completed/success。整个阶段只请求一次，未重派；实际package成功、query-etw skipped。runner原始日志ZIP/CRC及20份阶段日志核验，checkout确认为ca26，包锁回归35组/627passed/0failed/7ignored。

| 制品 | 原始ID / 大小 | SHA256 |
|---|---|---|
| 安装包ZIP | 11940418610 / 7,627,587 bytes | `2e945d4bd8ad35a0efb1c4a1449bc678138f1794d5326d588db448da32abff0f` |
| runner证据ZIP | 11940483370 / 1,662,626 bytes | `cff231eb27d29f9b3f2474e536e890a95ee1300c4b2ef69ecfc7f99830ac566f` |
| MSI | 4,567,040 bytes | `6abf10954e91f3bcbad2a0916aa152181d87edb5aad1f2f92a0e5ec0a4ed7d94` |
| NSIS | 3,271,178 bytes | `5162ef6fa6609908bca54c2aa9d1cb97d1a27cb933c8cc9a733eb453dad6d189` |
| MSI提取产品EXE | 8,777,216 bytes / 0.1.10 | `f43e81a9d96f25a29a937a54d2c779ce2459c3ecb30a73b131c57729f70cb929` |

两份artifact大小/API摘要/ZIP CRC/原始条目安全检查、包内source/版本与回执通过。证据包在首次curl下载发生TLS错误，没有留下其部分文件；原失败和方法保留，先独立确认已下载的安装包字节和缺失文件，再只读补取证据。恢复使用正常证书校验的HttpClient，新重定向请求不带Authorization；没有关闭TLS校验、覆盖原压缩包或重跑CI。完整任务日志的curl读取失败也保留，随后同一终态run经独立HttpClient取回原日志ZIP，79320bytes/SHA787d7d8b…，不是作业重试。

MSI数据库只读模式0，提取内嵌CAB，未调用宿主安装流程。产品EXE摘要与runner精确MSI载荷回执相等，产品版本0.1.10，读取原始字节确认内嵌完整ca26源码身份。NSIS/MSI在临时runner的安装退出码0、首屏通过、卸载退出码0且EXE移除；它们不是本机普通/管理员UI、默认data根/自定义根/升级保留/卸载数据选项、物理DPI或真实OTA验收。

本机候选保存于代码worktree `test/software/candidate-ca26e77-38663871103`，可执行载荷位于其 `msi-readonly/opencodex-desktop.exe`。本机未运行产品EXE或安装器、nativeCount=0，日常和旧隔离四摘要及TASK原字节保持。

最终`candidate-manifest.json`固定94份原件/方法，`verify-candidate.py`独立复核真实checkout、原始runner证据ZIP、627项回归、制品、精确载荷与保护检查；`fullVersionGatePassed=false/releaseAccepted=false`。最初12份准备快照不倒改。下一前台的精确候选已经就绪，仍需用户新的前台开始信号。

下一轮前台优先以一个普通隔离实例完成真实偏好备份、异值恢复及保护/固定/轮换和共享树表操作；实际操作时间及候选就绪后再安排。未开始前台，不提前把计划计为通过。

## 保护与未完成

09:09:58本机native0，日常EXE和旧隔离四摘要、TASK原SHA bc7965b4…保持，未修改真实数据/系统配置、未执行安装器或FZ50生命周期。原用户删除不入提交；Mac原生未交接。

完整八门禁、管理员窗口/活动云母、托盘退出/Snap/物理DPI、默认/自定义根及安装升级卸载保留、实际更新进度、备份树表/事件、30分钟退避/真实周期、官方0.1.9同机配对与Mac性能failed继续开放。旧版Known Folder/profile隔离和安装隔离前置仍缺，不启动未经证明的官方019基线。TASK/IMP仍in_progress，无关闭、合并、签名、晋升或发布。
