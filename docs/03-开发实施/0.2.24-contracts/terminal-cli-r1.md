# 0.2.24 终端 ocx 转交与登记

revision: 1；2026-11-11，代码 maine9b00583cfe5a84386f13897f5313c210b94f2d4。归属 IMP-34 P24-03；本批为共享转交和Mac用户入口，不表示Windows CLI/PATH适配或整版验收完成。

## 对用户的行为

设置 → 安装配置 → 终端命令 →「检查 ocx 命令」。先预览命令位置、转交入口、实际版本、配置HOME及将修改的用户Shell文件；明确确认后启用或停用。取消不写入；同名命令冲突显示来源并拒绝覆盖。新终端识别 ocx。它不使用现有“CLI控制面”的开关，后者仍是 ocxd 到运行中管理器的本机IPC。

Mac命令路径 ~/.local/bin/ocx，转交到当前用户程序根 runtime/bin/ocx，再调用实际 Desktop 可执行文件 --ocxd-runtime-cli。该标志在 main 的Tauri装配之前处理；只读读取当前用户新版manager-location和当前D/H，核对自有登记/命令原字节，核对current及镜像，再执行实际Node/官方入口。原参数、标准输入输出及退出码沿Unix exec转交；HOME/USERPROFILE不被替换，OPENCODEX_HOME固定为当前绑定的实例HOME，OCX_NO_DESKTOP_HANDOFF=1避免重新交回桌面。服务/更新命令仍由官方原包执行，Desktop不改其源码。

## 共享接口和文件归属

| 接口 | 输入/输出 |
| --- | --- |
| runtime_terminal_preview | 无页面路径输入。后端取真实profile/D/H/I/executable与环境；返回action、enabled、pending、version、commandPath、runtimeEntry、opencodexHome、profilePaths、conflicts、confirmationDigest |
| runtime_terminal_apply | 只接受action、confirmationDigest，拒绝额外字段；输出同action、enabled、commandPath与HOME并由前端再核对 |
| terminal_cli::Context | 明确的user_profile、desktop_root、opencodex_home、program_root、manager_executable；平台解析及实际持久绑定必须一致 |
| terminal_cli::dispatch | 正式二进制早期识别专用标志，转交前拒绝未完成入口事务、漂移绑定、改动程序或未知选择 |

D/manager-state/terminal-cli.json登记应用拥有的命令和精确PATH块；terminal-cli.pending.json保留本次原件/后像。两者与Shell原件可能含用户内容，只保存在私有文件，不通过IPC或日志输出。中断继续只允许原动作且必须重新预览，严格重建命令文本和标记块增删，不能把pending中的任意脚本或路径当授权。事务最终核对所有目标和原pending后才删除pending。

Mac只自动编辑当前用户的zsh .zshrc或bash实际login文件（.bash_profile/.bash_login/.profile）及.bashrc。保留原字节与现有权限；停用只去掉登记的块，后来用户添加的命令保留必要换行。由本动作新建、停用后空的文件恢复为不存在。链接/不明归属/异常写权限/过大或非UTF8文件拒绝；未支持Shell和重定向ZDOTDIR不自动猜测。原PATH/常见命令位置有别的ocx时不覆盖；新Shell中的标记块也在发现后来加入的同名命令时保留原PATH。

配置位置变更前必须先停用已启用或未完成的终端入口，避免登记和外部命令遗留在旧D。该准入和已有存储冻结/实例锁共同执行，不自动迁移旧目录。程序更新后稳定bridge仍调用同一Desktop，转交每次读取新current；Mac应用移动后先停用原登记再启用新路径。新终端命令位置及Shell写集仅属当前用户，不安装全局npm、不修改系统PATH。

## Windows接线边界

当前命令层在Windows明确拒绝注册，模块内Mac路径不是Windows实现。W1需使用已有独占windows_cli024写集实现真实用户注册表PATH/命令归属与撤销；不要复制Mac .local/bin/Shell写者。输入仍应区分真实user_profile和opencodex_home，转交必须消费同一版本选择和实际HOME。Windows GUI子系统的stdout/退出等待、cmd字面转义及原生ACL需按正式可执行构建验证，不能拿Mac exec证明。Windows和版本化卸载联动合同仍待冻结，当前不发布最终W1/W2验收指令。

## 证据

.adg/evidence/0.2.24/terminal-cli-r1/：18项隔离后端回归，41项前端/类型检查，全目标严格Clippy及格式检查。显式二进制测试2项：真实Desktop不启GUI，转交合成程序验证参数/HOME/退出23；真实Desktop转交官方2.99的 --version 和 config get port --json，从自定义集合根/HOME读到41234，配置和程序身份不变，停用后自有命令与新建Shell文件移除。全部真实二进制摘要、固定官方包摘要和网络拒绝沙箱保留。

浏览器实际共享组件的1040×760及580×480预览、启用/停用结果、冲突拒绝通过；截图里“用户 PATH 配置”折行后已缩短标签为“PATH 配置”，后继行为未变。实际原生测试为当前debug二进制置于隔离.app形状目录，非签名发布安装器；原包/原生候选最后验收仍需另行完成。测试仅写独立profile、配置、程序与临时目录，日常Shell/配置/Keychain与网络拒绝，无业务服务或模型请求。测试残留清理，证据留存。
