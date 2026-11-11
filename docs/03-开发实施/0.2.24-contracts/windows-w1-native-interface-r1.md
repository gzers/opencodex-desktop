# Windows W1可并行原生接口与安装器资源 r1

revision: 1；2026-11-11。归TEAM/IMP-34；本轮放行可独立提交的平台部分，不是Windows最终验收，也不假定Windows已消费。代码基线main 09392b38e09ea919566ccce831dcc95d16e7163b；先fetch并承接该真实基线。W0原user_profile/opencodex_home修正优先并行，不等本批CLI。

## 写集和原生调用边界

继续只写 infrastructure/windows_official_runtime024/**、新增 infrastructure/windows_cli024.rs、test/windows024-native/** 与既定 Windows installer-hooks.nsh / nsis023.nsi / wix023.wxs。Rust共享mod/commands/DTO、全部Vue、.github、版本号与主线合并由Mac单写。原生模块仍可由独立宿主引用真实实现定向测试；若必须引用共享类型或旧文件，先给实际最小接线补丁建议，不复制第二套程序选择/路径/JSON格式。

当前共享事实已具备：program_layout::ProgramLocations / RuntimeVersionPaths；program_selection::read/verify_current；完整program_image；program_catalog登记；program_activation/版本化安装；terminal_cli::Context/Action/Preview/Outcome；program_removal/reinstall/repair及其真实unused需求。用户CLI入口和内部ocx-runtime.cmd分离，不能互相冒用。Mac已经有UI和共同确认操作，Windows不另建UI/IPC。

## W1-A native PATH primitives：可立即做

新增windows_cli024.rs先提供进程内Rust原生原语，供Mac接到terminal_cli现有事务，而不是自己猜第二份terminal-cli.json协议：

    #[derive(Clone, Debug, PartialEq, Eq)]
    pub struct RegistryValue { pub value_type: u32, pub bytes: Vec<u8> }
    pub fn read_user_path() -> std::io::Result<Option<RegistryValue>>;
    pub fn replace_user_path(expected: &Option<RegistryValue>, desired: &Option<RegistryValue>) -> std::io::Result<()>;
    pub fn notify_environment_changed() -> std::io::Result<()>;

生产目标固定当前TokenUser的HKCU/Environment/Path，不写HKLM或调用setx；原类型REG_SZ/REG_EXPAND_SZ和UTF-16原字节/缺失状态必须可读回，拒绝其它类型、超限、奇数长度/错误终止等不明确输入，不自动改成新值。建议上限256KiB。replace持有当前SID命名互斥及句柄，完整比较expected原类型/字节/缺失，再写入并原样读回；删除值只在desired=None明确许可且当前expected相符时做。注册表没有对不合作外部写者的原生CAS保证，不能伪报全系统原子：外部变化/写后不符明确失败，不补偿覆盖外部新值。互斥/句柄不可读时未知拒绝，不能用“注册表命令exit0”当值正确。

notify使用有界WM_SETTINGCHANGE/Environment广播，不发通信消息给他人、不输出完整PATH；它只是本OS会话环境变更通知，不修改系统PATH，也不等于其它已启动终端继承新PATH。API只提供操作事实，PATH去重/分号/大小写/尾斜线/其它工具冲突规则及精确before/after写集由共享适配拥有，不能在native层清洗用户完整PATH。

原生命令引用/退出码/进程输出若需要新增helper，可在同文件提供内部结构并给Mac审阅；用户ocx.cmd最终转交真实当前Desktop --ocxd-runtime-cli，按当前OS profile选D/H，官方Node/script来自同一选中版本。不能把H写死到共享cmd、覆盖已有同名命令、写全局npm/系统HOME。此批先交注册表/广播原语和反例；Mac接现有终端确认/恢复/卸载撤销后再交Windows做受影响native链，当前不要求猜共享事务格式。

验证使用模块内部显式私有测试subkey或隔离测试账户，生产入口仍固定HKCU/Environment，无IPC任意注册表路径。保留/还原测试subkey原状态并删除本次新键，不改日常账户Environment/Path；提供真实API类型/值读回、错误expected、别类型/奇数字节、并发本SID写者、无法广播/权限未知等反例。只声明组件事实，不宣布真实终端最终通过。

## W1-B installer资源与路径：可立即做

固定build输入 .github/runtime-platforms-024.json：官方2.99原包SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c；Windows完整归档资源名 runtime-bundle-0.2.24-win32-x64.tar.gz，原始官方资源 official-runtime.tgz，另runtime-bundle.manifest.json/.sha256。真实平台descriptor由stage_runtime_bundle_024.py产生，不复制Mac descriptor或发明Win摘要。Wix当前仍硬编码0.2.23归档与maintained-runtime.tgz，必须适配这四个实际资源；NSIS用相同生成映射，不能因保持旧文件名就继续供应维护包。

I仍取用户实际选择的exe父目录，默认LOCALAPPDATA/Programs/OpenCodeX Desktop，自定义位置保持，不猜固定盘符；U/D/H取当前用户绑定，程序和配置不能混放。默认每用户独立安装与系统多用户配置隔离保持；升级保留已登记live runtime/回退/操作记录，不能递归删runtime、修权限接管未知目录或从旧安装/旧HOME自动迁移。旧022兼容扫描/迁移特例不作为024新策略继续增加；有必要删除既有PREINSTALL旧兼容路径时按当前无兼容合同说明范围。

仅靠静态template检查不能称MSI/NSIS已通过；先独立编译实际生成资源映射和默认/自定义落点，给真实编译/资源清单/首错及未完。若完整卸载或PATH撤销必须调用尚无共享helper，不自己发明命令填通过，交具体最小建议由Mac接线。已有30/5/10秒、双真实SID/桌面与未知拒绝不变，但本批不跑旧r5或W2全量。

## Git回交

先fetch共同main并承接最新基线，不reset覆盖成果。先交W0 HOME/Profile修正，再按原生PATH组件、安装器资源两主题分批提交；每批及时push代码与独立docs证据，回源SHA/实际执行SHA/环境/命令/完整状态/首错/清理。docs仅windows-dev-handoff.md及不可覆盖windows-dev/<轮次>，不改TEAM/IMP/共享索引。失败证据不覆盖，成功不当整版或候选验收。最终W2只等Mac固定产品候选和摘要后给用户简短提示词，当前不发布。
