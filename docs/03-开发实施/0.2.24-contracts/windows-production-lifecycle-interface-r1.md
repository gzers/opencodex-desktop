# Windows生产生命周期共享接口与定向接线 r1

revision: 1；2026-11-14；归IMP-34 P24-W1/04。新实现只提供可编译共享接口，**尚未把产品Windows生命周期切到新后端**。代码精确SHA见本合同末尾及同批证据integrated-source.json；MSI贡献0f1dc195e按ed01402a7/3eb32a3bd/42c030f6e/6b33016ba接入后净树一致，不重跑真实MSI/终端组件。MSI临时管理员通过、普通用户及清理未完分别归接收记录，不能用本合同抹平。

## 唯一共享入口

**精确共享代码输入：main 35f41ecc0e700a3a77d121cc2dccf7258c327bc1**。该提交包含已接收的四笔MSI测试贡献，以及1da73491f共享typed验签/Stop检查和35f41ecc0启动准入/预算接口。实际Windows编译/产品EXE需在此源重新构建，不用MSI-only旧exe替代。文档输入为包含本合同的docs/governance-main提交，用户提示词提供其真实SHA；本合同不是预填候选号。

模块 **modules::runtime::official_lifecycle024**；所有入口为同一main真实实现，不复制HMAC、猜路径或创建另一个包选择/记录Schema。

| 入口 | 实际合同 |
| --- | --- |
| ActionWindow::begin(action, started: Instant, parent_deadline: Instant) -> AppResult<ActionWindow> | 在公开动作入口只构造一次，Start min(parent,started+30s)、Stop min(parent,started+10s)、Restart min(parent,started+30s)，stop_deadline另取min(outer,started+10s)。deadline/stop_deadline只读；remaining到期拒绝，不重置。 |
| LaunchPlan::prepare(&SharedRuntimeExecutable, &ProcessCommand, deadline) -> AppResult<LaunchPlan> | 仅Start，command.executable必须同当前版本化stable entry，HOME必须等于实际working_directory/Profile，OPENCODEX_HOME独立，官方版本2.99.0/完整current镜像准入；内部exe为准确Node，args为准确bin/ocx.mjs、start、--port及明确数字（None为0）。不是Shell命令。 |
| LaunchPlan getters | executable/arguments/environment/user_profile/opencodex_home/program_root/version/deadline；无Clone/serde/公开字段，不把JSON/PID等当native许可。 |
| check_current/verify_full(&same_runtime) | runtime Arc/实际根/stable/current/清单及原deadline复核；verify_full另读完整镜像后再核当前。不是跨动作缓存。 |
| observe_official024(runtime, home, parent_deadline) -> ServiceObservation | 同连接HMAC health→ready→HMAC health，原私有record和完整来源前后复核；总窗口min(parent,当前+原Status5s)。无记录/未知/错误不返回Ready；pending/failed不变为ready。 |
| ServiceObservation getters/check_current | pid/port/ready/version/record_sha256只读；check_current绑定同runtime Arc、H、原record/source与观察截止。无secret/serde/Clone，不是OS进程归属、句柄或恢复写许可。 |
| RetainedService::verify_live(&mut self,pid,port,record_sha256) | **由实际Windows原创建句柄/Job拥有者实现**，仅当前内存/受控保留句柄证明；在第一次HMAC后及第二次HMAC＋record/source检查后、POST前各调用一次。不可凭旧记录构造“总返回Ok”的生产实现。 |
| request_stop_official024(runtime, home, pid, deadline, &mut impl RetainedService, phase) -> StopAccepted | 原官方HMAC、private token、/api/stop能力、PID及同TCP连接POST、sharedTeardown performed/not-owned、record与程序前后稳定不变；不要求旧desktop_lifetime。用调用者Stop绝对deadline，不另起10s。 |
| StopAccepted getters | 仅官方响应中的pid/port及两次checked原record摘要；**不是正常退出/Job空/端点释放/原件恢复证明**，不能直接映射LifecycleResult::Stopped。 |

HTTP新接口内部复用原stop_official_with_checks：旧stop_official_with_witness行为通过no-op额外检查保持，旧Windows维护包Binding/Witness分支未删除；新官方024产品消费者必须明确走新接口，不能以旧分支的成功或回退掩盖未知。

## Windows生产适配与所有权

新增单写 **src/infrastructure/windows_official_lifecycle024.rs**，复用既有windows_official_runtime024真实Win32原语。模块对上层提供具体拥有者/状态，不接收IPC任意路径/PID/返回true的回调。用现有ProcessContext/ProcessCommand/实际D/H/I，明确每个实例/账户身份；共享EnvironmentPolicy清空继承环境并白名单补齐。主线程/status采集不因运行句柄存储锁形成死锁；读写/更新/停止动作互斥沿既有SharedRuntimeInstall/动作门，不能以第二个全局JSON状态机替代。

