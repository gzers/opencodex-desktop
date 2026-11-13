# Windows 0.2.24 W0开发回交

接令依据：docs/governance-main 3d5c1b302301ad6cc97f8f4ed5b649575892a72b，TEAM-0.2.24 revision1及IMP-34。代码共同基线origin/main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b；未合并旧原型分支。只写新增windows_official_runtime024/**和test/windows024-native/**；父mod、共享Schema/入口/状态/预算、CLI及安装器待Mac接口提交。

## 第一批：身份/私有记录/路径

- 代码分支codex/024-windows-native，源码提交779d820159403f1edad25ef0ad7fc4a62a3bac59；清理编译缓存后实际核验HEAD 12c69e27e615fc3d27dc3d341a9f66d3e0e72903。首次提交误含测试target产物，已在下一提交移除并增加test/windows024-native/.gitignore；未改共享根.gitignore，未改历史。接收以12c69e27e版本树/净diff为准。
- 提供真实Win32 TokenUser/session/elevation，显式当前TokenUser owner的User/SYSTEM私有目录/记录创建，现有ACL只读拒绝不修补，逐层父路径pin、文件ID/单硬链/非reparse核验、有界不可变记录。尚未接产品，类型仅进程本地Rust合同。
- 独立Cargo宿主直接path引用apps中新模块，无复制源码、无Tauri或父入口改动。4项真实原生测试通过：当前TokenUser，原记录/目录分享锁/身份，旧宽权限与硬链接拒绝且不修补，设备/ADS/dot等路径及长度边界拒绝。cargo fmt --check、cargo test --locked --offline、cargo clippy --all-targets -D warnings均exit0。
- 首次开发编译发现OpenProcessToken从错误windows-sys模块导入，修正为System::Threading后通过；属于本批实现缺陷，不冒充原官方包故障。原执行错误由对话工具输出观察，未伪造丢失的原始stderr文件。
- 实际命令、工具链/Token身份、来源文件摘要见.adg/evidence/0.2.24/windows-dev/w0-batch01-20261110/result.json及同目录日志。测试为新临时小目录，RAII释放句柄并删除精确新目录；未操作日常HOME/官方运行包/账户/Profile，无服务启动、最终全量或发布。
- 下一批：原始创建句柄、无先运行窗口的Job分配/Resume、成员/原退出和IPv4/IPv6监听事实。产品整链/正式恢复仍未接线；不重复r5单用户可行性链。

最小共享接线建议：仅在Mac明确模块/类型接口提交后，由Mac注册父模块，并将已解析程序/HOME和共享deadline传入平台原语；记录序列化、官方HMAC/ready准入、状态机与总30/5/10预算仍由共享层持有。W1未放行。

## 第二批：原创建句柄/Job/退出/监听

- 代码HEAD b2e89665d2864ea8e29bbaa91ac7843dff7ac4cb已推送；相对共同main的净变更仅W0新增目录和独立Cargo宿主。没有修改windows_private_create.rs、windows_query_job.rs、runtime_lifetime.rs或任何未分配旧模块。
- 新OwnedRuntime接受共享层已解析程序/HOME/环境/输出句柄，直接CreateProcessW挂起创建，限制继承stdio句柄，原创建句柄采集TokenUser/session/elevation/FILETIME，核对固定程序文件ID；在匿名User/SYSTEM私有Job分配/验证后才ResumeThread，禁止breakaway。新ROOT本体不经PID重开；子进程只来自该Job有界成员列表，重开即核对Job成员/代次/SID。无需官方旧维护lifetime字段。
- FamilyState保留已观察成员句柄/真实退出码，Job账本和完整有界PID列表不同则unknown/WouldBlock；根进程退出不足以证明子进程退出。wait_normal_exit仅观察，不发信号/HTTP/强杀，使用共享调用者绝对deadline；已超时或迟到结果也不补通过。force_owned_cleanup另有ForcedCleanup返回类型，只面向保留的自有Job，不冒充正常Stop。
- 原生IPv4/IPv6 TCP表读回绑定保留句柄代次和Job成员，前后成员变化保持未知；无全机进程归属/命令扫描。最终若另一进程占用原监听端点，仍拒绝“监听已退”成功，不终止新监听者。单次表快照不宣称连续监控或给未曾保留句柄的历史子进程编造退出码。
- 普通令牌与提升令牌同源各11项通过（4个身份/私有路径单测＋7个原生进程组件用例），cargo fmt/test/clippy均exit0。证明真实Unicode/引号/反斜杠参数及同HOME注入、Resume前无用户代码执行、错SID/PID/FILETIME/HOME拒绝、原非0退出码7保持、根exit0仍有活子进程拒绝完成、超时不强杀、显式强清exit242独立、两族监听及新占用者拒绝、大小写重复环境在启动前拒绝。普通和提升不是不同SID，不冒双SID或最终矩阵。
- 两种模式实际SID/session/elevation、源文件摘要、执行命令/耗时/exit和原生事实Debug输出分别见w0-batch02-ordinary-20261110/、w0-batch02-elevated-20261110/。提升helper原保留句柄exit0。未启动官方服务、改官方包、运行r5可行性链、读写日常HOME或创建临时账户/Profile。新临时程序只是本批独立Rust测试fixture。

### 第二批开发失败与保留

首次组件用例直接指向Cargo以硬链接发布的fixture制品，被平台“单链接程序”准入拒绝；测试清理Drop先于被固定目录句柄释放又导致析构panic/abort。改为新测试目录中单独复制的可执行文件、明确释放fixture句柄后清理，不改平台单链接保护。随后发现文件和Job的FA映射不同，私有Job权限误用FILE_ALL_ACCESS核验；已分离SE_KERNEL_OBJECT/Job完整访问掩码，仍只允许本SID/SYSTEM。Clippy的question_mark提示修正后通过，原语无保护放宽。

开发首错原输出在对话工具观察；未留完整原stderr文件的轮次只记录观察，不重建伪原件。第一轮4个ocxd024-process-68932-{0..3}小型失败目录仍保留，原小文件已只读归档到cleanup-readback.json；自动安全审查拒绝了递归删除命令，原因返回blocked by policy，因此未换工具绕过。成功的普通/提升测试自有进程/Job/句柄和目录正常清理，不将这4目录记为已删除。无其它本轮ocxd024-native私有目录残留；不是全机无进程证明。

### 当前交付/未完与接线建议

W0请求的身份/受保护记录/路径、保留原创建句柄/Job集合、正常退出和原生监听事实已实现并可由独立宿主编译测试；产品尚未调用。所有返回值是本进程Rust类型，不冻结IPC或持久Schema，不复制共享状态机/HMAC/包准入/事务总预算。官方2.99 revision5仅作为已验证来源与技术输入，未把C#脚本复制进apps或整分支merge。

Mac可审查净diff main0828a3955..b2e89665d。第一批曾误含编译产物，应接最终净版本树或同时消费779d82015＋12c69e27e，避免单独拾取含target的中间提交；第二批为单独b2e89665d。父infrastructure/mod.rs由Mac按真实公共接口注册，不由Windows猜类型/文件名。推荐共享层持有OwnedRuntime生命周期：输入已核对的绝对exe/cwd/HOME，传最小环境和输出句柄；调用create_suspended→绑定记录/HMAC前提准备→resume；共享官方正常停止后，在同一个总Stop10秒deadline内调用wait_normal_exit、listener_exit_readback及共享恢复核验。错误不降级为对象不存在；强清返回值单独记录。

未完：Mac P24-00/01共享合同、父模块/命令/产品入口接入、真实包/HMAC/ready/共享恢复与实际总30/5/10秒产品链验证、产品持久身份/并发事务/后像归属；W1的Windows CLI/PATH及NSIS/MSI接线等待Mac真实接口提交，尚未实施。W2最终固定集成候选/双SID控制台与RDP/安装更新卸载验收当前未跑。未改IMP/TEAM/共享索引，正式进度由Mac接收后回写，未发布。

## 2026-11-13 W1第一批：Profile/HOME修正

已fetch并合入共同main cbc6174de；消费docs/governance-main 61f0f99d8的TEAM revision5、W1接口r1和W0 finding。代码eb818d601（codex/024-windows-native）已推送。LaunchSpec分别传user_profile/opencodex_home：HOME和USERPROFILE取前者，OPENCODEX_HOME取后者，RootBinding.home/home_id及私有目录pin继续只证明实例H；不对Profile修权限。大小写环境覆盖仍拒绝重复输入。

独立原生宿主process目标7项通过，包含两个不同合成目录的三变量真实子进程读回、参数/挂起窗口、旧SID/PID/FILETIME/HOME反例及受影响进程回归；fmt/clippy exit0。执行为提交前相同源码，SHA256和原始日志位于.adg/evidence/0.2.24/windows-dev/w1-profile-20261113/。Python编辑预检曾因GBK默认解码失败，尚无文件变更；仅工具输出可核，不重造stderr。原生测试无首错。

本轮PID115732测试目录已清理；历史4个W0失败目录保持原事实，未将其记为本轮成功清理。共享接线建议：将ProcessContext.environment_policy.home传user_profile，opencodex_home传独立同名参数；RootBinding.home仍表示实例H。产品调用/完整官方链和W2未执行。下批为PATH原语和安装器资源；不改TEAM/IMP/共享索引，不发布。

## 2026-11-13 W1第二批：当前用户PATH原语

代码4269e4499已推送，新增windows_cli024.rs并由独立宿主path引用；产品父mod/共享事务未改。r1三个公开Rust函数保留原类型、UTF-16原字节及缺失，REG_SZ/EXPAND_SZ以外、奇数字节、非单终止/非法UTF-16和超过256KiB拒绝；当前TokenUser的Global SID互斥验owner/受保护DACL/原句柄，2秒等待、遗弃/未知拒绝；HKCU固定Environment/Path，拒绝线程模拟，expected完整相符后写并读回，不对外部变化做回滚。不用setx/HKLM、无PATH清洗、无新JSON。WM_SETTINGCHANGE/Environment使用1000ms每接收窗口有界调用，非整个桌面统一1000ms；不承诺已启动终端环境更新。

attempt01原生7项通过，clippy首错manual_is_multiple_of及chunks_exact_to_as_chunks，原日志保留。修正后attempt02定向8项、fmt/clippy全exit0：真实类型/字节/缺失还原、错误expected、错误类型/长度/终止/超限、只读句柄拒绝、外部写后变化保持、同SID双进程一成功一冲突、未知/遗弃互斥拒绝、自己窗口通知/失效窗口错误/挂起窗口1014ms失败；日常PATH前后相同且只读，无输出完整PATH。测试子键逐个删除读回，残留清单为空；窗口/句柄/子进程已释放。.adg/evidence/0.2.24/windows-dev/w1-path-20261113/保存两次不可覆盖日志及来源摘要。

最小共享接线：Mac注册windows_cli024并在terminal_cli现有确认/恢复事务中调用read/replace/notify，明确记录写后失败的未知后像；desired=None仅由共享精确撤销允许。当前不是用户终端启用/撤销/卸载整链通过。未改官方2.99/日常配置，不跑r5/W2、不发布。
