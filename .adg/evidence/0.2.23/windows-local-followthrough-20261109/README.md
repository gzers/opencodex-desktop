# Windows剩余项本机定向进展

用户已授权本机连续负责双SID、提升、安装器与第18项的定向修复/验证，继续不core-sweep/全量、不碰日常配置、不发布。[范围和先行假设](../../../../docs/03-开发实施/0.2.23-contracts/windows-local-followthrough-20261109.md)。

## 双普通SID服务与清理：本轮通过

产品采用第13项原sealed程序：source fdf5d4db551f71cbf493dc97b428f8b9a9364a7a/build42631183420/artifact13681803354/lifecycle.3。新增本机测试入口不改产品，测试源码5ad443222585f1bcf395d70703e45200206614d4。最新Git已在代理恢复后拉取确认；第13项此前待推的a6f1bbb40及范围文档f60a4b865已补推。

two-sid-01前置失败原件保留：New-LocalUser的Description52字符超过Windows48上限，账户0/应用Start0；没有产品运行。只缩短测试描述、首错立即落盘、无账户时不枚举全机残留，然后在新目录two-sid-02首次执行产品。

真实本机Windows Server2025 Datacenter/build26100、RDP会话2，两个不同临时普通SID同桌面原生窗口并行，实际未提升、真实Profile、sandbox:false。不同SID并非用同用户多会话替代。

| 项目 | 用户A（SID尾1014） | 用户B（SID尾1015） |
|---|---|---|
| 唯一Start回执 | 1次/成功1 | 1次/成功1 |
| 原30秒启动 | 15973ms | 16992ms |
| 原生服务PID/端口 | 52368/57052 | 95764/58541 |
| 原10秒正常Stop及实际客户端还原 | 7245ms/通过 | 7864ms/通过 |
| 服务原句柄清理前退出 | exit0 | exit0 |
| 账户/Profile/原目录 | 全删 | 全删 |

两边先经实际Desktop process_action和原保护记录/HMAC/健康/原生监听证明各自Start，再同时observe；正常Stop B后A仍为52368/57052且认证健康，随后正常Stop A。两个Stop均1次，不是强制清理冒正常停止。原30/10/5秒、归属/权限/就绪/还原/退出断言不变。控制器protocolPassed/boundedTwoSidPassed true，desktopAclRestored/programUnchanged true，residualProcesses空；独立读回账户和CIM Profile均0、原目录及服务PID消失、原端口无监听。

四项原证据逐账户见two-sid-02/user1和user2：action-001.raw.json/action-002.raw.json，start-runtime-step.json/stop-runtime-step.json，runtime-record-native.json，overlap-runtime-step.json；A另有survivor-runtime-step.json。各自user1-result.json原数组trace保留原顺序；original-exits-before-cleanup.json含完整镜像/SID/创建时间及真实exit0。账户清理和环境在controller-finished.json、post-cleanup.json、host-environment.json；verified-summary.json为独立断言摘要。

这补齐本轮双普通SID服务互不干扰及双账户清理证明，不覆盖旧失败、不自动关闭所有第18项分支，不代表控制台提升、同SID/IPC/凭据或安装器整项通过。以下项继续分开核对。

## 尚未完成

- 提升/控制台场景；必要时保留环境限制，不用普通令牌冒充。
- 固定端口冲突、同SID/IPC/凭据等剩余实际边界，按已有证据和diff选测。
- 新同源NSIS/MSI构建及隔离账户下实际安装/保留/更新/卸载。已派仅构建任务42642616981/source9ae6b14f71a006e85e53a3c9f327feccccb19d1e，不执行全量/安装/测试，等待新制品后在本机处理。
- 第13项遗留三个编译/下载缓存删除曾被自动审批blocked by policy拒绝，不绕过，仍按原清单单列。

另一次额外提权只读进度观察也遭自动审批blocked by policy，命令未执行；没有换机制绕过，实际结果仅用既有控制器回执。目录各有evidence-manifest.json，原字节大小/SHA256已核验。