- Start：同动作deadline内先LaunchPlan完整准入→create_suspended把实际Node/args/Profile/H/environment和当前私有输出文件传给W0→保留原创建句柄/Job→核Profile/H/source→resume。没有先运行后按PID“补Job”的窗口。root是官方launcher，不假定它就是service；Bun service需在原Job枚举中按PID/FILETIME/SID/镜像文件ID及同current来源绑定。原CLI exit0单独不算ready。
- Ready/Status：共享observe提供HMAC/当前record/ready事实；native owner必须独立证明record.pid对应原Job当前代次/准确镜像/H绑定和端口listener，重读原记录/程序及原deadline。只有二者同时满足才Running/Started/实际端口，signedHTTP单独不授权OS归属。Status5s内查不到保持Unknown，不显示上次端口/就绪冒通过。
- Stop：捕获所有原Job成员/保留代次/原listener表、private record摘要并落实RetainedService两个发送边界。调用新request_stop官方还原接口成功之后，同一个Stop10s内以原句柄等全部受控成员正常退出、所有原listeners内核缺席、HOME record/pid缺席、同HOME当前受控集合无新成员/官方supervisor事实已确认且source未变；复核到最后再返回Stopped。非0/强清/不可读/新监听者/只root退出一律拒绝成功，ForceCleanup只能独立失败清理事实，不能转普通Stop证明。
- Restart：同一个公开30s，Stop子窗口从原动作开始算最多10s；Stop完整通过后才Start，剩余outer deadline不重置。旧proc/record/Job/pid/source/端口变化，启动未就绪及恢复未确认均不得后继Start。Auto端口仍由既有port::prepare_start选择保存、冲突有界重选同Start预算；显式端口冲突不换号。
- 生命周期拥有者不得在Start成功后丢原句柄或Drop自动杀服务。失败/取消/超时仍保留确切owner待显式处理，不为测试清理宣布成功。窗口关闭、管理器退出/重启、更新/切换H时不得把活owner丢掉后凭盘面记录重建原生权限；无原保留句柄的既有服务只能Unknown/受限恢复，不能伪装fresh normal exit。此连续性是整版要求，若当前架构不能完成，应在完整状态表单列；不以“当前单进程normal轮通过”宣布跨管理器恢复通过。
- 当前shared恢复拥有者的Windows全机扫描和旧desktop_lifetime均不是新024的替代证明。只在准确官方024/current上下文切到新具体owner；其它来源/Mac分支语义保持。观察无法确定服务或manager保护时未知拒绝，不将“未读到”当不存在。
- 客户端恢复：官方/api/stop返回不单独证明所有客户端文件已恢复。用隔离合成Profile在Start前/Ready/PreStop/Stopped后记录本次声明scope内Codex/Grok/路由/认证/配置字节及原权限/存在性；两稳定锁例外只按原明确有限释放合同，不删除/重建旧globalL或收养未知文件。未有Windows独立原件/恢复权限的范围必须继续Required/Unknown，不复制POSIX实现硬当Windows原件证明，不把客户端恢复未完错误说“未启动”。完整恢复持久协议若缺共享已定格式，回交最小具体设计/证据，由集成线定下一接口；不捏造新恢复journal或冻结第二事实源。

## 可写集：W-L1生产接线，直到Git回交由Windows单写

本机发布本合同后不并行改下列临时写集；Windows可在此范围连续排查/修复/定向验证，小批及时push，无须每次结果回Mac。

