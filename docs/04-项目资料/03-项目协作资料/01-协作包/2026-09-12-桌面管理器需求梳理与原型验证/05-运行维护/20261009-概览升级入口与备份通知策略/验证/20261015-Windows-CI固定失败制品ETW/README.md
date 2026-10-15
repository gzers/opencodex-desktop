# Windows CI 固定失败制品的单次 ETW 诊断

2026-10-15，Asia/Shanghai。绑定 IMP-20 / TASK-182，执行依据为原完整失败取证目标及用户“批准修正版后续都可以管理员后台 ETW 录制”的持续授权。本机前台窗口结束后，本段只在后台推进；没有本机桌面、Node、原库或 ETW 新执行。

## 实现与准入

代码 `feature/0.1.10-windows-query-etw@ca26e771b0808b8b1b58c7ffadf5d1bad46189ff` 已推送。相对 e6 仅四份变更：CI 增加离线入口测试，已有 Windows candidate 工作流增加独立手动诊断模式，新增准入脚本和离线测试；`apps/**`、原测试夹具、原 collector 三份实现均未变。原 candidate job 除模式条件外正文一致。

`query-etw` 模式仅在临时 Windows Actions runner 执行，固定原 bd2 失败 run `38537166560`、artifact `11881042603`、实际 PR checkout `31e7748d9e82c79bef97fc1b73233f08bc1f1b3d` 的原测试 EXE；历史实际 checkout/tree 归属沿用已存原始准入，运行时再核对失败 run、artifact API 摘要、包内 EXE 回执和实际 EXE 摘要。测试 EXE SHA256 `497814ca08c071dcd4a2666ceb954bd4788e7cc6cf1dc8972c3efd515884c0d3`。

Node 固定官方 `24.21.0`，ZIP 官方 SHASUMS 与 EXE SHA256 `ba4e6d110e8c1592a1ecd390f6b05f3da124b13871a5be62b341a07a853c6c32`，执行前 Authenticode 为 Valid/OpenJS。TraceEvent `3.1.21` 固定 nupkg 及两 DLL 摘要；没有全局安装或生命周期脚本。跨主机重定向不转发 Authorization，ZIP 拒绝逃逸、链接、大小写冲突和超预算解压。首次 Windows 离线测试发现 ZipInfo 提前规范化反斜杠，修正为校验 `orig_filename` 后 10 项通过，红绿原件均保留。

三个实际固定压缩包 CRC、路径与载荷摘要检查通过，10 个 PowerShell 工作流脚本块解析通过；宿主执行入口被 CI-only 准入拒绝。未安装 YAML 解析器，也不宣称做了本机 YAML parser 校验；最终两轮远端工作流实际成功另证。

## 同 SHA CI 终态

push `38660621496` 与 PR `38660625647` 均 completed/success，五必需 job 成功、main-only build skipped。每轮 Mac 35 组 / 900 passed / 6 ignored，Windows 35 组 / 627 passed / 7 ignored，0 failed。完整日志 ZIP 摘要/CRC及后台 checkout/test 原日志封存；push 实际 checkout 为 ca26，PR 合成 checkout 独立记录，不以 head 或 tree 相同推定二进制相同。

第一次只读观察器 905.391s 到时仍记录远端 in_progress，没有重派作业；第二观察器继续原两个 handle，169.813s 后取得真实终态。间歇 TLS/API 失败与运行中 job-log 404 均保留，读取失败不解释为作业已停。PR5 描述首次 PATCH 未生效，后续两次实际读回确认旧内容后才执行一次恢复更新，最终精确读回，PR 仍 open/draft/unmerged、无 auto_merge。没有合并或创建新 PR。

同 SHA push 成功及原始 checkout 日志核验后，只 dispatch 一次诊断请求，HTTP204，实际 run `38662786466`、attempt1。工作流名字仍为 Windows candidate installer，但实际 `query-etw` job 成功、`package` skipped，未打包安装器。

## 原库及 ETW 真实结果

08:56:06–08:56:22（UTC+8）在临时 runner 执行一次原失败制品的完整原库，零 CLI 过滤参数、默认并发、原输出捕获，不预热。原 2s query / 4s cleanup / 5s observer / 8s watchdog 与 collector 120s 外层保护、64MiB 输出/32768 queue/32768 IRP 保持。

- 原库 exit0，**530 passed / 0 failed / 4 ignored**，七个 query 全部通过；当次源码文档路径存在，`license.opened` 不再报本机缺失路径。这不是原七 query 失败的修复或解释。
- 根 PID9668、creation `2026-10-15T00:56:06.1703141Z`，持有句柄映像匹配；真实 token elevated=true/type1（runner 未拆分令牌），不与本机产品 type2 证据混用。
- 原始 JSONL **17,085 条 / 5,811,506 bytes**，UTF-8/LF 实际字节=预算计量，无 BOM/CRLF。事件 SHA256 `a4f4260a3a58d812a1e17a436364c2616e1a9e8fb8c9c40086cf116c4f76b1d8`。
- 31 process start/stop：test1、Node22、command-basename8；940 thread start/stop，映像1864，ready2155、scheduled-in3419/out3175，file-create2025、operation-end2264、read/write239。保存的进程生命周期都有 stop，已知映像覆盖缺口0。
- serialization/kernel/decoder loss/error 全0，consumer/writer 排空、自有 session stop 成功，native stop status0/final loss0，原测试根已退出，没有外层强制清理。
- 根高频 scheduling158572、file-I/O initiation287481 按原 scope 省略并计数，不能宣称根热点全覆盖；没有写整机 ETL 或无关事件载荷。

**目标失败未复现，实际失败时间窗 ETW 仍未取得。** 本次是固定原失败 EXE 在 CI 环境的成功对照，不能清除 bd2/e6 历史 CI 失败、认定根因或改预算。本轮不自动重复运行求绿。

## 原件与后续边界

artifact `11939861138`，340,221 bytes，SHA256 `6f53dd395106cceb285f5e6339f0d017f3cba05f180ced7f5ea6708db2ccff05` 与 API digest 匹配；13 份包内原件逐项解包校验。job 原始日志 ZIP/checkout及九份阶段日志另封存。证据目录 `.adg/evidence/OCX-0110-20261015-WIN-CI-ETW/`。

`preparation-manifest.json` 固定 36 份准入前快照，不因后续执行改写；最终整体清单和验证脚本独立覆盖双 CI/实际 checkout、诊断 run、制品字节、原测试结果和 ETW 数量/收尾。两类快照不冒充同一时点。

本机 native 实例归零，日常 EXE/旧隔离四摘要、TASK原3878bytes/SHA `bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93` 保持；用户原删除未纳入提交。前台不再操作，Mac原生尚未正式交接。

官方0.1.9制品身份通过不代表 profile 隔离通过：旧版 `dirs::home_dir` 及 Known Folder 实验显示 HOME/USERPROFILE 覆盖不能代替真正测试用户配置环境；本机尚不能安全启动配对基线。也没有 WindowsSandbox.exe，安装/升级卸载需独立隔离方案。管理员完整交互/活动云母、托盘退出/Snap/物理DPI、安装根/保留、真实更新与备份树表/事件、最终源码退避/真实周期、原预算配对性能与 Mac CPU/RSS failed 均继续开放。

TASK/IMP保持 in_progress，不降低门禁、不关闭证书、自动合并、签名、晋升、替换日常安装或发布。
