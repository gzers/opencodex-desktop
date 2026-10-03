# wkprobe · WKWebView 几何探针

真实渲染层（WKWebView，与应用内 WebView 同引擎）几何取证工具，用于复现 Blink/无头 Chromium 查不出的排版差异
（例：`grid + justify-items:center` 下网格项 fit-content 宽度在 WebKit 里定位偏移）。

- `wkprobe <dist 目录> [hash] [副标题文案] [宽] [高]`：加载 `<dist>/index.html`（软件 `apps/desktop/ui/dist`）。
- `wkprobe2 <根目录> [html 相对路径] [hash] [副标题文案] [宽] [高]`：加载任意原型 html（相对路径在 `../` 里解析同源脚本/样式）。
- 两者都会在加载 4s 后把 `.motion-caption` 文案改写为给定串（用于复现「环境准备」态长副标题），
  读取 `.motion-fixed` / `.motion-mark` / `.motion-identity` / `.motion-mainline` / `.motion-caption` 的中心并打印 JSON。

构建：`test/smoke/wkprobe/build.sh`（需 Xcode 命令行工具）。产物在 `test/smoke/bin/`，不入库。

用法示例（概览「环境准备」态，IMP-08 回归）：

```sh
./test/smoke/bin/wkprobe "$PWD/apps/desktop/ui/dist" "#overview" "启动条件未满足 · 请按下方指引处理" 1180 760
PROTO="docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/01-需求分析/05-原型"
./test/smoke/bin/wkprobe2 "$PWD/$PROTO" "/候选/2026-09-28-Logo本体形变/overview.html" "#overview" "启动条件未满足 · 请按下方指引处理" 1180 760
```
