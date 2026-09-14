# 项目报告

## 定位

面向评审的**单页 HTML 报告**：一个长页，左侧固定目录负责章节跳转，滚动时自动高亮当前位置。

- 派生产物，**不作为事实源**；有冲突时以 DMD `revision 3` 与 `docs/02-项目核心` 为准。
- 不构成开工授权；实施契约仍由 IMP 冻结。

## 生成方式

正文用 Markdown 写，跑一个脚本生成 HTML。**改内容改 `.md`，不要手改 `index.html`**（它是构建产物，下次构建会被覆盖）。

```
需求分析报告.md  ──build.py──▶  index.html（+ report-metadata.json）
                                    ▲
assets/shared-style.css             │
assets/side-nav.css                 │
assets/side-nav.js  ────────────────┘
```

```bash
cd 01-需求分析/输出/需求分析报告
python3 build.py
```

- 依赖：Python-Markdown（`python3 -m markdown`）。
- 侧栏目录不是手写的：脚本从正文的 `## NN · 标题 {: #anchor }` 自动收集，标题或锚点改了目录跟着变，不会和正文脱节。
- 标题、品牌名、页脚取自 Markdown 文件顶部的 front matter（`title` / `brand` / `brand_sub` / `footer`）。
- 构建会写出 `report-metadata.json`：记录源文件 sha256、生成时间、章节列表与 REQ / AC / WP 数量，方便核对这次 HTML 是从哪一版正文来的。

## 文件

| 文件 | 用途 |
|---|---|
| `需求分析报告.md` | **正文源文件**，14 个章节，改内容改这里 |
| `build.py` | 构建脚本：Markdown → `index.html` + `report-metadata.json` |
| `index.html` | 构建产物，报告本体（首页即全文，14 个章节），不要手改 |
| `report-metadata.json` | 构建产物，记录源文件指纹与章节索引 |
| `assets/shared-style.css` | 样式方案，取自已确认的参考项目 HTML 报告（`guoguocorp.com` 的 `html-report/shared-style.css`），原样使用 |
| `assets/side-nav.css` | 左侧固定目录的布局补充，沿用同一套颜色 token、圆角与选中胶囊 |
| `assets/side-nav.js` | 滚动高亮；脚本不可用时锚点跳转仍可用 |

## 章节

01 概览 · 02 这是什么 · 03 解决什么问题 · 04 做与不做 · 05 能力地图 · 06 需求清单 · 07 架构 · 08 状态模型 · 09 关键流程 · 10 数据与安全 · 11 原型与设计 · 12 进度与治理 · 13 开工清单 · 14 依据索引

## 覆盖

- 需求：`REQ-01` ~ `REQ-27` 全部出现。
- 验收：`AC-01` ~ `AC-13` 全部出现。
- 工作包：`WP-01` ~ `WP-22` 全部出现，且附「需求覆盖核对」表。

## 打开与维护

- 用浏览器直接打开 `index.html`。图片以相对路径引用本协作包内的 PNG（`../架构与流程图/导出图片/`、`../../05-原型/文档/截图/现行/`），单独拷走 html 会看不到图。
- 窄屏（≤1024px）时左侧目录自动收成顶部横向条。
- 改正文：改 `需求分析报告.md`，再跑 `python3 build.py`。
- 改样式：优先改 `assets/side-nav.css`；`assets/shared-style.css` 保持与参考方案一致，不单独改动。
