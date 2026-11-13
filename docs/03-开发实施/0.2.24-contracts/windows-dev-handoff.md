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

## 2026-11-13 W1第三批：024 MSI/NSIS资源与目录组件

安装器代码f3b40e71c、独立验证脚本b8f1bba91ab611765f23aad6935c21bb66490577已推送，后者是本次代码分支最终HEAD，包含共同基线cbc6174de及前两批。净diff核对无越出TEAM revision5/W1接口r1写集。WiX映射真实runtime-bundle-0.2.24-win32-x64.tar.gz、official-runtime.tgz及manifest/pin；保留既有组件GUID/注册键作为安装身份，不代表仍供应023。NSIS继续消费同一个实际stage映射；删除PREINSTALL自动调用022卸载器的旧兼容分支，保持默认Programs I/明确自定义I。MSI原有machine022只读阻止条件未扩大；本批无自动迁移、无live runtime递归清理、无新卸载/PATH猜测命令。

由本线真实stage_runtime_bundle_024生成Windows输入；原官方2.99 SHA256仍为b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c。归档274033721字节，SHA256 14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7；真实Windows descriptor SHA256 42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a。没有复用Mac摘要、执行依赖生命周期或启动官方服务。

固定CLI2.11.4/实际candle-light3.14.1/NSIS原生编译MSI和NSIS均exit0，资源清单/编译日志/生成输入/制品摘要完整保存。打包EXE明确使用本线standalone Rust测试fixture，测试identifier/upgradeCode，不是产品候选；NSIS测试禁WebView下载安装及压缩，出现unused install_webview2 label警告，未记为失败。MSI数据库确认四个024资源且无旧维护包、无ALLUSERS/通配数据删除；静默原生成本解析default/custom两格通过，未执行安装动作、未创建I。NSIS从真实生成脚本逐字提取.onInit/RestorePreviousInstallLocation/SetContext，仅替换测试注册表key，独立原生编译并执行default/custom/remembered/explicit-over-remembered四格通过；目录均未创建，测试key删除读回。

首次MSI表探针把COM void返回混入结果，修正后第二次又把空runtime-bundle RemoveFolder过宽判作数据删除；两次原始错误保留。第三次默认UI COM探针未返回，126.8秒后仅终止匹配命令/原句柄的自有测试进程（exit -1），没有执行Install/杀系统安装服务；第四次静默UI并以父进程30秒限时，exit0。NSIS首轮因Python把含空格/D=整个参数加引号而忽略自定义位置；修为原生要求的最后且不加引号后四格通过。这些是探针失败，不冒充产品故障。最早stage执行源单独保存，最终脚本只补sys.dont_write_bytecode避免导入共享脚本时产生pycache，语法检查通过，未重复昂贵stage。

不可覆盖证据.adg/evidence/0.2.24/windows-dev/w1-installer-20261113/：52份原文件逐字节校验归档，原始descriptor、资源/制品摘要、执行命令、失败/超时原件、最终源文件摘要和净写集核对；.gitattributes禁止原日志换行转换。本批无产品安装/正式W2验收结论。代码eb818d601/4269e4499/f3b40e71c/b8f1bba91可按主题审阅，docs前两批3f371e2e5/13020b3a0，当前docs提交提供第三批证据。

### 本轮清理与明确未完

本轮process fixture目录和PATH/NSIS所有私有测试注册表key已清理，相关测试进程/窗口/Job/句柄已退出或释放；MSI超时探针原句柄退出已记录。不宣称全机无进程。原4个W0目录仍只含已归档空日志/空home；精确非递归清理命令被自动审批以blocked by policy拒绝，命令未执行、无换工具绕过。新生成target/installer-w1的归档后精确清理也被自动审批以同一理由拒绝，仍保留11808文件/2108815151字节；首次stage产生的两份共享脚本pycache亦列为清理未完。cleanup-pending.json给出路径/读回。历史证据不改写，不将“已归档”说成“已删除”。

Mac后继最小接线为注册新增windows_cli024（基线infrastructure/mod.rs已注册windows_official_runtime024，勿重复注册），将ProcessContext两个HOME输入分别传入，并在已有terminal_cli事务内消费三个PATH原语，补真实卸载撤销helper。共享终端启用/禁用/恢复/卸载整链、实际产品安装升级卸载保留及固定候选双SID/W2仍未验；当前只完成被放行的原语/资源组件。未修改IMP/TEAM/共享索引、官方2.99或日常配置；未重跑旧r5、未跑W2全量、未发布。

## 2026-11-13 W1-C：MSVC产品、终端事务与卸载hook回交（未全通过）

