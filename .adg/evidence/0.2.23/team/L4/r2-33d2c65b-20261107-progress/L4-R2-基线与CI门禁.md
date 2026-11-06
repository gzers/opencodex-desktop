# L4 R2 首轮交接：基线合并与MSVC CI门禁

2026-11-07，Asia/Shanghai。依据 docs/governance-main@d707d28888f065a2a88a0c6f67f31f3608409d02 的 WINDOWS-0.2.23.md，执行已放行R2。

已合并指定集成基线33d2c65b58ba8880d6deea7b52d69117238c3d33并保留原贡献；代码贡献codex/0.2.23-windows@400c4e7f4a765b18e4406ba889d13fe5021ccb9b已推送。两父为e011ae2ee582d738fb148b7370e39e41c60c8467与33d2c65b58ba8880d6deea7b52d69117238c3d33。add/add冲突仅windows_user_scope.rs，已核对stage3逐字节等于首批helper、stage2等于已验e011类型修正，保留typed ACL声明，不以全局ours/theirs覆盖共享语义。合并后指定SHA是HEAD祖先；相对该集成树仅helper一文件11增/11删，无新增公共文件改动。

**当前MSVC验证入口**

本机仍只发现GNU Rust，未发现常规MSVC/VS/SDK。为实际获得Windows MSVC结果，建立面向codex/0.2.23-integration的draft PR #71：

~~~text
https://github.com/gzers/opencodex-desktop/pull/71
head: 400c4e7f4a765b18e4406ba889d13fe5021ccb9b
base at creation: 33d2c65b58ba8880d6deea7b52d69117238c3d33
PR merge ref observed: 74583c49f1a137d05b28c78d0bbf01a272ab45d9
CI run: 42249148982
~~~

GitHub连接器创建PR返回403 Resource not accessible by integration，随后使用仓库现有Git凭据在内存中完成API调用，凭据未写文件、未输出。只创建draft，不合并、不发布、不改工作流。既有CI通过pull_request触发；CI执行的是PR合并树，不能把它无条件等同安装候选或最新集成树。

CI backend-windows job id127999232404已取得windows-latest runner并完成Node、Rust、前端构建准备；cargo fmt --check失败。该失败使普通clippy/Windows backend regression被跳过。always条件的Verify isolated native credential protection仍在执行，最终MSVC编译/测试结论须等该job终态，不记通过；不取消重跑、不修改门禁绕过失败。详细快照另交。

**可由L0立即处理的格式门禁**

本地只读cargo fmt --manifest-path apps/desktop/tauri/Cargo.toml --all -- --check复现退出1，共432行输出，命中10个L0共享文件：

~~~text
apps/desktop/tauri/src/commands/about.rs
apps/desktop/tauri/src/commands/mod.rs
apps/desktop/tauri/src/modules/model_config/config_file_v2.rs
apps/desktop/tauri/src/modules/model_config/file_objects_v2.rs
apps/desktop/tauri/src/modules/model_config/mod.rs
apps/desktop/tauri/src/modules/model_config/provider_auth.rs
apps/desktop/tauri/src/modules/model_config/sync_local_v2.rs
apps/desktop/tauri/src/modules/model_config/sync_local_v2_tests.rs
apps/desktop/tauri/src/modules/runtime/mod.rs
apps/desktop/tauri/tests/runtime_bundle_real.rs
~~~

未命中L4 helper。仅检查，没有对公共文件运行写入式fmt，也没有扩大写集。原始local-rustfmt-check.log与local-format-summary.json可供L0修正并发布后继精确SHA。不要以移除fmt/clippy步骤、ignore测试、GNU通过替代MSVC必需结果。

**Windows smoke实际覆盖**

CI windows-smoke-evidence job id127999232579已completed/success；原始日志ci-windows-smoke-evidence.log已从连接器取回。其子步骤包括process token/smoke语法、安装矩阵及隔离Profile准入、普通账户first-logon Profile-only、恢复等待器测试。日志明确若干nativeRecovery/fullVersion/performance gate为false；Profile-only检查创建runner内临时账户并读回SID/Profile，但并未运行两个真实SID的管理器。因此仅说明既有smoke脚本检查通过，不充当本版GUI/端口/DPAPI/安装更新通过。

Artifact id13517894778，name=windows-process-token-evidence，ZIP摘要sha256:0bcb5b163cfe2d985b55a01a43c71bd7fbdadec618c0535e8729d4d4135ba96d，API size=6911字节。连接器提供文件引用成功，但Windows本机下载重定向/临时文件域名均DNS失败；未将“获得链接”写成ZIP已本地校验。CI原始日志已可Git回交，归档ZIP仍由Actions按artifact id获取。

release-tools job已成功，属于既有工具单测，不是本次发布。macOS backend/frontend仍排队，R2后续状态以最终CI快照为准。

**其余前置与限制**

用户已说明第二账户“已有其他受控执行通道，稍后提供”，本轮尚未收到；quser仍显示15119会话2 Active、liangjh23会话3 Disc，两SID真实产品并行、同SID跨会话/提权、跨SID秘密/pipe拒绝、端口冲突/进程归属仍未验。不修改日常账户/Profile/注册表/ACL、不注入其它会话，不把CI Profile-only当成该证明。

已读.github/runtime-platforms-023.json及build_desktop_runtime_bundle.py。Node26.8.1/npm11.19.0与runtime 2.97.0-desktop023.names.1的版本/hash均有配方；runtime tgz目标摘要afb72a4c792c1ca70a0fcd9209d212c78249281d2d7bc42f8eee63aa4464b26b。配方存在不是实际tgz/bundle/NSIS/MSI已交付，等待L0真实可取资源描述符和候选摘要，不另造同版本新包。材质补丁未到，不重测玻璃。

前一批isolated-test-temp的自动审批拒绝清理状态保持原样，本轮未重试删除；不更改其回执。当前代码工作区干净，未启动真实同步/推理/日常管理器操作，没有运行真实安装/卸载或发布。

本回交是R2可立即消费的门禁/环境事实，未关闭MSVC/两SID/安装候选验收；继续跟踪同一活跃CI run，不因观察超时重启任务。
