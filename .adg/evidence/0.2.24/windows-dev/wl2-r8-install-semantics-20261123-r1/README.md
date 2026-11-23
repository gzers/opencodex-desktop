# 0.2.24 安装/卸载语义实测（2026-11-23）

候选来自源码 `dffb3b1d64745f815476e77fc12103d3d34d90e4`（tree `9832faf40673aacc7b7818408b5a0baedacd98b8`），MSI `6d4ca083…`、NSIS `94900554…`。全部在 `EZIO-MINI\marenben`（SID ……1003，session 7，非管理员，普通令牌）自己的交互桌面上执行。

## NSIS：真实安装与正常卸载通过

`/S` 静默安装到 `%LOCALAPPDATA%\Programs\OpenCodeX Desktop`，退出码 0，耗时 21,332 ms；`uninstall.exe /S` 退出码 0，耗时 1,221 ms。安装后卸载登记出现 `OpenCodeX Desktop 0.2.24`，`InstallLocation` 指向该 per-user 目录；卸载后登记消失。

落地文件与包内成员、构建输入三方一致：`opencodex-desktop.exe` `2eb01bc4…`、`ocx.exe` `3ddfadf7…`、`ocxd.exe` `04b9ad6d…`、`ocx-native-entry.json` `c12dc54d…`（315 字节）。`runtime-bundle` 四个成员的字节数与固定输入相符，`resources` 下为 `LICENSE`、`THIRD-PARTY-NOTICES.txt`。

卸载后程序目录不存在且无残留文件，卸载登记、开始菜单、桌面快捷方式都已消失；用户 PATH 在安装前后与卸载前后都是同一个原值，说明本安装器不写 PATH。首错为 null。

卸载后仍留下 `HKCU\Software\gz ers\OpenCodeX Desktop`。这是模板行为：删除该键的那条语句只在「同时删除应用数据」被勾选时执行，而 `installer-hooks.nsh` 的非升级路径会把该勾选强制置 0，避免更新或卸载误删当前与旧数据。它记录的是上一次安装位置，供下次安装预填。

`ocx-native-entry.json` 的摘要与我在 Linux 侧按同样内容重建的结果不同，是因为构建脚本在 Windows 上用 CRLF 写这个文件；按 CRLF 重算即得 `c12dc54d…`，与安装落地值和包内成员值一致，不是内容漂移。

## MSI：被机器策略拒绝，与包无关

同一 MSI 在 `marenben`（session 7）和 15119 两个普通账户上都返回 **1625**，日志为 `Machine policy value 'DisableMsi' is 1` 与 `Rejecting product '{A4D717F0-…}': Non-assigned apps are disabled for non-admin users.`。

包本身是 per-user 的：摘要信息 `Template = x64;0`（不需要提升），日志也写着 `MSI_LUA: Package is marked as LUA installation capable with no elevation required`。同一份日志里 SOFTWARE RESTRICTION POLICY 判定该包 `permitted to run at the 'unrestricted' authorization level`，所以未签名只是提示，不是拒绝原因。

生效的策略值对普通令牌不可见：`HKLM` 与 `HKCU` 的 `Policies\Microsoft\Windows\Installer` 都不存在，`CurrentVersion\Policies\Installer` 在 32/64 视图下都不存在，`reg query /s /f DisableMsi` 在 `HKLM\SOFTWARE\Policies` 下 0 命中；机器在 WORKGROUP、无 AppLocker `SrpV2`、无 SRP 规则（仅 `authenticodeenabled=0`），本地 `Registry.pol` 只有 Terminal Services 设置。

因此 MSI 的真机安装/卸载在本机无法验收：需要管理员改动机器策略，或换一台允许普通用户 per-user MSI 的主机。两者都超出本轮授权，未执行。

## 未做

两包仍未签名；未跑 W2 双 SID；未发布；未改机器策略、日常配置或官方 2.99。