已fetch并合入指定main83b27e9ae7d0919f57298e38f5f52fd887d50650和docs/governance-main6ee2d2103；消费TEAM revision7、windows-terminal-transaction-r1。贡献分支codex/024-windows-native两批已推送：9671a1511（终端私有记录1MiB修正/原生测试与隔离账户脚本），e004259feba4a077375db01a79f851470ed0ff74（非升级卸载先撤销终端、安装器探针）。相对main83b27e9ae净写集仅合同指定文件；未改program_selection.rs/paths.rs、共享清单/锁、官方2.99或Vue。Tauri构建生成的共享文件换行触碰已按相同规范化字节还原；未删除的windows-schema.json仍为未跟踪生成物，未提交。

### MSVC产品编译及本批修正

宿主原只有GNU Rust。独立工具目录准备真实MSVC14.44.35207、Windows SDK10.0.26100.0、rustc1.100.0 x86_64-pc-windows-msvc，环境仅当前进程，不改日常PATH/全局工具默认。原Microsoft payload和Rust官方manifest摘要核对；网络停滞/镜像按同一摘要恢复、原始下载日志保留。首轮offline check因缓存缺libsqlite3-sys0.35.0失败，后以未变Cargo.lock摘要133c182a6a2c87864fe97778797e46c7e999672690dc9fa3ee8e241aa4a9c13f补齐。

全产品MSVC cargo check --bins --lib exit0；cargo build --bins --lib（dev）首轮及修正后均exit0，实际Desktop/ocxd.exe已生成，测试目标no-run exit0。未做release签名/发布候选。保留38/39条产品及41条测试warnings原输出，未声称严格Clippy无警告。实际执行HEAD为7317cc94a合入基线加当时工作源码；out04精确源码/输入制品摘要已归档，最终提交保持已测产品内容；最终test另补“不允许argv失败在成功官方路径后被吞掉”的断言及1项PrivateRecord回归，不重跑昂贵旧组件。

首个授权范围内产品错误：terminal_cli_windows把16MiB传给W0 PrivateRecord，而后者只允许1MiB，导致新pending写后无法读回。本批改Windows分支为原生1MiB，并在publication前检查完整序列化事务/文件后像大小，不放宽W0上限或改Mac语义。真实隔离账户修正后可继续；最后单项MSVC原生回归private_reader_matches_w0_bound_without_touching_user_path通过（1 passed，1857 filtered，不运行全量），fmt及新增Python/PowerShell语法检查通过。

### 一个隔离账户每轮的真实执行结果

各失败/修正轮次顺序使用新的ocxw1c*普通账户，严格匹配TokenUser SID、原OS Profile、非提升令牌；不是双用户/双SID验收矩阵。日常账户不写Environment/Path。首轮创建后子进程被日常Profile访问权限拒绝，未执行PATH；后改本轮Public临时输入根，未授予新账户访问日常Profile。原失败不覆盖。

out04通过：实际ocx.cmd→MSVC产品ocxd.exe→同current Node/合成脚本的cmd完整参数（空格/Unicode/引号/&|<>^!()/百分号/尾反斜线/空参数）、二进制stdin、stdout/stderr及exit23；OS HOME/USERPROFILE和自定义实例H正确，选中合成版本9.0.0及Node/script精确位置均读回。合成probe.cmd是明确测试JS载体，不是官方包改名/改写，不拿其成功证明官方2.99。同current与H是合成程序的调用事实；裸ocx命令在新终端的解析仍未单独证明。

真实HKCU PATH启用/幂等/停用与后来条目保留通过；10格Enable/Disable×pending/command/path/record/notified中断，原pending保存后由新原生进程继续同方向，反向拒绝，最终清掉原pending；4类command/record/pending/PATH外部变化拒绝且原字节保留；显式通知中断后原事务保留并可继续。PATH私有before/after及最后精确还原有证据，无输出完整日常PATH。通知失败格是明确测试注入；正常路径真实调用Win32广播，不把注入说成真实OS通知故障。

actual helper通过：从未启用时只读成功且不创建固定anchor；pending Enable返回1且保留pending；额外参数返回2；正常停用返回0并撤销登记/命令/PATH。实际NSIS PREUNINSTALL宏编译成只执行原hook与“允许继续”标记的无删除探针，4格通过：未启用、pending Enable失败保留程序/原pending、升级保留登记/PATH、正常非升级撤销。没有执行完整安装器删除/服务生命周期或业务请求。

### 尚未通过：PowerShell与原包消费者

