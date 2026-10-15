# Windows 首次 CI 回归同步留证

2026-10-15，Asia/Shanghai。上一轮固定失败EXE构建后回放仍成功，未自动重录。本轮把留证接入普通CI首次库执行，源码`feature/0.1.10-windows-query-etw@43142bdbcc7a0b9f1325103febae3f44dce407ab`、父8e266aba；只改7份test/ci.yml，apps/desktop树仍bf4365586d，与原bd2相同。原TASK、产品和查询夹具源码不改。

## 实际路径与身份边界

仅本仓库诊断分支push/PR的attempt1启用；新准入验证历史失败run/artifact、官方Node24.21.0及固定解码器，再通过临时Cargo runner配置接入正常`cargo test --locked --workspace --features integration-test -- --show-output`。库首次启动即同步录制，每个作业仅一次，原退出码传回Cargo；其他workspace套件按原参数正常执行，不因失败自动重试。配置位于当次输出目录，含空格的程序路径用参数数组，不改全局Cargo配置。

本模式当前CI本身是验证，`collectorCiRun=null/currentCiIsVerificationNotPreviouslyAdmitted=true`，不伪造已通过的当前CI准入。执行的是**当次正常编译库**，历史失败EXE497814ca…仅作独立固定参照，二者摘要分别保存；应用tree相同不证明二进制相同，也不把新制品记成原失败EXE。标准libtest capture与唯一`--show-output`保持，无过滤/串行化/Node预热。

保留2s query、4s cleanup、5s observer、8s watchdog、120s原库外层以及64MiB/32768队列/32768 IRP。临时runner用户配置保留，TEMP仍使用系统临时盘的独占任务子目录以限制记录路径；启动器、额外校验和ETW有观测开销，不宣称历史环境完全重现或性能验收通过。仅持久化自有代际范围，未知等待数值/版本边界继续保留。

## 本轮结果

本机12项Python准入、108项采集器纯检查、工作流脚本语法、runner宿主拒绝及含空格路径TOML数组回读通过；未本机启动Cargo、原库、固定Node或ETW。

push38705201081与PR38705207794完整success，五必需job成功/main-only build skipped。各Mac900/Windows627passed、忽略6/7，各35组；普通回归含新增留证不是额外重跑。push实际checkout43142bdb，PR实际485372d7，整tree均ccb9f9ab；原始日志ZIP、实际checkout及选择日志逐字节核验。

| 原CI首轮 | 库结果 | 实际EXE SHA256 | 自有ETW |
| --- | --- | --- | --- |
| push | 530通过/0失败/4忽略，exit0 | 874cd89cada161299750c3beb9e24457b2b26b377c22a0ae5641ec9d218e4e8b | 18732条/6787959bytes |
| PR | 530通过/0失败/4忽略，exit0 | 4e86d7b28bcebb1fd7f98e4205aea2b74164d940c4c3e99bfef63566b0125591 | 16128条/5825028bytes |

两轮七query均通过，每轮31个process-start/stop及948对线程生命周期，丢弃/kernel loss/decoder errors/已知映像覆盖缺口0，consumer/writer排空、自有session原生STOP0，未外层强制清理。push根PID2016/creation08:31:13.4347165Z，PR根8196/creation08:30:53.8924927Z；均runner实际提升type1，不替代宿主产品双权限令牌。

artifact11958274917/9881449bytes/SHA1a290e13…和11958514404/9851782bytes/SHAa24b07f6…与API匹配，各14份相关原件核验。原始制品ZIP保持，ETW另以gzip保留并复核解压原字节、LF和摘要，避免在Git中重复存放大份展开JSONL。证据`.adg/evidence/OCX-0110-20261015-WIN-FIRST-CI-ETW/`118项独立清单/verify-evidence通过。

## 结论与边界

已验证普通CI第一次库执行的同步留证路径，无需等失败后重新启动进程。**本次没有当前库失败，也没有原历史失败制品的失败窗口ETW，根因仍未知。** 两轮成功不能撤销bd2/e6旧失败，不自动加跑或通过工具开销后的数字认定性能。根高频调度/I/O仍按原范围省略并计数，非全根热点跟踪。

16:45保护检查本机native0，日常EXE、旧隔离四摘要、原TASK3878bytes/SHA bc7965b4…保持，代码43142bdb干净；Profile/安装环境依赖仍缺，未建账户或正式Mac原生交接。PR5说明更新并独立读回仍open/draft/unmerged/无auto_merge。没有新桌面候选，旧493 UI证据仍绑定旧SHA。Mac性能failed、完整原门禁/预算/真实周期保持，未关闭、合并、签名、晋升或发布。
