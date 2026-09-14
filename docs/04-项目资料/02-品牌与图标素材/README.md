# OpenCodex 品牌与图标素材

本目录是 OpenCodex Desktop 的 Logo、应用图标、托盘图标、Favicon 和平台交付素材的长期管理入口。

## 目录原则

- 原始来源、设计基准、平台交付和归档物分开保存。
- 设计基准确认前，候选 SVG 不得进入应用代码、安装包或发布物。
- 平台交付物必须从已确认的设计基准生成，不直接从原始素材生成。
- 每个平台目录只保存该平台实际需要的格式，不混放预览图和源文件。
- 不用文件名表达状态；状态、来源、规格和校验信息统一记录在本 README 或对应清单中。

## 公开分发说明（2026-09-14）

- `01-原始素材/codex-official.svg` 为**官方来源标识**，已移出版本控制、仅本地保留（见根 `.gitignore`）。
  理由：本项目声明不复制或重新分发 OpenCodex 官方源代码、构建产物、图标、界面素材与商标标识。
- `01-原始素材/LICENSE` 为上游素材的**来源许可**（MIT，Copyright (c) 2026 opencodex contributors），作为**溯源信息**保留在本仓库。
- 本节以下内容描述的目录结构仍包含该文件，因为本 README 记录的是**本地素材工作区**；克隆后的仓库中该文件缺席属预期。

## 当前阶段（2026-09-12）

平台交付物已从设计基准生成，覆盖 macOS 和 Windows 的 AppIcon、Tray、Logo 与 Favicon。完整素材清单（规格 + SHA-256）见下方"素材清单"。

```text
01-原始素材/
├── LICENSE
├── codex-official.svg
└── aggregation-source.svg

02-设计基准/
├── opencodex-desktop-routing.svg
├── opencodex-desktop-aggregation-knockout.svg
└── previews/
    ├── preview-light-{16,32,64,128,512,1024}.png
    └── preview-dark-{16,32,64,128,512,1024}.png

03-平台交付/
├── macOS/
│   ├── AppIcon/
│   │   ├── AppIcon.svg
│   │   ├── AppIcon.icns
│   │   └── png/{16,32,128,256,512,1024}.png
│   └── Tray/
│       ├── tray-template.svg
│       ├── tray-template.png
│       ├── tray-template@2x.png
│       ├── tray-white.svg
│       ├── tray-white.png
│       └── tray-white@2x.png
└── Windows/
    ├── AppIcon/
    │   ├── AppIcon.ico
    │   ├── AppIcon.svg
    │   └── png/{16,24,32,48,256,512}.png
    └── Tray/
        ├── tray-dark.svg
        ├── tray-dark.ico
        ├── tray-dark@2x.png
        ├── tray-light.svg
        ├── tray-light.ico
        └── tray-light@2x.png

04-品牌组件/
├── Logo/
│   ├── logo.svg
│   ├── logo-light.svg
│   └── logo-dark.svg
└── Favicon/
    ├── favicon.svg
    ├── favicon.png
    └── favicon.ico
```

## 完整规划目录

设计基准确认后，按以下结构建立平台素材：

