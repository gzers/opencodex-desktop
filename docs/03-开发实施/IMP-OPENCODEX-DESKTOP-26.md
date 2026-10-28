# 0.2.17 Windows安装可靠性与全链验收

2026-10-28，Asia/Shanghai。状态：in_progress。用户已明确授权WP-89～95文档治理、开发、测试、发布，目标至stable。

基线v0.2.16/d2c08dd7，代码沿ocx-022-model-selector/codex/0.2.17-windows-install，治理在docs/governance-main；并行对话/树/WebSocket原型和协作内容不纳入本轮。

范围与验收归[DMD AC-WIN-017-01～06](../01-需求管理/需求/DMD-OPENCODEX-DESKTOP-MANAGER.md)，当前清单沿[WP89～95](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.16-bugs.md)。先修路径/安装事务及探针，再工具发现/文档/能力说明，再全量与Windows实际流程；不增加新的核对页、保存计划或通知入口。

安装前只读验证路径与目标归属；新包在临时区校验，旧包和入口保留至最终激活与来源记录提交。落位或后续失败有保护地恢复，回滚失败明确保留材料。进程内补偿不宣称突然断电双文件原子性。探针有界零退出/版本/超时，不忽略官方真实错误。

版本管理器只读受控路径/环境声明，有界扫描版本目录，不执行shell配置或盲扫PATH。许可与第三方声明纳入真实bundle资源；WindowsCLI仍禁用，只修“不存在IPC”的过时说明。

WP95以清单逐项审查原生覆盖：隔离Windows runner实际安装/故障、默认值、令牌、文件读回、NSIS/MSI和更新包；host Win10/11、混合DPI、PAC企业证书等不能由单一runner证明，保留未执行边界及可重复检查，不能用mock自称全域通过。不给用户日常配置或活跃Codex强制重启，不发付费生成请求。

检查名固定为review:windows-path、review:install-transaction、review:version-probe、review:tool-discovery、review:packaged-docs-cli、review:native-windows、review:regression-release。完整本地门禁、双平台CI和实际安装器/签名/匿名stable完成并回写后关闭。

## 实施与当前验证

首轮实现57578f895之后，dcc7c96b4补齐了来源登记、入口和包体同一安装事务，真实零退出/64KiB/超时与后代清理的版本探针、扩展路径分隔符、版本管理器有界发现、资源许可实际内容；22f34c82加入正式旧版MSI/NSIS经生产stable更新入口的OTA验收脚本。所有实现仅在代码分支；原型未新增样式与页面。

本地最终前端987通过、10既有跳过，clippy/typecheck/build通过；后端完整串行结果及21项原型/浏览器/真实文件门禁在证据目录，不将macOS结果当作Windows原生结果。第一轮Windows CI的扩展路径rename错误123已保留原日志，按真实原因修正混合分隔符；增强版CI继续执行，尚未确认正式包/OTA或stable。

原生验收使用隔离临时目录、真实Node/npm与官方固定版本包，含联网升级、热缓存离线重装、实际Windows锁回滚、默认/恢复默认向导及普通账户令牌。OTA在0.2.17成为stable后执行真实旧版更新与重开，不以签名文件检查代替应用替换。发布后的验证失败须记录并处理，不能提前关闭目标。

Windows 10/11物理设备、混合DPI、真实SMB/企业代理PAC、企业证书等保留环境边界；runner结果只声明实际执行的范围。历史审计与旧版通过结论不改写为本版全域通过。

## 原生反馈与向导补强

Windows增强回归暴露工具发现夹具受runner的真实工具声明影响，以及“根进程退出、后代仍持有输出管道”的平台差异；保留CI40613095767失败证据。受控发现保持显式版本管理器声明优先、用户常规目录优先于系统回退；专用HOME夹具显式注入空的宿主声明，真实默认环境由原生安装验收覆盖。探针场景分别报告超量输出与继承管道，不混淆为同一种故障。

复核WP95真实向导链路还发现“允许安装脚本并重试”恒禁用：之前按钮使用的allowScripts只在自身click里置true。8ba830d61改用后端稳定scripts_required原因允许明确点击，其余权限/文件等错误不获脚本授权；回归使用实际数字14错误封装。原型结构与样式未变，未自动执行脚本。

原生startup新增可选ManagedInstall：只在临时Actions runner向沙箱home/.local/bin复制固定Node、写入指向已知npm-cli.js的包装入口；真实Vue向导以自身默认路径安装官方2.91.0，必要时明确点击脚本重试，最终读回来源和版本。其余普通账户/安装器smoke不重复联网安装。真实npm后端升级与离线重装继续独立验证。

## 真实官方包离线重装反馈

CI40614790973的Windows后端及原生凭据通过；真实官方2.90.0→2.91.0安装/升级断言完成后，热缓存离线重装返回ENOENT/-4058。不是随机环境通过，也没有跳过离线项。源码发现SystemNpmRunner始终把version拼为包名@version，而fill_offline传入的是绝对tgz路径，导致Windows扩展路径被拼进registry spec。b9fcc1b60把经过验证的本地归档作为独立npm参数，并对npm侧作安全路径简化；registry版本保持原有语义。

Windows真实cmd参数回归补充本地tgz单参数检查；官方live增加阶段日志和失败分支的旧包sentinel、入口、来源原字节检查。共用runtime102项、clippy通过；最新Windows全链CI重新执行，未打tag、未公开0.2.17、未关闭WP95。证据见ci83-official-live-failure.log、ci83-native-progress.json及本轮RCP证据目录。

## 发布前最终本地结果

