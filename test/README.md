# 本机验收测试目录

这里放**本机真机验收**用的工具与产物（桌面壳的 WebView 冒烟、截图、OCR 点击驱动）。
它不参与前端/后端构建，也不进入发布制品。

## 目录

| 路径 | 内容 | 是否入库 |
| --- | --- | --- |
| `smoke/` | 冒烟工具（截图、OCR、按文字点击、窗口/锁屏探测） | 是（脚本与源码） |
| `smoke/bin/` | `build.sh` 编译出的工具二进制 | 否（gitignore） |
| `software/` | 被测试的 `.app` 副本 | 否（gitignore） |
| `runtime/` | 隔离 HOME（测试用数据根、钥匙串等） | 否（gitignore） |
| `out/` | 截图与中间产物 | 否（gitignore） |

## 为什么放这里

桌面壳是 WKWebView，只有真机（真实窗口、真实合成）才能验证视觉与交互；
`/tmp` 会在重启后被清空，所以工具与产物放在本仓 `test/` 下、并只把**大件/生成物**gitignore。

## 前置

- macOS，已安装 Xcode 命令行工具（`swiftc`）
- Python 3 + Pillow（`python3 -c "import PIL"`）
- 已构建的桌面壳：`apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app`

## 用法

```sh
# 1) 编译冒烟工具（输出到 test/smoke/bin/）
test/smoke/build.sh

# 2) 看当前会话是否锁屏（锁屏时 WKWebView 不合成内容，截图会变白）
test/smoke/bin/lids

# 3) 列出应用窗口（编号 / 是否在屏 / 边界）
test/smoke/bin/wins "OpenCodeX"

# 4) 前台化并把应用窗口截到 test/out/
test/smoke/fgh.sh test/out/overview.png

# 5) 识别截图里的文字（像素坐标，原点左上）
test/smoke/bin/ocr test/out/overview.png

# 6) 按文字点击（先刷新截图再点）
test/smoke/winclick.py "设置" test/out/_refresh.png
test/smoke/sect.py "WebDAV 同步" test/out/sync.png
```

> 锁屏状态下所有截图/点击都无效：`bin/lids` 的 `CGSSessionScreenIsLocked` 为 1 时，
> macOS 不为 WKWebView 合成内容，截到的画面会发白或发虚（边缘标准差会从 ~30 掉到 ~8）。
> `run-smoke.sh` 与 `winclick.py`/`sect.py` 已内置守卫，锁屏时会直接报 BLOCKED，不会产出假证据。


### 0.2.0 Windows真实WebView模型切片

windows-startup.ps1向windows-webview.mjs传入其本次新建的显式sandbox根时，附加执行windows-model-ui.mjs。先比对immutable currentDataRoot/activeDataRoot及runtimeActive，拒绝相对路径/其他目录，再通过真实Tauri服务建立专属模版，使用正式DOM编辑/保存和版本默认确认，检查旧版本不可变与渠道策略不变；仅模版文件导出/接收/采用沿同一IPC，不接运行服务或生成。生成model-ui.json和截图归当前Windows启动证据。未提供精确根的旧appearance/native调用继续原首屏流程，不自动执行写测试。脚本单测只证明准入边界与CDP读取合同，真实Windows结果必须等对应CI/制品实际运行后登记。

Windows普通CI将具有2秒生产截止的official_version_source三项冷进程检查先运行，再执行排除该已完成分组的其余workspace回归；每项仍执行一次，保持默认Rust工作线程、原2秒截止、4秒清理边界及子进程/无关控制句柄断言，无预热、重试或放行失败。首组失败立即返回失败。原ETW诊断分支仍保留完整首次回归，不以重放替代。该调度调整的实际有效性由新源Windows结果确认。

### Windows 本机托管安装与失败恢复

`smoke/windows-install-recovery.ps1` 接受 `-Executable`、`-Node`、`-NpmCli` 和一个尚不存在的 `-OutputDirectory`；协议驱动需要 Node 24，可通过 `-DriverNode` 与被测 Node 分开指定。脚本创建专属沙箱与带中文、空格的工具和数据目录，使用真实 WebView2、正式 Vue 事件处理和生产 Tauri IPC，执行官方 2.91.0 安装、缺失版本失败、旧包/入口/来源读回及 2.93.0 恢复升级。输出截图、命令日志和机器回执，结束时只终止本次启动的应用进程树，不替换日常安装。

`windows_runtime` 中显式忽略的 `windows_local_discovered_tools_install_upgrade_and_failure_recovery` 用本机实际发现的 Node/npm 和用户环境，安装写入尚不存在的绝对路径 `OPENCODEX_LOCAL_INSTALL_EVIDENCE`。它额外用冷缓存验证离线 npm install 的 ENOTCACHED 失败保持旧状态。仅在明确需要联网真包验证时运行：

```powershell
$env:OPENCODEX_LOCAL_INSTALL_EVIDENCE='D:\OpenCodeX Tests\Native New Root'
cargo test --locked --test windows_runtime windows_local_discovered_tools_install_upgrade_and_failure_recovery -- --exact --ignored --nocapture
```

Cargo 命令工作目录为 `apps/desktop/tauri`。报告只证明该次 Node/npm、源代码与测试目录组合；不能倒推历史退出码 1 的原因，也不覆盖真实 SMB、企业代理或所有 Windows 版本。
