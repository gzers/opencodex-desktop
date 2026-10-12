# Windows 候选制品与显式沙箱隔离修复

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182。本页续接 [Windows 接续核验](../20261012-Windows接续核验/README.md)，不构成真机门禁或版本验收通过。机器证据位于 `.adg/evidence/OCX-0110-20261012-WIN-CANDIDATE/`；测试二进制、ZIP、MSI、CAB、工具链和隔离运行目录保留在代码 worktree 的忽略目录 `test/software/`。

## 认证与精确候选

用户完成本机 Git Credential Manager 认证后，已核实账号 `gzers` 的仓库 push 权限；文档提交 `b783d1ba1496c9dbf46d121fc7e65732445e3ab3` 已推送到 `docs/governance-main`。没有保存或输出 token。

重新核对远端 main 精确为 `01a1d85504781f4cb189a68ed85d1d803a1409ff` 后，触发 Windows candidate installer workflow。运行 `38199793144 / attempt 1` completed / success；未创建 tag、公开 Release 或 stable 投递。候选版本确为 0.1.10，CI 安装、最终 EXE 首屏与卸载均 pass。CI 的 MSI / NSIS 应用真实令牌均 elevated=true / elevationType=1，仅代表该隔离 runner，不能替代本机普通 / 管理员双权限门禁。

下载与核验结果：

| 资产 | SHA-256 | 结论 |
|---|---|---|
| `11725935271` 候选 ZIP | `8e0035effd1b6498627057fa9f66dc5c0d2354e188ca9f3dd3997a36e262dc24` | 与 GitHub artifact digest 一致 |
| `11725530965` 证据 ZIP | `01a2aa4990a015d175a27d3b41a54ba28f2faac36bea646baafa46e7daecfada` | 与 GitHub artifact digest 一致 |
| MSI | `8f42a40b10113acb7053d4425deb006aa6db5ac0c43153b6364cc52e750d2174` | 与 CI 安装器回执一致 |
| NSIS | `8fead254d39ff256aba13dadd88d66d57c796e542865912b319d7128864bfef2` | 本地下载包与 CI 回执一致；本机未运行 |
| MSI 内主 EXE | `3a02f3d96834e6fc08f03c3f9c1a323072b7dc527456f2d0f971a59c578bad34` | 只读提取后与 CI 安装后的 EXE 完全一致 |

MSI 只读打开数据库并读取 embedded cabinet，再用系统 expand 提取文件；没有执行安装器、注册产品或修改卸载项。提取方法首次受 COM Execute 的空返回值影响，停止且保留失败目录；修正为抑制 COM 空返回值后成功。失败尝试不计通过。

官方 0.1.9 基线从 `v0.1.9` Release 下载，发布源码 `de6862d40389d527527b9db8ef52e186a58b625f`，MSI 发布 digest 为 `b33dd0c890511467e72a453daaceb4b60b6f5af64dcae5dba54f69ed46bbad6a`；下载一致，只读提取主 EXE `626d63ad6aa15c6ccd2a18eb683b5c3164e57a265da00ee7886d261d28296608` / 0.1.9。它与日常安装是不同构建，不用日常运行程序冒充官方性能基线。两者本轮均未作为测试实例启动。

## 本机安装隔离限制

只读登记显示，日常 0.1.9 是 HKLM 机器级 MSI，ProductCode `{7069DA97-F262-4585-98CA-AE416E6833A5}`，路径 `D:\Software\Develop\OpenCodeX Desktop\`。候选 MSI 为 ALLUSERS=1，ProductCode `{2481427F-7BFE-44A1-BE73-50B8F160DCF7}`；两者 UpgradeCode 都为 `{30FC0B19-5C02-5849-A234-D548E7AFC0DE}`，候选 Upgrade 表检测该产品族。

因此 `/D=` 或 `INSTALLDIR=` 指向测试目录不足以证明隔离。当前执行机不直接安装候选，不临时隐藏 / 删除日常产品注册。CI 安装证据只覆盖临时 runner；本机实际安装目录 `data`、受保护目录、升级与卸载保留门禁继续待验，需要不影响日常安装的专用安装测试环境。

## 发现的隔离缺口与代码修复

启动前读源码发现：

1. Windows `platform::home_dir()` 调用 `dirs::home_dir()`，其 Windows Known Folder 分支优先于环境变量。设置 `HOME/USERPROFILE` 不能可靠地让显式沙箱发现路径避开真实用户 HOME。Windows 工具发现还读取宿主 NVM 变量和系统工具目录。
2. Windows IPC named pipe 名称只按账户摘要区分，忽略沙箱根。若启用 CLI，隔离实例会与同账户日常实例共享名字；数据根隔离不能保证 CLI 目标隔离。

修复提交 `25d115b46dbd20eae336a84b8bf136ee3629eeee`，随后 `6b9abc35742d12a48d25482bf7f7f84d19981300` 补齐原数据根断言，已推送 `feature/0.1.10-windows-isolation`，尚未并入 main：

- 显式 Windows 沙箱 HOME 固定为 `<sandbox-root>\home`；缺失 / 相对根返回 None，不回退真实用户 HOME。
- 显式沙箱只发现自身 HOME 中的工具，排除宿主 NVM / Known Folder / Program Files 候选。日常模式维持原发现规则。
- 沙箱 CLI 管道使用独立前缀和账户 / canonical root 摘要，GUI 与 ocxd 共用解析；根非法或不存在拒绝连接，不回退日常管道。日常管道名字保持逐字兼容。
- 增加 HOME 的正反例、候选路径隔离、管道身份兼容 / 根隔离 / 拒绝坏根、Windows 真实命名管道 bind / duplicate owner / connect，以及独立 Windows 子进程环境验证。
- 原沙箱环境测试改用纯输入，不在并行测试进程修改全局环境。新增 ignored 条目是由父测试显式执行的子进程入口，不计独立通过或门禁豁免。

本机官方 Rust 1.99.0 rustfmt 通过六个修改文件的格式检查；编译 / Clippy / 回归由远端执行，本时点仍待结果。`25d115b` 的普通 CI 为 `38201539240`、候选构建为 `38201580630`；`6b9abc3` 的最新普通 CI 为 `38201999141`、候选构建为 `38202038747`。首补丁和补充断言的结果按各自 SHA 分开记录，不沿用先前提交为最终验收。未把待跑断言记为通过，待实测失败继续修复并保留原件。

## 保留的门禁

本轮没有候选应用真机运行、前台资源采样或真实周期起点；所有已下载应用 EXE 都未运行。日常 PID2232 在开始和 11:52 的观察中仍存在，11:58:11 的新快照未找到任何 OpenCodeX 应用进程，退出原因未知；代理未执行关闭或结束日常进程的命令。日常 EXE 摘要仍一致，真实用户数据未迁移、恢复或替换。

TASK-182 原始对象仍缺失；不能从旧回执 digest 重新构造其 scope / required checks 或正式 RCP。IMP 保持 in_progress，既有 `review:scheduler-performance-0110=failed`、八项门禁未整体闭合和原预算不变；更新详情 / 真实进度、备份异值恢复 / 轮换 / 固定、树表像素、事件解除 / 通道隔离、实际 30min 错误退避、同机 0.1.9 配对性能、真实 stable24h / beta6h / 面板周期均待验。
