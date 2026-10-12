# Windows 接续核验：01a1d855

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182。本轮仅完成源码、CI、制品可用性和本机测试工具核验；Windows 候选应用验收尚未开始。证据位于 `.adg/evidence/OCX-0110-20261012-WIN-PREFLIGHT/`，文件摘要见该目录 `manifest.json`。证据目录禁用 Git 换行转换以保留原始字节；两份人工 Markdown 的摘要明确使用 canonical-lf，校验方法见 `verify-evidence.py`。

## 来源与环境

- 已执行 `git fetch origin --prune`；源码 `main` 与 `origin/main` 均为 `01a1d85504781f4cb189a68ed85d1d803a1409ff`，开始时文档 `docs/governance-main` 与远端均为 `a83250baa7b5c6ed7c20fbf60477b9fa1f1f9fa9`。
- 隔离代码 worktree 为 `C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop`，精确检出上述源码 SHA；没有产品代码修改。
- 本机实际系统为 Windows Server 2025 Datacenter，build 26100，x64，Asia/Shanghai；不能标为 Windows 11 环境。测试工具所在 PowerShell 7.6.5 的实际进程令牌为普通权限。
- 日常安装为 `D:\Software\Develop\OpenCodeX Desktop\opencodex-desktop.exe`，产品版本 0.1.9，开始时 PID 2232，EXE SHA-256 为 `43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67`。本轮未启动候选应用、安装或卸载软件，未读取或修改真实配置载荷。
- 文档工作区开始时已有一份 `settings-start-envelope-invalid.json` 删除状态；原样保留，未纳入本轮提交。

## 同 SHA 的 CI 与制品

GitHub Actions API 快照记录 `CI / 38196769142 / attempt 1 / head_sha=01a1d855…`。Windows 令牌工具、release-tools、frontend、macOS backend、Windows backend 五项 job 均 completed / success；Windows backend 的回归、release build、最终 EXE smoke 及证据上传步骤均 success。11:07:43（UTC+8）的核验中，macOS bundle build 仍 in_progress，初始快照保留。11:11:50 刷新后整条 CI completed / success，六项 job 均 success，见 `ci-final-summary.json`、`ci-run-final.json` 和 `ci-jobs-final.json`。

同 SHA 当时只有这一条 CI，未发现 Windows candidate installer workflow run。制品为 `windows-startup-evidence` 和 `windows-process-token-evidence`，ID、大小、过期状态、GitHub digest 与来源 SHA 见 `ci-artifacts.json`。本轮仅核验 API 元数据，未下载，因此没有本地 ZIP 摘要核验结论；两份证据制品不能替代 0.1.10 候选安装器，也未使用其他 SHA 的安装器代替。

## 本机工具回归

在上述精确源码 worktree 执行：

| 工具 | 结果 | 实际覆盖 |
|---|---|---|
| `windows-process-token.test.ps1` | 9 passed / 0 failed | 保留进程句柄、真实子进程身份与普通令牌、错误 EXE / 相反权限 / 缺失或字符串令牌 / 已退出或释放进程拒绝、全部 smoke PowerShell 脚本语法 |
| `windows-cdp.test.mjs` 与 `windows-bundle-identity.test.mjs` | 12 passed / 0 failed | CDP 协议与网络错误边界、安装后 EXE 精确身份校验工具 |

令牌测试对象是独立 `cmd.exe` 子进程，`elevated=false`、`elevationType=3`；不能计作候选应用普通权限门禁，更不能计作管理员门禁。原始 JSON、测试命令及日志随证据保留。没有新增产品实例。

## 接续缺口与原门禁

1. 原始 `.adg/work/objects/task/TASK-OPENCODEX-DESKTOP-182.json` 不在本机；该目录被共享 `.gitignore` 排除，仓库全部现有 refs 中也未找到此 TASK 的提交历史。已有 RCP 保留 task digest，但不能据此重造 TASK 的 write scope、required checks、验收或状态。需从原执行机同步原始对象，验证其 content / summary / baseline digest 后继续正式 RCP。
2. 本机未取得可用 GitHub 写入凭据。禁止交互的 `git push --dry-run` 实际 exit 128，输出为无法提示凭据；未请求或记录密码 / token。因此本轮不能触发同 SHA 的 Windows 安装器 workflow，也不能完成远端推送。需由用户在本机完成 GitHub 认证，或提供已经生成并可核验的同 SHA 候选制品；推送仍需写入权限。
3. 未在 PATH 找到 cargo / rustc，亦未检测到默认位置的 Visual Studio Installer `vswhere.exe`；本轮没有进行本地 Rust / 安装器构建。该观测不等于断言系统任何位置都无工具链。

普通 / 管理员候选真实令牌、原生 DPI / 云母 / 标题栏、实际安装目录 `data` / 自定义根 / 卸载保留、更新详情与真实进度、异值备份恢复 / 轮换 / 固定、共享树表对齐、事件解除 / 通道隔离、状态读取错误真实 30min 退避均待验。安装器运行前还须确认注册项和卸载范围不会覆盖日常 0.1.9；隔离目录参数不能单独证明安装器隔离。

性能合同继续使用原预算：同机 0.1.9 配对，冷 / 热启动各至少 20 次，交互 p95 增量 ≤100ms 且 ≤5%，闲置 10min CPU 增量 ≤0.5 百分点、RSS 增量 ≤10MiB，检查无 >50ms 主线程任务；缓存命中 100 次切页 / 10 次重启为零自动网络与查询子进程。真实 stable 24h、beta 6h、面板周期、睡眠恢复和跨启动观察本轮均未开始，没有周期起点或到期通过结论。

此前 `review:scheduler-performance-0110=failed` 和八项 required checks 未整体闭合的结论保持。IMP 保持 in_progress；原 TASK 在本机缺失，本轮未修改或宣称重新验证其状态。未创建正式 RCP、完成证书、稳定核心事实晋升、tag 或 Release。
