# MSI 两项卸载专项入口

当前入口只完成 MSVC 编译和保护条件检查，真实事务尚未执行。适用 TEAM revision10、windows-w1d-receive-msi-followup-r1；不是 W2 或发布候选。

需要一台明确准备好、允许普通用户 per-user MSI 的独立 Windows x64 测试机，PowerShell 7 和提升控制器用于创建一个临时普通账户/Profile。测试令牌始终不提升。EZIO-MINI 被硬拒绝；策略键缺席不能证明安装许可。此入口不创建 VM、不改 Installer/机器策略。

在该测试机准备可读输入目录，包含原开发包重命名的 `w1d.msi`、真实 `node.exe`、最小原生 `native024-fixture.exe` 和已编译的产品 lib 测试 EXE。MSI 必须为 316784640 字节、SHA256 `9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282`，转移前后核摘要。另保留本仓库控制器和 `test/smoke/windows-ordinary-startup.ps1`（仅提取桌面 ACL helper，不运行该场景）。测试 EXE 发现的完整名称须为 `modules::runtime::terminal_cli::terminal_cli_windows::native_tests::isolated_msi_uninstall_only`。

配置示例中的机器名和绝对路径必须换成实际已授权的隔离环境；output 使用新的独立目录，所有输入/控制器/配置路径需让临时普通账户可读取：

```json
{
  "preparedPerUserMsiGuest": true,
  "msiOnly": true,
  "componentOnly": false,
  "expectedGuestComputer": "OCX-MSI-GUEST",
  "testName": "isolated_msi_uninstall_only",
  "input": "C:/ocx-proof/input",
  "output": "C:/ocx-proof/out01",
  "test": "C:/ocx-proof/input/opencodex_desktop_lib.exe",
  "desktopSource": "C:/ocx-proof/code/test/smoke/windows-ordinary-startup.ps1"
}
```

仅在环境已交付且策略前提明确后，由提升的 PowerShell 7 控制器执行：

```powershell
./test/windows024-native/invoke-msi-only.ps1 -Config C:/ocx-proof/config.json -ExpectedGuestComputer OCX-MSI-GUEST
```

控制器先只读核包身份和策略，再创建一个普通账户。真实安装若失败，停止两项依赖卸载；不切换组件模式。用例以合成运行镜像构造 pending Enable，真实 /x 必须 1603 且 cleanup action 在 RemoveFiles 前拒绝；独立核程序身份/字节、产品登记、PATH、pending/config 和 D/H 哨兵保留。显式继续原 Enable 后，真实 /x 必须成功，撤销终端登记/PATH、删除安装器拥有程序，保留 D/H 和 live runtime。不会运行官方脚本或模型/同步。

原始 MSI/stdout/stderr、前后像、退出码和账户清理回执保留在 output。20 分钟控制器超时保持。任何未决/未知产品登记均保留账户/Profile供原生恢复，不自动卸载或擦登记；未完成清理将使控制器失败。只有登记为 absent 才允许删除精确测试身份。归档后清理该次自有输入 ACL/文件残留并记录；历史审批拒绝不得重试或换工具绕过。

## 本机临时管理员场景（用户2026-11-14追加授权）

用户允许在原物理主机使用新建的专用临时管理员账户和真实独立 Profile 补测，不使用已有 Administrator 的配置。该场景独立记账，不替代普通用户验收。原普通账户入口和保护保持；新增 `isolated_msi_uninstall_administrator` 复用同样真实 MSI 断言，另核 per-user unmanaged 登记且无 machine 登记。包、安装范围、断言和超时保持；不改机器/Installer/UAC策略。

该场景配置使用相同 input/output/test 路径，改为 `testName=isolated_msi_uninstall_administrator`、`testAdministrator=true`、`authorizedLocalAdministratorMsi=true`、`msiOnly=true`、`componentOnly=false`，expectedGuestComputer填写本机实际名字。提升控制器执行 `invoke-msi-local-admin.ps1 -Config <绝对路径> -ExpectedComputer <本机名>`。

控制器创建一个新管理员账户，使用带该账户登录凭据的临时计划任务以 Highest 启动测试并核实际 SID、提升令牌和 Profile；凭据不落盘、不输出。任务限制20分钟；完成后删除任务，只有产品登记明确缺席才删除测试账户/Profile。日常用户/机器 PATH 前后仅保存摘要核对，不覆盖外部变化。管理员场景无须修改交互桌面ACL；原输入ACL恢复。失败日志和未清项保持可追溯。

## 离线接收复核

`verify-msi-evidence.py` 固定核对贡献代码 `1c6a0881c` 与证据 `b0e7c6d6a` 的 Git 对象。它逐份核验134个证据文件、执行源与最终源码摘要、真实MSI脚本顺序/退出码、原SID继续及前后像，同时保留首次失败、普通用户未验和清理未完。接收者先fetch两个贡献分支，然后在代码工作树运行：

```powershell
python test/windows024-native/verify-msi-evidence.py --code-repo . --docs-repo <文档工作树绝对路径> --self-test
```

本机可额外传 `--package <原MSI路径>`、`--test-executable <out02测试EXE路径>`，重新核本体摘要；省略时输出明确仅复核归档的摘要记录，不能声称已重算远端大包。`--self-test` 在内存中验证原字节损坏、先删文件、保留内容变化、pending残留、丢失原失败、令牌变化、清理失败被隐藏、冒报全通过和路径逃逸等反例；不创建测试身份或临时目录。

该脚本只读本地Git对象和可选二进制，不fetch、不执行安装器、不修改PATH/账户/Profile或策略。exit0表示归档证明内部一致且与固定Git源相符；输出仍为 `ordinaryUserAcceptance=false`、`cleanupComplete=false`、`allAccepted=false`，不能作为集成接收回执、当前机器清理成功或整版验收通过。
