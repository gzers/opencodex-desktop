# Windows终端共享事务与W1-C原生接续

revision: 1；2026-11-13；归IMP-34 P24-03/W1。代码main **83b27e9ae7d0919f57298e38f5f52fd887d50650**，完整源摘要见.adg/evidence/0.2.24/windows-terminal-transaction-r1/integrated-source.json。本批是共享实现与隔离验证，不是Windows原生或固定候选验收。

## 本批行为

Windows设置的同一个runtime_terminal_preview/apply入口消费真实user_profile/D/H/I，显式确认后创建I/runtime/bin/ocx.cmd并登记当前用户HKCU/Environment/Path增量。命令位置使用实际安装I，支持自定义安装位置；不写全局npm、HKLM或其它用户PATH，不修改原官方2.99。运行包没有预置这个用户入口，它由明确启用动作创建；不能将其说成只读使用安装器已有命令。

ocx.cmd关闭delayed expansion，按自身位置转交I/ocxd.exe --ocxd-runtime-cli及原参数；控制台专用模式在IPC命令解析前执行，按当前OS用户的固定manager-location取D/H，校验登记/入口，再运行同一current选择中的官方Node与脚本。Windows显式HOME/USERPROFILE仍取该OS用户目录，OPENCODEX_HOME取实例H。GUI入口已有的提前dispatch保留。真实cmd/PowerShell转义、stdio、等待与退出码必须由W1-C在Windows验证，当前不冒用Mac exec证据。

私有D/manager-state/terminal-cli.json与terminal-cli.pending.json保持同一产品语义；Windows是明确schema2平台记录，Mac的schema1不变，不读取旧目录或迁移。公开IPC只返回动作、位置、当前版本、冲突和确认摘要，不返回PATH原内容、登记原件或密钥。UI没有Shell文件时显示“当前用户PATH”。

PATH策略保留REG_SZ/REG_EXPAND_SZ原类型及UTF-16原字节/缺失，拒绝错误类型/超长/非法编码/终止或引号歧义。只为查重比较斜线/大小写/尾分隔符，不清洗或重写原PATH。已存在等价单条目录不取得其删除归属；重复或不确定输入拒绝。新目录如含分号以完整引号表达；REG_EXPAND_SZ下含百分号、无法保持字面含义的目录拒绝。停用时，原后像未变则逐字还原原值；用户后来改变其它条目时只删除原确切新增片段，保留其它字节，原片段被改动/重复/删除或类型变化则拒绝。没有对不合作外部注册表写者宣称系统原子CAS。

## 中断及来源约束

每次执行由既有terminal-cli动作文件锁串行，先重新建立相同确认，再create-only持久pending。启用先命令、再PATH、后登记；停用先PATH、再命令、后登记。pending固定全部before/after，每次publication前后检查文件和原生PATH只处于原两像，写后读回、环境通知及最终确认后才删除pending。已有后像不重复写；失败不补偿覆盖外部新值。只允许显式预览后继续原动作，不自动切换方向或重拍基线。

启用完整准入current镜像及console helper摘要，确认绑定该来源；动作内保留所选程序，同次完整镜像结束后重核当前选择、helper、文件、PATH和pending。新测试在该读回之后修改4种当前事实，均拒绝并保留外部字节。schema/上下文/脚本/PathEdit均由原逻辑重建，不执行pending中的任意文本。

Windows文件读回额外使用W0 PrivateRecord核当前TokenUser受保护ACL/原文件与单链接；D/H通过原生private pin，不修补已有权限。Mac测试用相同事务引擎、合成程序和Memory Registry，只证明共享状态机，不能证明Win32 ACL/真实终端或广播整链。开发期间原编译/格式失败留存；早期未保存原stderr的工具输出只列观察，不伪造原日志。

## 卸载接续

新增精确内部命令 **ocxd.exe --uninstall-terminal-cli --confirm**：未完整确认或额外参数拒绝。独立读回本用户实际绑定，只停用已登记入口；未完成Enable需先回应用完成，不在卸载中擅自完成Enable。从未打开、固定anchor和命令均明确缺席时只读返回，不创建D/H、锁或注册表键。来源未知保持失败。

数据清理在删除前先检查terminal登记/pending，尚有终端归属则拒绝删除，避免先删D中的恢复凭据而留下外部PATH。完整程序卸载入口当前仍受原Mac-only完整进程门保护；本批没有为Windows放开未经证明的生命周期/删除门。

**安装器hook尚未消费新helper。** W1-C须在真正非升级卸载、删除ocxd.exe之前，无论是否保留用户数据都调用它；非0/未就绪拒绝继续并保留程序/配置/原登记。升级模式不撤销正常终端登记。MSI/NSIS仅改当前安装器明确责任范围，不自动执行旧卸载器、递归清runtime或放宽原安全门。新helper及hook要一起定向验证后才能宣称卸载撤销完成。

## W1-C写集与Git交接

用户要求到需要Windows时停下给提示词。本机完成本批提交推送后停止，不并行改下列共享CLI文件；原Windows线接续到明确Git回交为止：

- 既有Windows独占：infrastructure/windows_cli024.rs、windows_official_runtime024/**、test/windows024-native/**、windows/installer-hooks.nsh、nsis023.nsi、wix023.wxs。
- 临时单写，仅修本批Windows编译/CLI转交/中断问题：src/modules/runtime/terminal_cli.rs、terminal_cli_windows.rs、terminal_cli_path.rs；src/bin/ocxd.rs；src/commands/terminal_cli.rs。可新增同目录terminal_cli_windows_native_tests.rs并注册Windows原生测试。不要修改Mac实现语义、其它共享状态机或全部Vue。
- 证据仍只写docs/024-windows-dev的windows-dev-handoff.md及新的windows-dev/<不可覆盖轮次>/**；IMP/TEAM和正式接收由本机维护。

先MSVC原生全产品check/build，逐个记录实际SHA/源摘要；本机交叉编译在ring依赖缺Windows SDK的assert.h失败，尚未编译到新产品Windows分支，不能据此判产品Rust通过/失败。然后在专用隔离测试账户运行真实cmd/PowerShell的新终端解析、空格/Unicode/引号及特殊参数、stdin/out/err/退出23、原包--version和自定义H的config get port --json；不启动真实业务服务/模型/同步。当前用户PATH生产操作只能在测试账户，日常账户只读；每次原值/新值私下精确读回并最终还原，不输出完整日常PATH。

覆盖启用/停用/无操作取消、五边界pending继续、错误确认/外部文件/PATH冲突、通知失败保留、安装器新helper的未启用/正常停用/未完成Enable拒绝/升级保留。按实际差异只做受影响定向原生验证，不跑W2、双SID或全安装器sweep。清理测试账户/Profile/进程/私有键及本次产物，证据先归档核对；原W1的自动审批清理拒绝/约2GiB产物保留原事实，不绕过自动拒绝。分批Git推送代码/原证据、完整状态/首错/未完；不修改官方包、不发布。

## 本机证据范围

见.adg/evidence/0.2.24/windows-terminal-transaction-r1/。最终terminal定向包含共享Windows policy/transaction及原Mac回归；原需真实二进制的2项ignored未偷偷改为通过。数据卸载13项、console IPC解析2项、前端11项/类型检查及严格Clippy/fmt单独记录。没有Windows服务、注册表写入、真实同步、模型请求、日常Shell或安装App变更。工程编译cache保留，隔离测试根逐字归档后清理。整版、W1完整接线及W2仍未完成。