```text
02-品牌与图标素材/
├── README.md
├── 01-原始素材/
│   ├── LICENSE
│   ├── codex-official.svg
│   └── aggregation-source.svg
├── 02-设计基准/
│   ├── opencodex-desktop-routing.svg
│   ├── opencodex-desktop-aggregation-knockout.svg
│   ├── opencodex-desktop-final.svg
│   └── previews/
│       ├── preview-light.png
│       ├── preview-dark.png
│       └── preview-tray-scale.png
├── 03-平台交付/
│   ├── macOS/
│   │   ├── AppIcon/
│   │   │   ├── AppIcon.svg
│   │   │   ├── AppIcon.icns
│   │   │   └── png/
│   │   │       ├── 16.png
│   │   │       ├── 32.png
│   │   │       ├── 128.png
│   │   │       ├── 256.png
│   │   │       ├── 512.png
│   │   │       └── 1024.png
│   │   └── Tray/
│   │       ├── tray-dark.svg
│   │       ├── tray-white.svg
│   │       ├── tray-dark@2x.png
│   │       └── tray-white@2x.png
│   └── Windows/
│       ├── AppIcon/
│       │   ├── AppIcon.svg
│       │   ├── AppIcon.ico
│       │   └── png/
│       │       ├── 16.png
│       │       ├── 24.png
│       │       ├── 32.png
│       │       ├── 48.png
│       │       ├── 256.png
│       │       └── 512.png
│       └── Tray/
│           ├── tray-light.svg
│           ├── tray-dark.svg
│           ├── tray-light.ico
│           └── tray-dark.ico
├── 04-品牌组件/
│   ├── Logo/
│   │   ├── logo.svg
│   │   ├── logo-light.svg
│   │   └── logo-dark.svg
│   ├── Favicon/
│   │   ├── favicon.svg
│   │   ├── favicon.png
│   │   └── favicon.ico
│   └── README.md
└── 05-归档/
    ├── 旧设计基准/
    ├── 旧平台交付/
    └── README.md
```

## 平台交付要求

### macOS AppIcon

- 使用有背景的 macOS 应用图标版本。
- 交付 `AppIcon.icns`，同时保留生成所需的 SVG 和 PNG 源级输出。
- PNG 至少覆盖 `16/32/128/256/512/1024` 像素。
- 不把托盘图标混入 AppIcon 目录。

### Windows AppIcon

- 使用透明背景版本，不添加 macOS 风格底板。
- 交付 `AppIcon.ico`，并保留 SVG 与 PNG 源级输出。
- PNG 至少覆盖 `16/24/32/48/256/512` 像素。

### Windows Tray

- 不使用背景，不使用渐变，不使用阴影。
- 提供纯色亮色版 `tray-light` 和纯色暗色版 `tray-dark`。
- 亮色版用于深色背景，暗色版用于浅色背景。
- 图形必须在 `16px` 和 `20px` 托盘尺寸下仍然清晰。

### macOS Tray

- 使用透明背景的 Template Image 语义。
- 提供纯色暗色版和纯色白色版。
- 交付 SVG 与 `@2x` PNG；如宿主框架需要，再从 PNG 生成对应资源目录。
- 不复用带 App 背景的 macOS AppIcon。

## 品牌组件

- `Logo/` 保存不带平台背景的品牌 Logo 及亮暗版本。
- `Favicon/` 保存网页和文档场景使用的小尺寸图标。
- 品牌组件应从最终确认的设计基准派生，不从历史候选派生。

## 命名规则

- 使用小写英文、短横线和明确平台语义。
- 平台优先写目录，颜色或主题写文件名。
- `light`、`dark` 表示前景颜色语义，不表示画布背景。
- `@2x` 只用于像素密度，不用于普通尺寸标记。
- 预览文件放入 `previews/`，不得与交付文件混放。

## 状态与变更

- `01-原始素材/`：来源素材，长期保留，不直接修改。
- `02-设计基准/`：候选或已确认的 SVG 方案。
- `03-平台交付/`：由确认设计基准生成的可消费文件。
- `04-品牌组件/`：跨平台 Logo 和 Favicon。
- `05-归档/`：被替换但需要保留追溯关系的历史版本。

设计基准确认后，新增最终文件 `opencodex-desktop-final.svg`，再生成各平台交付物；生成日期、输入基准、工具版本和 SHA-256 校验值应记录在后续素材清单中。


## 素材清单（2026-09-12）

生成基准：`02-设计基准/opencodex-desktop-aggregation-knockout.svg`（SHA-256 `bd6e561e73b243b9a999cce066dd329493057bcc07c03f64ae4cae899207c8b8`）  
生成工具：sharp 0.35.4（librsvg 2.62.91）+ macOS `iconutil`  
生成日期：2026-09-12

### 02-设计基准

