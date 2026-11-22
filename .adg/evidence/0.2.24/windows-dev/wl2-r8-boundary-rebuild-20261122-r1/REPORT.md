# Windows 0.2.24 自管根边界重建回执

源码基线 main b684e17887c46cda505810e6a84e7874596cf50c；源码分支 codex/024-windows-boundary-r2（0b78ea69d21ffd3238481a2110c7f28b7583f482）；治理交接 docs/024-windows-boundary 7e32150a4c4b886dade9ed111cd63249a6963f67。只保留程序根、当前用户入口与 PATH、安装器资源、真实 Start/Status/Stop/Restart；扩展统一走 modules::extensions::client_targets 与 managed_codex_root。

## 正向结果

- 单用户 out14：普通 SID S-1-5-21-587408064-856152790-2583328096-1001，session 2，Medium、未提升。Start raw0，17,998 ms，result started，stage ready-current；Status 三次 raw0，859／855／859 ms，均 running；Stop raw0，4,292 ms，result stopped，stage native-normal-exit。
- MCP 写入 codex，目标 C:\Users\15119\.codex-desktop\opencodex\home\codex-client\config.toml；真实 C:\Users\15119\.codex\config.toml mtime 1795184158 秒（2026-11-20T14:15:58.2969344Z），SHA256 830eb5183df83b3465cd01e487194898f7225671d36284dc29240e08357751b5，前后一致。
- 双账户 out-two：1001 / C:\Users\15119 与 1003 / C:\Users\marenben 的自管聚合根、数据根、Codex 根均不同，distinct=true。后续在 marenben 自己的 session 7、普通未提升令牌实跑同一条生产启停：Start 20,149 ms running，Status 828／900／869 ms 三次 running，Stop 4,429 ms stopped；MCP 只写 C:\Users\marenben\.codex-desktop\opencodex\home\codex-client\config.toml，真实 C:\Users\marenben\.codex\config.toml 的 mtime 1794892149 秒、SHA-256 d19c0773b53b6f016fbf35f317cae551c540b70e67fd5309a8a4c1d847bc7700 前后一致；停止后布局删除、aggregatePresent=false。
- 清理：out14 的 %USERPROFILE%\.codex-desktop 已由测试自身在正常停止后删除，aggregatePresent=false，leftover 为空。测试未创建账户，只读取既有账户与 Profile。

## 修复

1. windows_official_lifecycle024.rs：保持原 30/10 秒预算。Stop 成功后，最终来源复查改为只核当前安装根/程序选择（source_identity），不再用已退出原进程的活身份查询；活期 source 仍保留原生绑定复查。
2. Stop 验收以官方 root 退出 0 为服务终态，同时要求全部 Job 成员已退出、无 active 成员、原来的 loopback 监听器全部关闭、runtime-port.json 与 ocx.pid 缺席；不再把 Codex 会话/console 子进程的退出码当作服务结果。共享未知拒绝未放宽。
3. 新增 r2_boundary_single_user 与 r2_boundary_two_accounts 两个目标性 Windows harness，使用既有固定官方包 14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7 与 descriptor 42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a。

## 失败与边界

out01 首轮 Start 20,667 ms 成功，Stop 在 native-normal-exit 返回 windows024_owner_unknown，原始现场保留在 out01/field。out02–out10 为同一修复过程中的可复现失败：先暴露旧 Job 总量与保留成员不一致，再暴露两个 codex.exe 子进程退出 1；最终以 root 退出码为准收口。所有目录均以新 outNN 保留，未覆盖旧错误证据。

入口 launch-r8-marenben.ps1 在测试通过后的结果回执处使用了 PowerShell 7 的 utf8NoBOM 编码名，Windows PowerShell 5.1 因此未生成 result.json；产品测试、r2-events 与 stdout 均已完整保留，首错属于测试工具而非产品。本轮没有用管理员令牌冒充普通用户，也没有创建新账户；marenben 仍按用户已确认的可丢弃范围待清理。

构建：MSVC x86_64-pc-windows-msvc，cargo check raw0；测试目标链接 raw0；生命周期定向 3/3 通过、3 ignored，见 final-tests02。最终单用户 EXE SHA256 f08edf9a872463d98541885e86db9f922cfc5ad1b1d3a45bd0d99e1fb9585e6b，90,175,488 字节，对应 Git 树 8e9376181145c3455645376ae02d266d42387ba8。测试 stdout/stderr、逐事件 JSONL 与 build.json 均在本目录。
