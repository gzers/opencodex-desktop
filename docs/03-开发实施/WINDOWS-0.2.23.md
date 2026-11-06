# Windows Git协作任务：0.2.23

2026-11-06，用户已授权整版开发/测试/发布，Windows独立Git协作，由L0统一集成。本文是L4唯一可重读任务入口；状态与细目仍归IMP-32 WIN23-00～04。**当前可执行WIN23-00，不宣称R1代码/最终制品已就绪。**

## 当前指令

拉取远端docs/governance-main，读取TEAM、IMP-31/32和0.2.23-contracts/L0-paths-runtime-v1.md。继续只读核对平台事实，并回交Windows Git贡献分支docs/0.2.23-windows-g0：

- 当前用户Profile/LocalAppData真实API、普通/提权上下文和两不同SID条件；同用户多会话不作两用户证明。
- SID启动锁、数据锁、IPC及DPAPI/ACL应接新版U/desktop，不读旧HKCU/安装旁data兼容。
- 默认程序I=LocalAppData/Programs/OpenCodeX Desktop，runtime/bin/ocx.cmd及runtime/opencodex在I下；允许明确自定义I，不放用户配置进I。确认NSIS/MSI本版部署、更新保留与卸载可用的真实行为。
- 从现有代码核对所需具体Windows文件/接口修正，向L0提交最小写集和验证矩阵，禁止先改共享data_root/runtime/lib.rs。

代码起点0c57500f8f46ea50655c782c3f04334a39167abb，不把docs分支当实现分支。实际R1放行将在本文件明确写代码/合同SHA和单写文件后生效；用户无须复制长任务，Windows对话每次从Git读取此文件即可。

## 后续R1范围（等待明确基线后执行）

分支codex/0.2.23-windows；独占文件优先windows_startup_root.rs、windows_restart.rs/.ps1、必要windows_query_job.rs/fixture、windows/installer-hooks.nsh及独立测试。共享IPC身份/data_root/runtime由L0单写，按接口需求回交。原生验证覆盖两SID并行/同SID限制、进程/端口不误连误停、凭据归属、入口和受影响安装更新卸载；不重做UI/同步/模版算法或玻璃性能。

回交带代码SHA、依赖代码/文档SHA、实际环境/命令/结果、制品/运行包摘要和清理回执。Git资料/证据不得带Key/真实配置，不能只给Mac或Windows绝对临时路径。未完成/失败保留，不自行发布、不清理日常旧数据、不运行真实同步。