离线参数修复后的完整后端42套件1289通过、0失败、14既有忽略，证据backend-offline-final.log与summary；前端988通过、10既有跳过，21项门禁0失败/0跳过。最后e83ed669c仅把系统Node回退放到用户版本目录之后，发现专项6项通过，UI源码与已验门禁一致。

CI40617557310的Windows后端完整回归和原生凭据通过，真实官方包联网/离线继续运行；最终e83ed669c对应CI40618515412仍在执行。尚未合并PR26、打tag或公开版本，不将运行中当作成功。必须保留最终EXE、普通账户、正式安装器、实际OTA和匿名stable的独立回执。

## 普通账户失败证据及测试环境修正

CI40617557310完整后端、原生凭据、真实官方安装/升级/离线和最终EXE的真实Vue向导安装通过；普通账户失败，未当作发布通过。原始stderr显示Tauri setup在resolve app data directory返回unknown path，并非安装事务或模型保存错误。新建标准账户令牌为elevated=false/elevationType=1，但第一次Profile目录没有完成创建。

1347e2b7测试脚本在该临时账户下用Windows SHGetKnownFolderPath(KF_FLAG_CREATE)完成系统Profile/Roaming/Local目录创建，验证都在此账户Profile内，再沿实际返回值设置进程环境；不重定向Known Folders、不放宽系统ACL、不提权。之后重跑相同普通账户最终EXE/业务检查。原始失败与真包/向导通过回执在native-b9，最终提交CI继续执行。

d61915ae另补Windows原生混合分隔符、点段/设备名/ADS拒绝回归：安装准入前规范Windows存储路径并先词法拒绝点段，保留系统路径与重解析点限制。共用路径7项及clippy通过；Windows实际结果由最终CI确认。

## 最终CI普通账户反馈

CI40621630300的后端、真实包安装/离线和最终EXE界面通过；普通账户Profile创建仍失败，脚本明确返回E_ACCESSDENIED，未声称普通权限通过。原因进一步定位为Start-Process -Credential可能加载了目标账户注册Profile，却仍继承runneradmin进程环境；新账户Known Folder里的环境展开先指向原用户目录。

a305f8d8/d7abdc42仅修验收脚本：先按当前真实SID从Windows ProfileList只读取得注册Profile，检查为本次ocxacc临时账户且目录存在，再设置进程级USERPROFILE/HOMEDRIVE/HOMEPATH和AppData环境后调用Known Folder API。未改注册表、未重定向系统Known Folders、未放宽系统ACL。小型CI前置执行ProfileOnly验证，不把它当作实际应用通过；最终EXE普通账户业务验收继续独立执行。CI40625421002待完成，尚未合并/发布。

## 普通账户前置环境通过

CI40627338213（7683c569a）前置windows-smoke-evidence已通过。原始日志ordinary-profile-preflight-passed.log明确记录Native Profile、current SID及Profile/Roaming/Local同属新账户，并记录imageLocalAppDataCorrected=true。

最终验收准备方式以7683c569a为准，覆盖前文先前失败的脚本尝试：Windows Profile服务创建本次ocxacc临时账户Profile；若测试镜像Default User克隆的LocalAppData仍是runneradmin绝对路径，只在本次SID已加载的User Shell Folders/Shell Folders用户配置项初始化该账户自己的标准Local目录，再启动新的同账户进程用Windows Known Folder API读回。无全局注册表/日常账户/系统ACL更改，但这确实是临时测试Profile准备，不能表述为“完全未调整Known Folders”的机器环境验收。最终EXE普通账户仍待完整CI，不由前置通过替代。

## 原型证据人工复核范围

列表截图人工复核：实现表格间距、单行顶部状态与无修改保存禁用符合本轮保持原结构要求；file URL原型截图有部分文字未正常显示，但DOM列头/操作/排序入口检查通过。因此parity-list通过仅声明这些结构项，不将该截图当作像素完全一致的证明。本轮没有改列表/预览/通知布局，只改安装脚本确认按钮条件；完整浏览器行为和材质门禁另有独立报告。

## 普通账户最终EXE暴露的实际启动错误

普通Profile前置通过后，CI40627338213仍在最终EXE普通账户setup阶段返回tray icon error Access denied。源码追溯为Windows托盘颜色刷新set_icon的Shell_NotifyIcon(NIM_MODIFY)错误经问号传播，导致整个应用setup失败。07854ea7在已有托盘对象存在时将这项可选外观刷新错误记录到运行日志，保留默认图标；下次window_appearance刷新仍调用refresh，State尚未成功不会缓存“已完成”。不关闭业务/窗口/令牌检查，不把托盘完整跨账户能力声称已通过。

GUI测试环境还限定给刚创建的SID窗口站和输入桌面访问，保留并在finally恢复两者原DACL；不修改其它用户/系统目录ACL，应用令牌仍为标准用户。这一准备步骤前置执行，报告须读取实际恢复成功字段。失败日志ordinary-tray-artwork-failure.log保留。

## 最终托盘修复提交身份

07854ea7的首轮代码误将原有record_startup_event调用替换为普通窗口日志；CI clippy的dead_code错误捕获到这一误改。f77327aa8已恢复原调用，最终对lib.rs的差异只包含托盘外观错误记录而不阻断setup；本地最终clippy通过，构建来源日志保留。CI40632048037继续完整验收；前置环境和前端/发布工具已通过，后端进行中。未将失败CI当通过，也未提前发布。

GUI环境实际准备以bb33b5328为准：对当前新建SID授予窗口站与输入桌面所需GUI访问，finally恢复原DACL；ProfileOnly前置也执行该准备与恢复。ordinary-summary的desktopAclRestoredOnExit先为false，仅Dispose实际成功后回写true。普通账户业务仍须独立通过。
