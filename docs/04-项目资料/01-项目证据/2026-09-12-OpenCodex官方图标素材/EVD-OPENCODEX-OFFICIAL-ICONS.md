---
id: EVD-OPENCODEX-OFFICIAL-ICONS
title: "OpenCodex 官方图标素材"
object_kind: evidence.material
state: verified
summary: "保存并核对 @bitkyc08/opencodex 官方 Logo、favicon 与托盘图标，供桌面管理器视觉资产参考。"
source_refs:
  - DMD-OPENCODEX-DESKTOP-MANAGER
  - COL-LOCAL-20260912-01
---

# EVD-OPENCODEX-OFFICIAL-ICONS OpenCodex 官方图标素材

## 支撑对象

| 对象 | 用途 |
|---|---|
| `DMD-OPENCODEX-DESKTOP-MANAGER` | 桌面管理器候选视觉参考。 |
| `COL-LOCAL-20260912-01` | 官方生态与桌面壳方案调研支撑。 |

## 结论

OpenCodex 官方仓库 `lidge-jun/opencodex` 提供多用途图标资产，可作为桌面壳视觉资产参考；当前素材不等于本项目品牌资产，也不授权代码实施。

## 来源与核对

| 项目 | 内容 |
|---|---|
| 依赖包 | `@bitkyc08/opencodex` |
| 本机版本 | `2.50.0` |
| 官方仓库 | `https://github.com/lidge-jun/opencodex` |
| 文档站点 | `https://lidge-jun.github.io/opencodex/` |
| 许可证 | MIT（随素材附 `LICENSE`） |
| 核对日期 | 2026-09-12 |

## 素材清单与校验

| 文件 | 官方来源 | 规格核对 | SHA-256 |
|---|---|---|---|
| `logo-light.png` | `assets/logo-light.png` | 512×512 PNG | `b61a0e71a69dea391cec9f88c54bc7b2f360eb12ac46bd4127a75329ef1de51b` |
| `logo-dark.png` | `assets/logo-dark.png` | 512×512 PNG | `33d8e0753ca43f1d598e49800a2a9386346469ec32edf3e93f254160e2942132` |
| `favicon.png` | `gui/public/favicon.png` | 128×128 PNG | `58f18856402f1576f5daffaec0e86d6c1dae7c244a1203a474d9973ca7a4b787` |
| `favicon.ico` | `docs-site/public/favicon.ico` | 多尺寸 ICO | `f4e970dbaa33447b3cd886a13c94ab6e2c51b09427c2247afc86e58e96460bb8` |
| `opencodex-tray.png` | `src/tray/assets/opencodex-tray.png` | 512×512 PNG | `de6aafa7d193eecf0ee8905e0230417afcdd11971a2272e4cf184676fd7ba90d` |

## 适用边界与回写

- 可用于官方来源核对、视觉参考和桌面图标生成实验。
- 不得声称是本项目的原创品牌，不得在对外发布时移除或替代 MIT 许可声明。
- 若未来实现桌面壳，衍生图标规格、命名与生成脚本应回写后续 IMP；本 EVD 不授权实施。
- 官方资产后续更新时，应重新下载、重新计算校验并更新本文件；历史版本可按需替换。

## 保留策略

长期保留；素材体积小、具备官方来源核对价值。若上游许可或资产发生变更，以更新后的本 EVD 为准。

## 派生 macOS 图标（2026-09-12）

`generated-macos/` 目录保存基于官方素材派生的 macOS 规范草稿：

| 文件 | 说明 |
|---|---|
| `AppIcon-light-1024.png` | 亮色 macOS App 图标画布，浅灰渐变底 + 深色官方 logo |
| `AppIcon-dark-1024.png` | 暗色 macOS App 图标画布，深蓝黑渐变底 + 浅色官方 logo |
| `AppIcon-light.icns` / `AppIcon-dark.icns` | macOS `.icns` 全尺寸导出 |
| `tray-template.png` / `tray-template@2x.png` | macOS 菜单栏 Template Image，18/36pt，黑色单色 + alpha |
| `AppIcon-light-1024-512.png` / `AppIcon-dark-1024-512.png` | 预览图，非最终资产 |

派生规则：保留官方轮廓与透明背景；App 图标使用 1024px 画布、圆角 185/1024、图形尺寸 660/1024；托盘使用 macOS Template Image 语义。当前是本地派生预览，尚未进入实施资产或授权代码实施。
