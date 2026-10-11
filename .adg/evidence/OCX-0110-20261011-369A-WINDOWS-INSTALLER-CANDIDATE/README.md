# 369a Windows 安装候选：独立核验与 runner 回执

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182，版本 0.1.10，源码 main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27。[Windows 候选 38127059287](https://github.com/gzers/opencodex-desktop/actions/runs/38127059287) 于 2026-10-11 17:17:48（UTC+8）completed / success。该 SHA 仅触发一次；锁定依赖回归、MSI / NSIS 构建、隔离 runner 安装 / 首屏 / 卸载及上传成功。本目录证明候选资产身份和所附 runner 检查，不构成真机或发布通过证书。

## 下载与独立核验

两个 ZIP 的本地大小、SHA256 与 GitHub API 相符，ZIP CRC 通过。候选 ZIP 只有两个安装器和一份回执；两份 artifact 中的 installer-verification.json 字节一致，1970 bytes，SHA256 为 3144d002a110caca807595c2dd9ec7cb6c50e68de4d0d84f1e820a61b1ff4857。安装器本地重算大小、摘要与 runner 回执一致；源码、版本和 WebView 首屏回执均匹配。详见 independent-verification.json。

| 资产 | bytes | 本地 SHA256 |
|---|---:|---|
| 候选 ZIP，artifact 11696829726 | 7561927 | 77d4ca63098b18a93589761edbe639c6808fd1dc253e61d805c4b5e6513cc002 |
| 证据 ZIP，artifact 11696909705 | 1685851 | bae9aa0b6b1264d2c9df9d5f8fb1cec032ffc9a52f3b56559418a75337f68a5b |
| OpenCodeX Desktop_0.1.10_x64_en-US.msi | 4538368 | f149f01f3096fc7aafa8bc574358ab11fd6c77479db28d822ee414a57fed2fc3 |
| OpenCodeX Desktop_0.1.10_x64-setup.exe | 3231540 | c5a3b9dc9ed87e486933a64ff4402fb47c5d1eb9ecca46fcad5e090de6b3dd0f |

候选下载入口是上述 run 的 Artifacts 中 opencodex-desktop-0.1.10-windows-x64-candidate，预计 2026-10-25 17:17（UTC+8）到期。安装器及大 ZIP 留于 /private/tmp/ocx-0110-369a-windows-3sb0v4xp；临时本地路径不是用户下载入口。本机未执行 Windows 安装器。仓库只留精选回执、截图和日志，未复制 315 项证据中的 WebView 缓存或偏好载荷。download-verification.json 是紧凑视图，完整下载条目报告原字节另存 gzip。

## runner 报告的结果

MSI / NSIS 均安装 exit 0、最终 EXE 首屏 pass、卸载 exit 0，卸载后 EXE 不存在。30 秒观察中窗口响应、主 UI 已挂载，关于信息 0.1.10 / Windows，独立沙箱数据根已初始化。未发现 OpenCodex 为准备态，CLI 禁用。runner win25-vs2026，image 20260925.250.1；完整工具版本见 environment.json。

| 安装后 EXE | runner SHA256 | 核对 |
|---|---|---|
| MSI | ffe370ede6d7566afb63c632dba924024455549c0c778658eb83907480919741 | expectedInstalledExecutableSha256 一致；MSI marker |
| NSIS | cfbfe134fed9ab0f05db84fb8c1260723aff9f7aa8050fcf8627598fd7a74df8 | expectedInstalledExecutableSha256 一致；NSS marker |

打包前 EXE SHA256 为 49a207a606b933159b9ea9206803c082311834010f9ca0f2a85a68f2d5387dc2；Tauri bundle marker 位于 offset 7984421。artifact 未附安装后 EXE，上表只由 runner 回执证明，本机未独立提取安装后 EXE 重算。

## 验收边界和发现

- 两张 main-window.png 由 CDP Page.captureScreenshot 捕获 WebView，最新截图仍可见左侧透明区域黑底、部分文字对比度低。待真机整窗 / CDP 对照判断原生合成与捕获的差异，根因未确认；不据图判云母、原生标题栏或完整外观通过。
- startup-diagnostics 的 effectiveAdministrator / tokenElevated / tokenElevationType 来自 PowerShell harness 当前 token，不是目标应用进程 token。不能代替普通 / 管理员应用双权限验收。
- runner 使用 sandbox 根、结束强杀进程树；不代替新装默认安装目录 data、旧根 / 自定义根优先、不可写无静默回退、正常退出 / 锁释放或物理 DPI。renderer device-scale-factor=1 不是物理 DPI 矩阵。
- codeSigned / releaseAccepted 均 false。真机原生玻璃 / 动效 / 下拉框、IPC、配对性能、真实更新生命周期与 Authenticode 仍待验；不继承旧 SHA 候选结果。

PNG 及 JSON / 日志原始 gzip 保留 artifact 原字节；JSON 可读视图重新序列化为 UTF-8 / LF，原始大小与哈希以 provenance 的 rawBytes / rawSha256 为准。日志可读视图解码后去除行尾空格和末尾空行，原始与视图摘要见 log-provenance.json；API 原件、下载原报告及选定文件来源见 source-provenance.json。checksums.json 覆盖其余文件。IMP / TASK 保持 in_progress，公开发布及 stable 晋升未授权。
