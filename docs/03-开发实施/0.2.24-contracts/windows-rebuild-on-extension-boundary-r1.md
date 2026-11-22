# Windows 重建输入：自管客户端根与扩展落点

日期：2026-11-22。Mac 代码：main 81d823971。Windows 代码：codex/024-windows-native 6532cbb6e（仍为撤回范围）。

## 为什么再发一次

[上一份处置](windows-handoff-scope-disposition-r1.md)要求 Windows 以 main b6785b486 重建，只留程序根、当前用户入口、PATH、安装器资源与真实生命周期。此后 main 又前进两笔，都落在「客户端配置归谁」这条线上，Windows 重建必须按新 HEAD 做，否则会把旧落点带回去。

## 新的重建基线

代码 main 81d823971。以下两条是本轮新增、Windows 必须遵守的：

- 扩展写入的 Codex 落点是 <OPENCODEX_HOME>/codex-client，不是真实 ~/.codex。目标矩阵统一走 modules::extensions::client_targets(data_root, home)；不要再各自拼 home.join(".codex")。
- managed_codex_root() 只从已准入的数据根记录推导。数据根直接挂在共享系统临时父目录下时返回 None，按未绑定处理——Windows 上同样成立（%TEMP% 是多用户共享的）。

细节与验证见[扩展落点边界合同](extension-client-home-boundary-r1.md)。

## 重建后的写集

只保留：

- 程序根、当前用户入口、PATH 事务与安装器资源映射。
- 真实 Windows 正常 Start / Status / Stop / Restart，保持原 30/5/10 秒预算、未知拒绝与失败保留。
- MSI / NSIS 与卸载清理中原有的平台适配。
- 扩展相关的 Windows 侧接线，改为消费 client_targets() 与 managed_codex_root()，不得自行推导客户端落点。

不再保留客户端原件保管、恢复、owner-host 或 supervisor 缺席证明代码。

## 需要回交的验证

1. 单用户真实 Start → Status → Stop，记录耗时与终态。
2. 写入一条 MCP 到 Codex 目标，确认落在 <OPENCODEX_HOME>/codex-client/config.toml；确认真实 %USERPROFILE%\.codex 未被创建或修改。
3. 两个不同账户各起一个实例，确认各自的自管根互不覆盖（这同时验证共享 %TEMP% 的排除逻辑）。
4. 清理后确认测试账户与 Profile 删净；删不净则列残留进程 pid、镜像与 SID。

不跑全量 sweep，不发布。原失败证据保留原目录，成功不覆盖失败。
