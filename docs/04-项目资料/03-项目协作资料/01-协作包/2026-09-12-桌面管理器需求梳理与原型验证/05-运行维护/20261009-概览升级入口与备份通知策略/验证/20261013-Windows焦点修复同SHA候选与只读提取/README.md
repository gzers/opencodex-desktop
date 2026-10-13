# Windows 焦点修复同SHA候选与只读提取

已为焦点特性`be7395acfce0ec1e36b83300b576db408f8c35bb`准备同SHA的NSIS/MSI候选及可供后续原生复验的桌面EXE。当前本机仍运行原eae实例与原24小时周期，尚未启动候选或接受焦点原生门禁。

本轮原件归`.adg/evidence/OCX-0110-20261013-WIN-FOCUS-CANDIDATE/`；60份全集合与字节摘要冻结重验通过。

## 完整CI之后的候选准入

在重新认证核验CI38352525092/be7395a与五必需job均success后，核实远端feature分支仍为be7395a、同SHA不存在候选run，2026-10-13 14:16:40（北京时间）只dispatch一次已有`windows-candidate.yml`，前后分支SHA不变。没有改打包工作流、跳过回归或重跑旧失败run求绿；候选用途是准备原生复验制品，未集成main或作发布决定。

实际候选run38355803684/package115158292064确认同SHA，于14:30:02观察completed/success；完整包锁Windows回归、NSIS/MSI构建及隔离runner的逐个安装、最终EXE首屏、卸载均success。主库505 passed/0 failed/4 ignored；34条cargo结果合计601/0/7 ignored，ignored保留。

## 同run制品与本机只读身份

两个ZIP分别匹配认证API digest，路径、大小、重复项和CRC检查通过；两份installer-verification回执逐字一致，20份选取原件保留，安装包各自摘要再次匹配回执。

| 产物 | SHA256 |
|---|---|
| 11792974377证据ZIP | `747fb1415a5dd41ddaa3e8eb457b6e83b1a1e28872b4020386fd3e598c4089d0` |
| 11792959449安装包ZIP | `2e8639d5921a58499d73edc4ec1a8ee7dcbae1ff2169d3045d042ba5040b7d9f` |
| MSI | `6a500186d89c8a2d73216c5ac7b47b9ab27372938f1ea0048f95c6abc11f5573` |
| NSIS | `09b2526cfd2bc15ff17da753d8ce4bf5d03c71833cd61f31df3abecb4a570e4c` |
| runner启动的MSI桌面EXE/本机只读提取EXE | `3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1` |
| runner启动的NSIS桌面EXE回执 | `c6b5361585bec04bdbc5db525aaa0e16c2193232cab7b97540d15c3ac8362f34` |

已验证的安装包存于代码worktree的`test/software/candidate-be7395a-38355803684/`。使用既有方法以Windows Installer **OpenDatabase模式0**读取MSI，仅导出内嵌cab，再由系统expand解包；没有执行msiexec、安装器、桌面EXE或ocxd。方法及原始读回保留。

桌面EXE位于该目录的`msi-readonly/opencodex-desktop.exe`，实际大小8676864字节、ProductVersion0.1.10，摘要与runner真正启动的MSI文件一致。完整be7395a提交号存在于其字节中；焦点组件及测试blob分别a5d587f4…/3da03dbb…，与已经红绿验证的版本一致。

只在内存将回执记录的MSI补丁位置7987597恢复为UNK，整文件摘要严格匹配候选未打补丁built EXE的`8e2e9b3e003a136b9b7a5599976607b4aee9ced1c7e26ccc845ffefa45825f6a`；再分别计算MSI/NSS期望摘要，两者都匹配runner回执。没有把归一化文件写回磁盘。这个候选built摘要不同于先前CI直接cargo构建的8651ba83…，只证明同源及各自严格身份，不声称两次构建字节相同。

首次离线检查器误假设安装后的MSI标记唯一，实际两个偏移7987543/7987597中后者才是打包补丁位。记录该失败后，按既有严格bundle方法采用回执位置并核验整文件built摘要；产品、制品及产品断言不改，不能把检查器误判当产品回归。

## 保留的边界

MSI实际ALLUSERS1、UpgradeCode仍`{30FC0B19-5C02-5849-A234-D548E7AFC0DE}`，与日常产品共享；本次没有在当前主机安装。改INSTALLDIR或换同机账户不能取得安装隔离。只读提取不抵安装目录data、权限、自定义根、升级或卸载保留门禁。

两份runner startup实际令牌均elevated/type1、mode any/specificModeMatched false，overview挂载、无HOME、沙箱及首屏通过；图经查看为runner WebView的待接入/缺少Node.js页面，没有更新详情交互。不抵本机普通/管理员配对、物理DPI/云母/标题栏/焦点门禁。实际提取EXE的Authenticode为NotSigned，与候选回执codeSigned=false一致；没有签名、发布、晋升通道或改变信任设置。

补充纠正此前工具范围：当时仅搜索PATH和常规Cargo/Rustup目录，漏查已有隔离`test/software/rustfmt-tool/`。本轮14:25:38使用该Rust1.99工具对两份改动Rust文件执行只读`--check`，exit0，PATH随后恢复；不改既有原件，也不计本机完整Cargo构建通过。

原eae应用37276/监督器24300/监测3920的真实保留句柄、创建时间及唯一实例保持，偏好/根绑定/策略/调度四项字节、日常EXE摘要不变。快照188份/0观察失败、原截止2026-10-13 22:55:39/40保持。候选observer10164/exec43728正常观察终态后退出，未替换或拼接原周期。

后续先审阅原周期终态与拥有者cleanup；原实例结束并独立核验本机桌面进程为0后，才在人的前台窗口启动经过身份核验的候选进行焦点等复验。22:50–23:10仍须新人的开始信号，后台工作不取得前台权限。原失败CI原因未隔离，原TASK182/FZ50许可/安全安装与独立Profile、全部未完原生门禁、原性能预算与真实周期/历史failed继续保留。IMP in_progress，无正式RCP/关闭证书，不降低门禁、不关闭或发布。