1. 既有Windows独占infrastructure/windows_official_runtime024/**，新增windows_official_lifecycle024.rs及同名_tests.rs、test/windows024-native/**。这是原生owner实现和受影响测试，MSI脚本旧通过结果不重跑。
2. 明确共享消费者仅**Windows条件分支/注册**：infrastructure/mod.rs、process_runner.rs、status_source.rs、attested_status.rs；commands/mod.rs、runtime_recovery_v2_owner.rs、runtime_recovery_v2_sources.rs；state.rs/lib.rs的Windows owner状态注册。可以新增commands/windows_lifecycle024.rs并由父命令明确路由。共用签名/健康HTTP核心只消费本合同新类型，不复制/改其算法；发现共享接口实际缺陷回交具体最小补丁原因，不能临时放宽验证。
3. modules/runtime/official_lifecycle024.rs及其_tests.rs、infrastructure/official_runtime024_transport.rs及attested_status_tests.rs允许本接口的Windows编译/实测所需适配，保持既有Mac/transport断言；modules/process/mod.rs仅ProcessRunner默认扩展或Windows owner注入且保持现有fixtures兼容。新增status_source对应定向测试及commands/windows_lifecycle024_tests.rs允许。
4. 不改runtime_lifetime.rs维护协议或官方包，不把旧lifetime写入官方记录；不改全部Vue、其它模型/同步/网络/存储/原件格式、构建workflow或发布门。bin/ocxd及新的跨进程owner协议不在本次自动放行，若manager重启连续性确实依赖它，先交精确进程/句柄/私有IPC/持久输入草案及反例；当前未知拒绝保持，不能偷删连续性要求。根清单/依赖若native接口已足够不改；确需新增特性给最小diff后由集成接。
5. docs仍只贡献docs/024-windows-dev的新windows-dev/<不可覆盖轮次>/**及windows-dev-handoff.md；IMP/TEAM/正式核心事实由本对话接收。不得在代码分支提交docs/.adg；保护其它并行修改/共享README。

旧MSI-only写集限制本轮生产接线由上述具体授权替代；普通用户MSI/清理缺口仍留存，不为推进本线而要求改机器策略/提升普通验收令牌。临时管理员MSI成功只证明用户另行授权的具体场景；不重复同一管理员安装回归。

## 固定验证输入与受影响定向表

源输入为本合同末尾的精确main SHA及对应docs SHA，不用贡献历史测试EXE冒新的产品消费者。原官方2.99 tgz SHA **b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c**；Windows完整归档runtime-bundle-0.2.24-win32-x64.tar.gz **14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7**，274033721字节；descriptor **42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a**。重用Windows已保留且重算摘要的输入/cache，不改原包、不重复搬大制品；所有新的MSVC产品/测试EXE记录实际源码/tree/摘要/工具链和构建配置。

使用一名专用新建的普通测试账户及真实Profile的自有I/D/H、合成客户端，无需MSI安装（生命周期测试与MSI验收分离，不冒充安装器通过）；本轮只单SID，实际用户token/会话属性记账，日常账户PATH和配置不写。可编译debug和原发布profile针对30/5/10预算有差异时分别报告，最终预算结论用固定release参数，debug失败不得回填成功；原包服务只监听loopback，不请求真实上游模型/业务同步，测试token/secret不入公开日志。

| 项目 | 必须证实 | 反例/失败口径 |
| --- | --- | --- |
| 第一次真实生产Start | 实际应用命令路径/runner新消费者→原W0 create_suspended/Job→当前官方就绪，30s；Profile/H分离/当前Node脚本/原记录/服务PID/Job/listener同代 | 无record/错H/错SID/FILETIME/source、伪HMAC、pending/未就绪/超时均不算Started；native组件通过不替代消费者通过 |
| 高频Status | 同当前native owner与typedHTTP/record/same source；实际就绪与端口，≤5s | 缺句柄/旧record/Job变化/原端口被别人监听为Unknown，不用全机扫描/旧lifetime成功推断 |
| 真实普通Stop | 原连接HMAC/owner两个边界→官方client teardown→原Job所有受控成员正常退出/原listeners缺席/记录释放/声明客户端读回，≤10s | root退出但子还活、非0、强清、HTTP200但native不明、新listener/late子/路径或权限变化拒绝，不覆盖首错 |
| Restart与第二Start | Stop子10s在原outer30s内，后继Start继承剩余；停止后同H可正常再次启动，无旧记录/旧进程冒认 | Stop不完整/恢复未确认时不进Start；不得重置预算或拼接两张不同source通过表 |
| 取消/失败/更新/管理器重启 | 当前所有权保管/失败后像/权限与UI状态如实保留 | 不能Drop丢owner后宣称干净；无法跨进程连续性如实列未完、给最小接口需求 |

每轮先写可证伪假设、一次收集所有互不依赖项（可continue-on-error但总accepted=false直到全部必需项过），首行错误/阶段/原始输出分别留存，再批量修复；有充分新事实才新轮，不“一关一轮”。沿原完整断言/绝对预算，不跳不可读对象、不把tasklist成功当清理。已过CLI参数/官方只读/NSIS/MSI组件非受影响不重跑，**固定候选前不跑W2/双SID，不发布**。

## 清理与接收范围

所有本次新进程/Job、账户/Profile、原锁/原PATH、ACL/文件存在性分别归档核验后清理精确自有对象。旧临时管理员SID尾1040的Loaded Profile/error32/System与Registry共享占用是既存未完，不杀共享系统进程、不强卸载hive/重启服务或机器；旧自动审批删除拒绝不绕过不重试。新失败现场保留原目录，成功不覆盖失败；清理未完另报身份/数量/字节/原因。

本机接口测试包含4项启动准入/原预算、12项transport（3新typed/expiry/publicStop用例及两个native边界反例）、20项既有owner回归（1原ignored保持）；这些是合成loopback与host证明，不是Windows实机/产品接线通过。代码接口可消费与生产已接通分别记录，后者以Windows真实调用点/原包动作证据为准。整版其它未完（NAME、Mac根恢复/一般客户端/代理/官方交付门/最终签名后entry来源/候选）仍按IMP-34推进，不缩减。
