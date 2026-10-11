# 03e0 Windows 安装候选：下载核验与 runner 回执

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。源码 `03e0ad942c16fe2ef75473eb6693012b3ae684ac`，版本 0.1.10。2026-10-11 15:38（UTC+8）单次触发的 [候选 run 38122059291](https://github.com/gzers/opencodex-desktop/actions/runs/38122059291) 已 completed / success，package job 于 15:49:19 完成。本目录只证明同 SHA 候选生成、资产下载完整性及所附临时 runner 安装 smoke，不是 Windows 真机或发布通过证书。

## 独立核验

两个 ZIP 的本地 SHA256 与 GitHub artifact digest 一致。`installer-verification.json` 在候选与证据两份 artifact 中字节完全相同，SHA256 `bd2810039883604c96b0d1c5b0518668710c87d9806b6d2caaf2468aa03a5eab`。MSI / NSIS 本地重算大小、SHA256，与 ZIP 内对应条目及回执相符。API 原始 gzip、可读 JSON 和 source-provenance 也复核一致。结果见 independent-verification.json。

| 资产 | bytes | 本地 SHA256 |
|---|---:|---|
| 候选 ZIP，artifact 11696430158 | 7562238 | `98319a0d2c2b5bdb2c6ec1806d1470820cf006e11d7548422cc494b04af3b005` |
| 证据 ZIP，artifact 11696305214 | 1604535 | `86dc0986d3f76087d9bd5ae2b1223ce5d6a8e2bc76d623f32e10b4e41ce1277d` |
| OpenCodeX Desktop_0.1.10_x64_en-US.msi | 4538368 | `1c803172d7cca9d294c011f40e0b58c1e057f33680b32cc29759f41e649e3527` |
| OpenCodeX Desktop_0.1.10_x64-setup.exe | 3231828 | `c6c0520c37d3957d66132f086c4f0fb85a9ed2df37d7e4b9fe02437781357cde` |

两个 artifact 预计于 2026-10-25 15:49（UTC+8）到期。大 ZIP 和安装器留在 `/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocx-0110-03e0-windows-jlrwq9hs/`；临时本地路径不是用户下载入口。该目录未运行 Windows 安装器。仓库只保存选定回执、日志和截图，未复制 WebView 缓存、偏好载荷或安装器。download-verification.json 保留原下载文件清单，不代表将这些文件全部纳入仓库。

## runner 报告的安装结果

MSI / NSIS 均为安装 exit 0、最终 EXE 首屏 pass、卸载 exit 0，卸载后 EXE 不存在。30 秒首屏观察确认窗口响应、WebView 主 UI 已挂载、关于信息 0.1.10 / Windows、隔离数据根初始化、CLI 关闭；未发现 OpenCodex 时为准备态。环境与源码身份见 environment.json、各 startup / webview / app.log。

| 最终 EXE | runner 回执 SHA256 | 对照 |
|---|---|---|
| MSI 安装后 | `c6e4b492db5c31107a7f1ecd3e3cff1bbeee9007859ec84dc067ec689f41b0a4` | expectedInstalledExecutableSha256 一致，MSI marker |
| NSIS 安装后 | `d9314c1908e883571fe8ed8f9b8cde2b10300a544dd140bd4f9490cf6d811aa0` | expectedInstalledExecutableSha256 一致，NSS marker |

打包前 EXE SHA256 为 `fc7424b146b91b8b03323d0b215615956708bc306f980581921721416e469c43`。Tauri bundle marker 位于 offset 7984293，安装后与打包前哈希不同已有严格预期校验。artifact 未附安装后 EXE；上表是 runner 回执，本机没有提取安装后 EXE 并独立重算。

## 验收边界及发现

- 两张 main-window.png 均来自 `test/smoke/windows-webview.mjs` 的 CDP `Page.captureScreenshot`，只捕获 WebView。图中左侧透明区域呈黑色、部分文字对比度低。可能涉及原生合成或捕获方式，原因未确认；需要真机整窗截图与 CDP 对照，不能据此判云母 / 标题栏完整外观通过，也不根据该图直接判原生渲染根因。
- startup-diagnostics 的 effectiveAdministrator / tokenElevated / tokenElevationType 读取 `WindowsIdentity.GetCurrent()`，属于 PowerShell 测试脚本 token（本轮 elevated true）。不是目标应用 token；不能代替应用普通 / 管理员双权限验收。实际设备应取得目标进程 token。
- 启动使用独立 sandbox 根，脚本最终 `Kill(true)` 清理测试进程树。不能代替新装默认 `<安装目录>\data`、普通退出、锁释放、旧根 / 自定义根优先或不可写目录无静默回退验收。
- codeSigned / releaseAccepted 均 false。物理 DPI、原生玻璃 / 动效、IPC、配对性能、真实更新生命周期和 Authenticode 未通过本轮验收。

`*.raw.gz` 保留日志原始字节（包括 BOM / 空日志），`.log` 为解码后去除行尾空格的可读视图；log-provenance.json 记录编码、大小和双重摘要。JSON 回执 / PNG 保留 artifact 原字节。checksums.json 覆盖目录内其他文件。IMP / TASK 保持 in_progress，公开发布及 stable 晋升未授权。
