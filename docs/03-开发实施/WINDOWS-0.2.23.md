# Windows Git协作任务：0.2.23

2026-11-06，用户已授权整版开发/测试/发布，Windows独立Git协作，由L0统一集成。本文是L4唯一可重读任务入口；范围与验收仍归IMP-32 WIN23-00～04。

**当前已放行 R1 Windows 平台开发与隔离验证。** 前序“只读核对、等待R1”已被本条替代。代码分支已真实推送，不能继续用0c57500f8或本机旧分支作为新实现。

## 当前基线与获取

- 仓库：gzers/opencodex-desktop。
- 代码：远端codex/0.2.23-integration，明确基线9e69f4598512aac0c4e068b9f9f9c08ede906a28。
- 文档：远端docs/governance-main，以包含本条R1指令的提交为准；读取本文件、TEAM、IMP-31/32及0.2.23-contracts/L0-paths-runtime-v1.md。
- Windows代码贡献分支：codex/0.2.23-windows；文档/脱敏证据使用独立docs/0.2.23-windows分支。若已有自己的改动，先保留并合并上述明确代码基线，不重置或覆盖工作树。
- 两种产物必须分开checkout：代码分支没有docs，不据此误判资料缺失；docs分支没有新产品实现，不据此编译。先fetch远端上述两分支，再确认git rev-parse HEAD与祖先关系。
- 此为0.2.23实施基线，尚非最终安装候选；版本清单仍待L0统一更新。是否取得新代码以完整SHA及实际diff为准，不能仅看软件显示版本。

## 共享基础已经具备

U=当前用户HOME/.codex-desktop，D=U/desktop，H=U/opencodex/home；固定指针U/desktop/manager-location.json，v2元数据不重复活动根。启动、切换、IPC、卸载已转新版规则，旧迁移入口明确拒绝。运行程序位于实际安装根I/runtime，与D/H分离；managed-only解析器要求固定Node及激活证明，不扫描旧安装、旧HKCU、PATH或独立ocx。

固定依赖：Windows I/runtime/node/node.exe、I/runtime/node/node_modules/npm/bin/npm-cli.js；入口I/runtime/bin/ocx.cmd，包前缀I/runtime/opencodex。主线正在准备完整离线供应，不要求你读取日常旧程序来冒充集成。平台R1可使用明确构造的隔离夹具；实际完整安装/更新制品项待L0交后继制品SHA补验，不能记为已通过。

本机共享检查：data_root 55、IPC 48、workspace 8通过；安装器38项普通检查及固定I布局真实npm6项通过。首轮全Rust回归1333通过、10失败、10忽略：其中4项正是windows_startup_root旧规则测试，另5项目录/IPC消费者已由本机修正，1项200ms进程夹具在隔离重跑通过；保留首轮失败，不把这些数字当整版验收。

## L4 独占写集与本批任务

逐文件单写，L0在本批不写下列文件：

1. apps/desktop/tauri/src/infrastructure/windows_startup_root.rs：退役注册表/安装旁data/旧绑定链恢复逻辑及旧测试，接当前OS用户Profile/SID；保留lib仍使用的defer_main契约。旧四项默认路径测试改为真正验证新版规则及无旧扫描/无副作用，不通过ignore遮盖。
2. apps/desktop/tauri/src/infrastructure/windows_user_scope.rs（按需新增，由windows_startup_root.rs内部path模块引入）：实际Token SID、Profile/LocalAppData、目录拥有者及当前用户ACL辅助。固定待接接口：windows_startup_root::current_user_profile()->std::io::Result<PathBuf>；protect_current_user_path(path:&Path,mode:u32)->std::io::Result<()>。保护新用户数据及私有文件，拒绝接管其它用户根；不修改日常配置。L0据此接infrastructure/platform.rs，不直接由L4改该共享文件。
3. apps/desktop/tauri/src/modules/ipc/endpoint.rs：本批所有权交L4，但只改Windows路径/账户身份帮助及相关测试，Unix行为保持。当前Windows pipe_name仍从USERNAME派生，需要改成真实SID；显式验证客户端SID/管道访问归属，拒绝不同用户误连，保留沙箱与实际协议。模块注册/公共业务服务不归本批。
4. apps/desktop/tauri/windows/installer-hooks.nsh：按新版默认当前用户安装根/数据根修正受影响安装、更新和卸载，不扫描、迁移或删旧数据，不删其它用户/其它拥有者目录。完整runtime-bundle生产部署由L0统一接；需要具体接口先给最小补丁需求。
5. 按实际需要可改infrastructure/windows_restart.rs及同目录已存在对应ps1、windows_query_job.rs、windows_query_fixture.rs；仅Windows进程归属、入口/当前用户实例及本批隔离检查。无必要则不修改。
6. 新增独立平台检查可放test/smoke/windows023-user-scope.ps1及其同名前缀测试，或上述文件内cfg(windows/test)单测。回交实际逐文件清单。

共享lib.rs、data_root、runtime核心、commands、infrastructure/platform.rs、Cargo及锁、Tauri配置、版本、发行工作流、Vue/TS不归L4。需要Windows依赖feature或共享hook时，回交精确函数/feature和最小补丁，由L0接入；不要自行扩大写集。

## 验证和回交

证明两真实SID可并行、同SID多会话/提权仍单实例；固定Profile/根、ACL/DPAPI、pipe/客户端身份、端口占用与进程归属不误连误停。端口属于整机，冲突须明确报出，不能抢占其它用户服务。已有玻璃效果无需本批重测。

真实SID/安装器环境不足时如实分开报告，不用同用户两个RDP会话冒充两用户。普通UI/共享算法不重复做平台专项。隔离数据、保留失败/成功及清理证据；不运行真实业务同步/推理，不改日常配置或自动清理旧数据，不自行发布。

回交必须推到可取得的代码/文档贡献分支，给完整SHA、依赖基线、合同、环境/命令/结果、未完原生/制品项及清理回执。不能只给Windows本地临时路径。L0核验后统一集成，再提供R4实际运行包/安装候选补验。