| 文件 | SHA-256 |
|---|---|
| `opencodex-desktop-routing.svg` | `3ab647bf86a42ddd1f511d04b30eda2a4ccd8a26d55ed050d7b1769ec11c104f` |
| `opencodex-desktop-aggregation-knockout.svg` | `bd6e561e73b243b9a999cce066dd329493057bcc07c03f64ae4cae899207c8b8` |

### 03-平台交付 / macOS AppIcon

| 文件 | 规格 | SHA-256 |
|---|---|---|
| `AppIcon.icns` | 白色圆角底板，主体 75% | `3a89852efb32d8e0169a0dfdbef6efec53dccb47cfedd9b1445dfc802d180357` |
| `AppIcon.svg` | macOS 白色圆角底板 SVG | `6f37da285a35795332d8cff8a382f946a2ff27ef1751e61ead1677066e57a75e` |
| `png/1024.png` | 1024×1024 PNG | `0afa607809efce06ea6e61617b1261f627f8524fa1247b0b0b3aaa3ef963ac99` |
| `png/512.png` | 512×512 PNG | `1048245a2561a9a9e54eeae9584308a225444d7463426686b9a4fb00798c1a43` |
| `png/256.png` | 256×256 PNG | `afed76155c13ff64546ad2e34cf209d63f574b3579cc88589b3bd6263796fac1` |
| `png/128.png` | 128×128 PNG | `359133687664b93d368fa5c9b0da7002cf1dcb3dd58d82d80ff43335153f791d` |
| `png/32.png` | 32×32 PNG | `22dc59c4fb7539e68c71a6b92411132bf94f8843681e48b376e692024cdceaa2` |
| `png/16.png` | 16×16 PNG | `616acd7214b27404db467c99c27fc581f523cf7d857e5ad6d29978bb614f3c6e` |

### 03-平台交付 / Windows AppIcon

| 文件 | 规格 | SHA-256 |
|---|---|---|
| `AppIcon.ico` | 多尺寸 ICO（16/24/32/48/64/128/256） | `9e66134cada279adade3a0f9bc9f0197455ecc9bac32f0d2e4505287d79b075f` |
| `AppIcon.svg` | 透明背景 SVG | 见 `02-设计基准/opencodex-desktop-aggregation-knockout.svg` |
| `png/256.png` | 256×256 PNG | `e341c8074d8f9a06ff9846a3afdf9f3b827d9a8b92edac1929eaee0af4b683b7` |
| `png/512.png` | 512×512 PNG | `3ceab57db1b486da7c526fb10181b61cd9b0ac2b5eec4341fc15dbb6f15c6e92` |

### 03-平台交付 / macOS Tray

| 文件 | 前景色 | SHA-256 |
|---|---|---|
| `tray-template.svg` | `#000000`（Template Image） | `6f0d29447763299eeda4dea5ff493721c7a62906ce3bfeefd638d77291e3122e` |
| `tray-template.png` | 18×18 pt / 1× | `736a0f04075ec2bbfc64e84d760afbc04d1c3ecb6834fafb79175ba146c60c0b` |
| `tray-template@2x.png` | 36×36 px / 2× | `74617f6d0f0ef1deabacd1c048e89c807d7669e292c48591b8a844ffb7ac2351` |
| `tray-white.svg` | `#FFFFFF` | `d9b705416e8daec6c7774915e5435e06adb7c491ed2cdc2bc88ef2258e3602fd` |
| `tray-white.png` | 18×18 pt / 1× | `1cd3a6f2c8255b0656d44f59436f39522755e8e21c06b5805aa6899fefb84a74` |
| `tray-white@2x.png` | 36×36 px / 2× | `1bec7a8f03c3a8c65839a99a55ffd78cd7fdd30af906bac18b465be3e4c79082` |

### 03-平台交付 / Windows Tray

