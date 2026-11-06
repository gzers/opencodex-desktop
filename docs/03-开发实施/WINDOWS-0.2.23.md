# Windows Git协作任务：0.2.23

2026-11-07，用户已授权整版开发/测试/发布，Windows独立Git协作，由L0统一集成。本文是L4唯一可重读任务入口；范围与验收仍归IMP-32 WIN23-00～04。

**当前已放行 R2 Windows 集成影响与原生边界补验。** R1贡献已接收并集成，前序R1独占编码派发仅作为来源保留；本轮操作与写集以本节R2为准。代码已真实推送，不以旧R1基线或软件显示版本判断是否取得新实现。

## 当前基线与获取

- 仓库：gzers/opencodex-desktop。
- 代码：远端codex/0.2.23-integration，明确基线2bbe1aa92dd824a6c14e7f02caafb7a1e4f27c00。
- 文档：远端docs/governance-main，以包含本条R2指令的提交为准；读取本文件、TEAM、IMP-31/32及0.2.23-contracts/L0-paths-runtime-v1.md。
- Windows代码贡献分支：codex/0.2.23-windows；文档/脱敏证据使用独立docs/0.2.23-windows分支。若已有自己的改动，先保留并合并上述明确代码基线，不重置或覆盖工作树。
- 两种产物必须分开checkout：代码分支没有docs，不据此误判资料缺失；docs分支没有新产品实现，不据此编译。先fetch远端上述两分支，再确认git rev-parse HEAD与祖先关系。
- 此为0.2.23实施基线，尚非最终安装候选；版本清单仍待L0统一更新。是否取得新代码以完整SHA及实际diff为准，不能仅看软件显示版本。

## R2 当前派发（2026-11-07）

已接收并集成Windows代码codex/0.2.23-windows@42d2a4a1770a5be1050dec3de641efab77a65917，证据docs/0.2.23-windows@f7c54ad2b7027b209ff99a4374a4d4f6cf9179d0；代码原提交911310f1、0c2d1a1d、42d2a4a1对应集成c8e7ac62e、0f3d670d5、7d1ace177。R1证据按原字节归档于[回交目录](../../.adg/evidence/0.2.23/team/L4/r1-20261106-01a1b0c8/)，集成文档提交1a45c44c92c00af9adac00ac66c0bdf821c6eda6。R1的97项GNU检查通过只证明其实际覆盖范围；同一SID的RDP会话不作为两用户证明，GNU复制EXE不作为正式安装包。

本次集成基线已包含04511769c的平台hook（实际Profile/ACL及连接前服务SID验证）与f0f76b7a4的Windows条件编译修正。先在独立代码树fetch并合入明确基线，确认该SHA是当前HEAD祖先；保留贡献分支已有改动，禁止reset覆盖。

1. **现在可执行**：使用当前集成源码，在Windows原生MSVC工具链编译/运行受影响检查，补验两真实SID并行、同SID跨会话/提权单实例、跨SID凭据拒绝、pipe及CLI身份验证、整机端口冲突与进程归属不误连误停。报告真实账号/SID关系，证据脱敏；缺少第二用户权限时明确未验，不以模拟通过代替。
2. **现在可执行**：检查本基线中已集成的R1实现与L0 hook是否正确组合。必要修复仅限R1已列Windows专属文件与test/smoke/windows023-*；platform.rs、lib.rs、runtime核心、Cargo/版本/发行工作流仍归L0。需要公共修正时提供精确接口/最小补丁需求，不并发改共享文件。
3. **待后继制品**：MSVC正式NSIS/MSI的安装、从0.2.22更新、卸载保留数据以及完整离线runtime供应，以L0后继交付的安装包/资源描述符摘要为准。本次源码交接不是最终安装候选；不能将隔离夹具或本机缺失的Mac临时路径记作制品交付。仓库已提供.github/runtime-platforms-023.json、.github/scripts/build_desktop_runtime_bundle.py与build_runtime_023.py，可读配方核对，stage/发行接线已在205d72b1a实现，可使用.github/scripts/stage_runtime_bundle_023.py按固定配方生成完整资源；最终版本/签名正式制品仍由L0统一收口。
4. **本批新增条件补验**：公共玻璃弹窗修复已在2bbe1aa92集成；保留模糊并增强正文/次级文字对比，低效果和无backdrop支持仍为实底。Mac Chromium/WebKit真实窗口像素及原回归64/64通过，最低正文5.56:1、次级4.88:1；请只对受影响WebView2弹窗核对亮暗/高低效果、长内容滚动与嵌套焦点，确认不是单纯透明或意外实底，不重做全站页面/模版逻辑。受既定RDP硬件例外约束时保留原失败事实，不伪造绝对帧率通过。

运行包名称版本已确定2.97.0-desktop023.names.1，包SHA256为afb72a4c792c1ca70a0fcd9209d212c78249281d2d7bc42f8eee63aa4464b26b。本条用于后继制品对照，不表示tgz或完整bundle已上传可下载；L0必须提供真实可取位置后再做制品验收。

平台检查保持隔离，不改日常配置、不做真实WebDAV或上游推理、不接管其它用户端口/服务、不自行发布。回交到代码codex/0.2.23-windows及证据docs/0.2.23-windows，明确依赖基线、完整SHA、命令/结果、未完项与清理回执。每批可交付成果完成即提交推送，并更新本入口；只在本机commit或聊天报“完成”不算交付。

## 共享基础已经具备

U=当前用户HOME/.codex-desktop，D=U/desktop，H=U/opencodex/home；固定指针U/desktop/manager-location.json，v2元数据不重复活动根。启动、切换、IPC、卸载已转新版规则，旧迁移入口明确拒绝。运行程序位于实际安装根I/runtime，与D/H分离；managed-only解析器要求固定Node及激活证明，不扫描旧安装、旧HKCU、PATH或独立ocx。

固定依赖：Windows I/runtime/node/node.exe、I/runtime/node/node_modules/npm/bin/npm-cli.js；入口I/runtime/bin/ocx.cmd，包前缀I/runtime/opencodex。主线正在准备完整离线供应，不要求你读取日常旧程序来冒充集成。平台R1可使用明确构造的隔离夹具；实际完整安装/更新制品项待L0交后继制品SHA补验，不能记为已通过。

本机共享检查：data_root 55、IPC 48、workspace 8通过；安装器38项普通检查及固定I布局真实npm6项通过。首轮全Rust回归1333通过、10失败、10忽略：其中4项正是windows_startup_root旧规则测试，另5项目录/IPC消费者已由本机修正，1项200ms进程夹具在隔离重跑通过；保留首轮失败，不把这些数字当整版验收。

## R1 历史写集与来源（不作为R2重复开发指令）

以下为R1已派发与接收的写集；R2的可改范围见前述新派发节：

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

回交必须推到可取得的代码/文档贡献分支，给完整SHA、依赖基线、合同、环境/命令/结果、未完原生/制品项及清理回执。不能只给Windows本地临时路径。L0核验后统一集成；R2缺口及R4实际运行包/安装候选仍按前述新派发节继续。

后继交接补记（2026-11-07）：2bbe1aa92代码批次已推送，包含离线资源stage配方205d72b1a、设置37项新版路径回归cdf3585e3及公共材质修复。原33d2为R2首派来源，已由本段明确基线替代；新增同步Service/LocalFile与恢复后继仍由L0继续集成，不能据此提前发行。