out04 PowerShell默认.cmd调用混合特殊参数返回1，stderr原文“| was unexpected at this time.”，无stdout；该格仍失败，没有改测试判成通过。早一轮cmd混合引号位置也触发同类解析错误；改为合法cmd引用后cmd完整逐项比较通过，PowerShell仍失败。现有.cmd入口在PowerShell默认native argument passing下的完整特殊参数保真仍须修；若采用native ocx.exe或额外PowerShell入口，应先有明确合同，不能在本批静默改入口/持久Schema。

原包2.99实际供应最终被runtime_selection_invalid“所选运行程序不可执行”拒绝：src/modules/runtime/program_selection.rs:459把script也传给paths::validate_executable；Windows该函数只收exe/com/cmd/bat，因此原官方bin/ocx.mjs被拒绝。该共享文件不在W1-C临时写集，本批未越权修改、未重命名/改写官方脚本、未放宽任意程序准入。建议Mac单写把Node/Bun原生可执行准入与“交给已核验Node的脚本”普通文件/摘要/边界检查区分；共享最小修复后再做官方--version、自定义H config get port --json和裸命令解析。上述两项官方命令当前没有通过，完整W1-C不得标accepted。

### MSI/NSIS接线、证据与清理

非升级NSIS在删除任何程序文件之前无条件调用精确ocxd.exe --uninstall-terminal-cli --confirm，非0终止并保留程序/配置；升级不调用。静默失败MessageBox加/SD避免无界等用户。MSI使用deferred/当前用户impersonation/Return=check的原生类型1058动作，精确引用[INSTALLDIR]ocxd.exe，条件Installed AND REMOVE="ALL" AND NOT UPGRADINGPRODUCTCODE，序号3499在RemoveFiles3500之前。实际MSVC产品/helper随新模板编译MSI/NSIS exit0；本地包使用test identifier/upgradeCode、dev产品/禁WebView安装，非release候选。MSI真实数据库核对动作/命令/顺序，原生MsiEvaluateCondition的fresh install/normal uninstall/upgrade/repair/partial removal五格通过。MSI完整卸载执行仍未验，不拿表/条件等同真实删除生命周期通过。

不可覆盖证据：.adg/evidence/0.2.24/windows-dev/w1c-msvc-20261113/（编译/环境/原始失败/安装器真实输出/单项回归）及w1c-native-20261113/（139份原文件及SHA256、out01..04/source/stdout/stderr/10格pending与恢复/失败私有后像/输入制品摘要）。gitattributes禁止换行转换。原始失败Profile私有内容通过backup语义复制归档，未修权限/接管owner；Win32 DeleteProfile返回87后通过精确新SID的Win32_UserProfile删除。最终本轮ocxw1c账户/Profile均零残留、临时桌面ACL还原，所有原测试进程退出，隔离账户PATH最终精确还原；不宣称全机进程清洁。

自动审批拒绝了生成schema/误建空目录清理，以及归档后五个精确W1-C生成根的清理，均返回blocked by policy、命令未执行；没有换工具绕过。本轮Public输入/构建/两份本地包等合计26869205653字节仍保留，清单见generated-before-cleanup.json/cleanup-blocked.json。旧W1约2GiB产物/旧失败目录继续保持原事实。未跟踪windows-schema.json和误建空$input/powershell亦列清理未完，不提交为产品改动。

本批回交为组件结果与明确未完，不是W1-C完整通过：共享.mjs准入、PowerShell特殊参数、新终端裸命令解析、真实MSI卸载生命周期仍未完。后继按真实diff再给定向输入；未改官方2.99/日常PATH，未跑W2/双用户、旧r5或全安装器sweep，未发布。

## 2026-11-13至14 W1-D：原生入口/schema3与消费者回交（MSI策略阻塞，非全通过）

已fetch并合入main0d231aebc、docs/governance-main29f0f4dfe，按TEAM revision8/windows-w1d-native-entry-r1执行。日期跨过上海时区午夜：本轮证据目录ID仍为windows-dev/w1d-native-entry-20261113，实际子进程/控制器时间完整保留。代码codex/024-windows-native：3627d7eba（原生入口/schema3）、6a1902498（安装器拥有的程序及来源清单）、b476fa933c35e08b4ca5f58a3f03e370d445ee8e（schema3拒绝边界/NSIS增量）；均已push。独立docs首批d3e06ef01保存编译及三轮真实MSI安装失败，最终本条及其证据继续沿docs/024-windows-dev推送。

### 实现与来源边界