| 文件 | 前景色 | SHA-256 |
|---|---|---|
| `tray-dark.svg` | `#1A2948` | `0aac6ce7eb2fb9e33458c1b38fd777430d3151d891b26598049ab0ff8b064f9c` |
| `tray-dark.ico` | 多尺寸 ICO（16/24/32） | `a8492ef181fcea38fe29a50b67bcb4ae2fbf34e44f906deca3b430452633956c` |
| `tray-dark@2x.png` | 32×32 px / 2× | `d8512c5d6cd5717995ab71d1e66a30f4aa092e3e7c9cbe1a8f907b20b2687d59` |
| `tray-light.svg` | `#FFFFFF` | `d9b705416e8daec6c7774915e5435e06adb7c491ed2cdc2bc88ef2258e3602fd` |
| `tray-light.ico` | 多尺寸 ICO（16/24/32） | `2248c14febc5c55bf725b6e0578fff359be5abca4745ffe92fef0d17d59891a2` |
| `tray-light@2x.png` | 32×32 px / 2× | `67ccd16320d030659ecd880b9a7698ed75008135bc8938be35049c056405e026` |

### 04-品牌组件 / Logo

| 文件 | 说明 | SHA-256 |
|---|---|---|
| `logo.svg` | 原始渐变 | `c546f8d8b3f7e75111e2992f4671729cc55961ff74ddfc5727cc1c5868b2f8c0` |
| `logo-light.svg` | `#FFFFFF`，用于深色底 | `d9b705416e8daec6c7774915e5435e06adb7c491ed2cdc2bc88ef2258e3602fd` |
| `logo-dark.svg` | `#1A2948`，用于浅色底 | `0aac6ce7eb2fb9e33458c1b38fd777430d3151d891b26598049ab0ff8b064f9c` |

### 04-品牌组件 / Favicon

| 文件 | 规格 | SHA-256 |
|---|---|---|
| `favicon.svg` | 透明背景 SVG | `c546f8d8b3f7e75111e2992f4671729cc55961ff74ddfc5727cc1c5868b2f8c0` |
| `favicon.png` | 128×128 PNG | `f96ab822704b6ca841dbb42a6143db1194b623dbca05494a62e09c8fd94d89fe` |
| `favicon.ico` | 多尺寸 ICO（16/32/48/64/128） | `79666d0c671521daef73e4540555e8e5547c9dd758fb3fc6c8d3f5784a7febe9` |



## 素材清单修订（2026-09-12）

本次修订只更新平台交付细节，不改变设计基准。

- 补充 macOS AppIcon 源级 SVG。
- 将 macOS AppIcon 调整为白色圆角底板，彩色主体边界放大到 75%。
- 将 Windows AppIcon、Windows Tray、Favicon ICO 更新为多尺寸 ICO。
- 移除 macOS Tray 未纳入交付目录规划的 512px 中间 PNG。

| 文件 | 规格 | SHA-256 |
|---|---|---|
| `03-平台交付/macOS/AppIcon/AppIcon.svg` | macOS 白色圆角底板 SVG | `6f37da285a35795332d8cff8a382f946a2ff27ef1751e61ead1677066e57a75e` |
| `03-平台交付/macOS/AppIcon/AppIcon.icns` | 白色圆角底板，主体 75% | `3a89852efb32d8e0169a0dfdbef6efec53dccb47cfedd9b1445dfc802d180357` |
| `03-平台交付/Windows/AppIcon/AppIcon.ico` | 多尺寸 ICO（16/24/32/48/64/128/256） | `9e66134cada279adade3a0f9bc9f0197455ecc9bac32f0d2e4505287d79b075f` |
| `03-平台交付/Windows/Tray/tray-dark.ico` | 多尺寸 ICO（16/24/32） | `a8492ef181fcea38fe29a50b67bcb4ae2fbf34e44f906deca3b430452633956c` |
| `03-平台交付/Windows/Tray/tray-light.ico` | 多尺寸 ICO（16/24/32） | `2248c14febc5c55bf725b6e0578fff359be5abca4745ffe92fef0d17d59891a2` |
| `04-品牌组件/Favicon/favicon.ico` | 多尺寸 ICO（16/32/48/64/128） | `79666d0c671521daef73e4540555e8e5547c9dd758fb3fc6c8d3f5784a7febe9` |
