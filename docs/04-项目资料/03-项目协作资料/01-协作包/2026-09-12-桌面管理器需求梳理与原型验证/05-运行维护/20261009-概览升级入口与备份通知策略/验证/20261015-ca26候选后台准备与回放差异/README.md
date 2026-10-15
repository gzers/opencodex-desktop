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

下一轮前台优先以一个普通隔离实例完成真实偏好备份、异值恢复及保护/固定/轮换和共享树表操作；实际操作时间及候选就绪后再安排。未开始前台，不提前把计划计为通过。

## 保护与未完成

09:09:58本机native0，日常EXE和旧隔离四摘要、TASK原SHA bc7965b4…保持，未修改真实数据/系统配置、未执行安装器或FZ50生命周期。原用户删除不入提交；Mac原生未交接。

完整八门禁、管理员窗口/活动云母、托盘退出/Snap/物理DPI、默认/自定义根及安装升级卸载保留、实际更新进度、备份树表/事件、30分钟退避/真实周期、官方0.1.9同机配对与Mac性能failed继续开放。旧版Known Folder/profile隔离和安装隔离前置仍缺，不启动未经证明的官方019基线。TASK/IMP仍in_progress，无关闭、合并、签名、晋升或发布。