Windows CLI路径改为实际I/ocx.exe，PATH增量为I。ocx.rs/Cargo bin已在共同基线装配，本线复用该独立std控制台bin：直接同目录ocxd.exe --ocxd-runtime-cli，无Shell重解析，继承流/cwd/OS环境及原退出码。Mac入口和schema1未改。原子化事务为schema3，登记绑定Profile/D/H/I/manager、commandPath及entryKind=installer-native-exe，只发布PATH/登记before-after；不创建/复制/覆盖/删除EXE、不把4.5MiB helper塞进1MiB JSON，不兼容/迁移开发schema2。原生文件句柄在一次动作内保留，命令/console摘要、卷/文件ID、无reparse/普通文件/单链接和当前安装来源在预览及边界重核；命令正常存在不等于已登记，未登记的shim仍拒绝转交。

安装器新增明确I/ocx.exe、ocxd.exe组件与ocx-native-entry.json；NSIS明确复制/删除这些安装器自有文件，WiX固定组件及HKCU keypath。来源清单由本线打包脚本从本次真实构建的ocx/ocxd最终字节生成，不写官方runtime/current或版本清单；Desktop主EXE会被Tauri按包类型改标记，故该清单不拿未转换的Desktop摘要误挡安装态。动作核当前清单和保留句柄；长期登记不固定旧EXE哈希，正常动作间程序换代/文件ID变化不误阻止升级，未决pending遇来源变化仍拒绝。该清单是当前安装来源，不冒充签名认证或防同所有者任意改写的信任证书。真实release集成者应从最终签名后的sidecar字节生成来源清单，再打包；本轮仅dev/no-sign，本线未改共享workflow。

MSVC复用14.44.35207/SDK10.0.26100/rustc1.100已验证工具链与原缓存，产品bins+lib check/build、native测试no-run通过，shim参数纯单测1项及Windows cli_entry纯路径1项通过。实际dev Desktop/ocxd/ocx随MSI/NSIS两种模板编译通过，使用独立OpenCodeX W1D Proof产品/upgrade身份，不覆盖日常安装。三个程序输入缺席会在准备阶段/模板编译失败；用户PATH未由安装器直接登记。保留原有warnings（产品38、最终测试42），不冒严格Clippy/release验收。相对共同基线净改仅合同允许的runtime CLI/program_layout/两份Windows tests/安装器及test/windows024-native；Cargo bin早已在基线，不额外改根清单/锁、Vue或program_selection。

### 完整状态表

| 独立项 | 结果 | 真实范围/证据 |
| --- | --- | --- |
| MSVC产品check/build、测试编译 | 通过 | check01/build01/test-build05；dev，非固定候选。 |
| 原生shim/Windows命令位置纯测 | 通过 | shim-unit、path-unit各1项；未跑全量。 |
| schema3显式启停/幂等/命令归属 | 通过 | out04，PATH为实际I，EXE在停用后仍在且摘要不变。 |
| 默认PowerShell混合/空/Unicode/引号/特殊参数 | 通过 | PowerShell7.6.9，默认PSNativeCommandArgumentPassing=Windows，逐项argv、二进制stdin、stdout/stderr及exit23；未改模式。 |
| cmd相同参数与stdio/exit23 | 通过 | out04真实裸ocx.exe→产品ocxd→选中Node/合成probe.cjs，不再用.cmd转交。 |
| 新终端裸ocx解析/停用后缺席 | 通过 | userenv按当前测试token重新生成环境，真实Get-Command/where与实际运行指向I/ocx.exe；不拼接父进程PATH冒充新终端。 |
| 同版本同H/Node/script来源 | 通过 | 合成9.0.0参数夹具和原官方2.99分开记录；OS Profile/HOME与实例H明确分离。 |
| 原官方2.99 --version | 通过 | out04 exit0，原bin/ocx.mjs、同current真实Node，未改扩展名或官方内容。 |
| 自定义H config get port --json | 通过 | out04 exit0返回18241，config.json前后逐字不变、完整版本树验证通过。 |
| Enable/Disable×五边界跨进程继续 | 10格通过 | 原pending留存后新进程同动作继续；schema3不含EXE原字节。 |
| 入口/PATH/登记/pending外部变化 | 4类拒绝通过 | 原外部字节保留。入口重建导致fileID改变时旧pending仍拒绝，测试仅清自己注入的pending，不把它伪作产品重拍基线。 |
| 动作内入口pin/动作间程序换代 | 通过 | 活跃句柄拒绝rename；动作间相同来源EXE以新fileID替换后登记不变、可转交/停用；不是跨版本MSI升级验收。 |
| schema2、已有等价PATH、通知中断 | 3项通过 | out05：旧schema拒绝不迁移，原PATH精确保留，明确通知失败注入保留pending并继续；非真实OS广播故障声明。 |
| NSIS原宏受影响增量 | 5格通过 | 未启用、pendingEnable拒绝保留、升级pending保留、升级active登记/PATH保留、非升级撤销；精确原宏无删除探针，EXE保留，不当完整NSIS卸载。 |
| 真实普通账户MSI安装 | 阻塞/失败 | out03 exit1625，机器DisableMsi=1：“Non-assigned apps are disabled for non-admin users”。 |
| 真实MSI正常/失败卸载保留 | 未执行 | 安装前提被系统策略拒绝，不能用组件helper/条件或另一个安装器替代通过。 |
| 本轮账户/Profile/PATH/临时桌面ACL | 已清理 | 五次顺序单账户开发轮次均删除账户/Profile；两次有效组件轮次PATH原值精确还原，ACL恢复；不是双SID矩阵。 |
| 本轮文件生成物 | 清理受阻 | 两个新根及小型unit文件保留，见cleanup-inventory/generated-cleanup-blocked；旧拒绝项不重试。 |

