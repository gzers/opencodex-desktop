# Windows 最新交接在新范围下的处置

日期：2026-11-22。Mac 代码：main b6785b486。Windows 代码：codex/024-windows-native 6532cbb6e；Windows 证据：docs/024-windows-dev 32b0483e3。

## 结论

不整批合并该 Windows 分支。

## 依据

以共同基线 cfb3c8aa7 对比，Windows 分支新增 269 个文件里绝大部分是本次撤回的客户端原件保管与恢复链：

- windows_client_originals024 及 before_resume 记录、内容、协调、历史。
- windows_owner_host024 的 empty host、continuity、recovery control、GUI 探针。
- windows_supervisor024 的缺席证明。
- windows_official_lifecycle024 下的 actions/recovery 与 start_worker。
- 对应的 test/windows024-native 验证宿主与脚本。

这些模块服务的对象是「Desktop 接管并保管真实客户端原件」。0.2.24 已把客户端文件交给 OpenCodex 自己管理，Desktop 只通过 CODEX_HOME 指向自管目录，真实用户目录只读。带入这批模块会重新引入已被撤回的职责，还会重建对已删除模块的依赖。

main 侧仍保留 Windows 正常生命周期本身：LaunchPlan、原创建句柄与 Job、正常零退出、监听与记录清理、程序身份不变，以及后继启停。本轮已修好该文件对已删除 recovery_required 的残留引用。

## 交接要求

Windows 侧需要以 main b6785b486 重建分支，只保留：

- 程序根、当前用户入口、PATH 事务与安装器资源映射。
- 真实 Windows 正常 Start / Status / Stop / Restart，保持原 30/5/10 秒预算、未知拒绝与失败保留。
- MSI / NSIS 与卸载清理中原有的平台适配。

不再保留任何客户端原件保管、恢复、owner-host 或 supervisor 缺席证明代码；这些不再属于本版范围。

## 验证边界

本机没有 Windows C 工具链，无法交叉编译或运行 Windows 目标。上述结论来自逐文件净差审查，不冒充 Windows 构建或真机结果。Windows 真机启停与取消后清理仍待其重建分支后补验。

未修改日常配置，未执行真实业务同步或模型请求，未发布。
