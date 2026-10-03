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