### 首错、失败保留与实际执行来源

out01 MSI测试命令没有生成安装日志、Windows Installer窗口未返回；只观察到窗口标题，未保存UI内容，不把“帮助窗”当已取证事实。经命令/父PID/创建时间核对，仅停止该自有探针PID115876，原退出-1；随后修正PROPERTY值引号并移除空尾属性。out02为1619，原日志1314显示MSI把输入的正斜杠源路径错误解析；仅将MSI参数转换为Windows分隔符。out03到达真实系统门：exit1625，日志确认LUA/per-user包不需提升，但本机DisableMsi=1禁止非管理员未指派应用；未改策略、未将测试账户提升为管理员、不继续相同源重试。

为记录不依赖MSI的独立终端事实，out04明确使用原编译EXE及当前来源清单直接构造私有测试根，不执行任何安装器、不登记产品，不将此视为绕过MSI通过。所有参数/新终端/PATH/恢复/官方消费者/assert均通过；out04最终componentAccepted=true、allAccepted=false、msiUninstall=false。out05只补新schema反例和原NSIS宏增量，没有重复官方供应/相同MSI尝试。官方版本/端口输出与原程序树来自真实2.99，未以合成版本替代。未发业务同步/模型请求或服务Start。

运行从合入HEAD326b914a9加当时工作源码开始；out01..05精确执行源码、完整stdout/stderr/控制器时间及源摘要归档。产品bin本轮未再次改字节，安装器来源清单/真实包摘要可核对；out04旧测试EXE在重建覆盖前没有单独保存摘要，保留其执行源码/编译日志，绝不补造丢失的原binary摘要。final-executed-inputs为实际保留的最终out05测试EXE摘要。最终b476fa933额外只含out05实际执行的测试/账户选择，生产代码仍为已编译验证的3627d7eba/6a1902498。

独立原证据目录.adg/evidence/0.2.24/windows-dev/w1d-native-entry-20261113/：先写hypothesis；编译/真实dev打包/三轮MSI失败已在docs d3e06ef01推送，后续attempts/out04/out05、component-raw-manifest、最终source/inputs、完整status-table/final-result和清理记录作为第二批。全部raw按SHA256逐字核对，gitattributes禁止换行转换。未覆盖W1-C原out04或既有清理拒绝。

### 清理与未完

所有本轮ocxw1d账户及Win32_UserProfile为零残留，控制器原子保存退出、PATH还原、桌面ACL恢复；除首轮明确终止的MSI探针外，本轮测试/子进程均完成原退出等待；不宣称全机无其它进程。新Public根874693587字节、w1d-bundle01根1069796702字节及unit小文件仍在；归档核验后对两个精确新根的删除被自动审批以blocked by policy拒绝，命令未执行，未换工具绕过。旧W1/W1-C缓存、schema生成物及原失败根保持原拒绝事实；本轮复用旧MSVC/cache，没有重复制或重新清空它们。

W1-D当前组件已收口，但allAccepted=false。唯一请求范围内的验收缺口是本机系统策略阻止的真实MSI正常/失败卸载；需要允许该per-user MSI的合规测试环境后按已保存源码/包SHA定向执行，不能要求用户放宽日常机器策略来凑通过。release流水线仍由Mac/集成线配置最终sidecar来源清单生成；本线只交Windows模板及真实dev编译证据。未修改官方2.99或日常PATH/配置，未跑W2、双SID、旧r5或全量sweep，未发布。
