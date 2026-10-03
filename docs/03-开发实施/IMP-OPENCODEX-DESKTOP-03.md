---
id: IMP-OPENCODEX-DESKTOP-03
object_kind: implementation.change
state: completed
title: 目录结构规范化改造（src-ui / src-tauri → apps/desktop/{ui, tauri}）
summary: 开源前的目录规范化：把顶层 src-ui/ 与 src-tauri/ 重构为 apps/desktop/ui/ 与 apps/desktop/tauri/，使命名对齐 Folo / spacedrive 的 apps/<产品名>/ 惯例，Rust 外壳目录改用 spacedrive 同款名 tauri；行为、契约与功能不变，仅目录与路径引用变化。
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
task_ids: ["TASK-OPENCODEX-DESKTOP-81"]
completion_summary: "2026-09-27 执行完成（TASK-81）：src-ui/src-tauri 迁移为 apps/desktop/{ui,tauri}；同步全部现行引用（CI、tauri.conf.json、build.rs、源码注释、前端测试相对路径、原型 qa/*.mjs、根 README、docs/02-项目核心）；前端 typecheck/vitest(259)/build、后端 fmt/clippy/test、原型 jsdom(315)/浏览器(415)、仓库根真实 tauri build 出 .app+.dmg 全绿。执行中发现并修正两处：Tauri 构建 hook 的工作目录是 tauri 目录的父目录（apps/desktop），故钩子前缀为 ui 而非 ../ui；后端 project_document 的仓库根改为向上 3 层。"
source_refs: ["DMD-OPENCODEX-DESKTOP-MANAGER", "IMP-OPENCODEX-DESKTOP-01", ".adg/work/three-way-audit/summary.md"]
---

# IMP-OPENCODEX-DESKTOP-03 目录结构规范化改造

- 建立日期：2026-09-27
- 状态：**已执行并通过门禁**（2026-09-27，TASK-81 / RCP-81 / CERT-81）；行为、契约与功能不变，仅目录与路径引用变化。执行结果见文末「执行结果」。
- 来源：2026-09-27 用户决定（开源前把 `src-ui` / `src-tauri` 的命名与分层规范化）。
- 相关：[原实施契约](IMP-OPENCODEX-DESKTOP-01.md)、[质量整改](IMP-OPENCODEX-DESKTOP-02.md)。

## 目标

把顶层结构

```text
opencodex-desktop/{src-ui, src-tauri, ...}
```

重构为

```text
opencodex-desktop/
├─ apps/
│  └─ desktop/
│     ├─ ui/        ← 前端界面（原 src-ui）
│     └─ tauri/     ← Rust 外壳（原 src-tauri）
├─ .github/workflows/ci.yml
├─ docs/            （属 docs/governance-main 分支）
├─ AGENTS.md  README.md  README.en.md  LICENSE  .gitignore
```

**行为、契约与功能不变**；只改目录位置与由此产生的路径引用。

## 依据（已核实，非臆断）

- **Tauri 不要求目录名**：读官方 CLI 源码 `crates/tauri-cli/src/helpers/app_paths.rs`，`resolve_tauri_dir` / `resolve_frontend_dir` 是"从当前目录起、**最多 3 层**内搜索含 `tauri.conf.json` / `package.json` 的目录"，**目录名无关**。故 `tauri/` 合法。
- **惯例佐证**：Folo = `apps/{cli, desktop, landing, mobile, ota, ssr}` + `packages/*`；spacedrive = `apps/{…, tauri, web}` + `crates/*` + `core/` + `packages/*`。→ 单个可运行产品放 `apps/<产品名>/`；Rust 外壳可用 `tauri`（spacedrive 即 `apps/tauri`）。
- **深度余量**：`apps/desktop/ui` 与 `apps/desktop/tauri` 均恰为深度 3，满足 CLI 的 3 层上限；执行时以真实 `tauri build` 复核。

## 非目标（Out of scope）

- 不改功能、数据、状态与契约（`AC-*` / `FZ-*`）；不新增或修改任何字段。
- 不新建 `packages/` / `crates/` / `core/`（当前无共享包，避免空目录；将来需要再加）。
- 不改写历史记录文本（如 `IMP-01` 的按日期变更行、旧协作包记录），只更新**现行入口**与结构性引用。
- 不做发布、签名、公证、平台扩展（Windows 按用户计划在其本机另行处理）。

## 影响面清单（现行引用，须同步修改）

代码 / 配置：

| 文件 | 现状 | 改为 |
|---|---|---|
| `.github/workflows/ci.yml` | `npm --prefix src-ui …`（×4）、`working-directory: src-tauri`、注释 | `--prefix apps/desktop/ui`、`working-directory: apps/desktop/tauri` |
| `src-tauri/tauri.conf.json` | `beforeDevCommand` / `beforeBuildCommand` = `npm --prefix ../src-ui …`、`frontendDist` = `../src-ui/dist` | `../ui …`、`../ui/dist` |
| `src-tauri/build.rs` | `…/../src-ui/dist` | `…/../ui/dist` |
| `src-ui/tests/window-drag.test.ts` | 读 `../../src-tauri/capabilities/default.json` | `../../tauri/capabilities/default.json` |
| `src-ui/tests/page-surface-material.test.ts` | 读 `../../src-tauri/tauri.conf.json` | `../../tauri/tauri.conf.json` |
| 前端若干注释中的 `src-tauri` / `src-ui` 路径字样 | `scale.ts`、`notificationActions.ts`、`extension-detail.test.ts` 等 | 更新为现行路径 |
| 原型 QA 脚本 `…/05-原型/原型/qa/*.mjs` | 用 `src-ui` 解析 jsdom 依赖 | `apps/desktop/ui` |

文档（现行入口）：

| 文件 | 处理 |
|---|---|
| `docs/02-项目核心/{产品定义, UI规范, 日志分类展示方案, 扩展管理详情方案}.md` 等的路径/落点引用 | 更新为 `apps/desktop/{ui,tauri}` |
| `docs/03-开发实施/{IMP-02, REL-01, quality-remediation/04, 05}.md` 的现行路径引用 | 更新（历史按日期记录保留） |
| 原型包 `README.md` | 更新现行路径引用 |
| `docs/02-项目核心/系统架构.md`、`术语与命名.md` | 登记新目录结构（`src-ui/src-tauri` → `apps/desktop/{ui,tauri}`） |

历史（保留不动）：`IMP-01` 各日期变更行、协作包历史产物、`docs/README` 历史条目。

## 阶段计划

| 阶段 | 内容 | 完成门槛 |
|---|---|---|
| A 准备与冻结 | 记录基线（HEAD/工作区/工具链/依赖锁）；确认影响面清单；建立 TASK 与检查点 | 清单齐备、基线可追踪 |
| B 目录移动与引用更新 | `git mv` 两个目录；改上表全部现行引用；`ui` 重新安装/生成 `dist` | 无残留现行引用（历史除外）；构建产物就位 |
| C 门禁与真实打包 | 前端 typecheck / vitest / build；后端 fmt / clippy / test；**真实 `tauri build`**（仓库根运行）；原型 QA 探针复跑 | 全绿；出 `.app` + `.dmg`；探针通过 |
| D 收口与回写 | 更新核心文档（架构/UI规范/术语登记）；写检查点与证据；TASK/RCP/CERT 收口；按需提交 | `project-check = valid`；文档与实际一致 |

## 验证门槛

- 目录结构符合"目标"；除历史文本外无 `src-ui` / `src-tauri` 现行引用。
- 前端 typecheck / vitest / build 通过；后端 fmt / clippy / test 通过。
- 真实 `tauri build` 通过并产出 `.app` + `.dmg`（证明 Tauri 认到 `apps/desktop/tauri`）。
- 原型 jsdom / 浏览器探针通过（QA 脚本路径已修）。
- `project-check` 判定 `valid`（索引 fresh、source binding 无漂移）。

## 风险与回滚

| 风险 | 缓解 |
|---|---|
| Tauri 目录发现（3 层上限） | `apps/desktop/{ui,tauri}` 恰为深度 3；执行时先真实 `tauri build` 预检 |
| 测试内相对路径漏改（`../../src-tauri/…`） | 以门禁为准；先 grep 兜底再跑测试 |
| 原型 QA 脚本解析依赖失败 | B 阶段同步改 `qa/*.mjs` 的依赖路径 |
| 提交粒度与历史 | 改造独立成提交；不改写历史；出问题 `git revert` |

- **回滚**：改造在独立提交上完成；任一门槛不过即回退到基线，不影响数据与契约。

## 授权边界

- 本文件只**制定计划**，不启动执行。
- 执行时：目录移动与引用更新在本地进行；**GitHub 推送与 PR 由用户授权后另行处理**；不涉及发布、签名、真实用户数据。

## 记录位置

- 计划：本文件（`docs/03-开发实施/`）。
- 执行 TASK、检查点、回执与证据：`.adg/`（执行时新建 `.adg/work/` 下的目录）。
- 人类长期结论：`docs/`。

## 执行结果（2026-09-27，TASK-81）

### 目录与引用

- `git mv src-ui apps/desktop/ui`、`git mv src-tauri apps/desktop/tauri`；行为/契约/功能不变。
- 现行引用已同步：`.github/workflows/ci.yml`（`npm --prefix apps/desktop/ui` ×4、backend `working-directory: apps/desktop/tauri`、build job 自行 `npm ci`）、`apps/desktop/tauri/tauri.conf.json`、`apps/desktop/tauri/build.rs`、`panel.rs`/`scale.ts`/`notificationActions.ts` 注释、前端 3 个测试相对路径、原型 `qa/*.mjs` 的 jsdom 解析路径、根 `README.md` 开发命令、`docs/02-项目核心/{UI规范,产品定义,日志分类展示方案,扩展管理详情方案}.md`、`05-原型/README.md`；并在《系统架构》《术语与命名》登记新结构。
- 历史文本（IMP-01 日期变更行、协作包历史产物、旧执行记录中的路径字样）按计划保留不动。

### 执行中修正的两处（计划未预见，按真实 `tauri build` 复核）

1. **构建 hook 工作目录**：Tauri 以「tauri 目录的父目录」为 hook 工作目录（旧布局 `src-tauri` 的父目录＝仓库根，故原 `--prefix src-ui` 成立）。迁移后父目录为 `apps/desktop`，故钩子应为 `npm --prefix ui run build`（**不是**计划里写的 `../ui`，实测 `../ui` 会去找 `apps/ui/package.json` 而失败）。`frontendDist` 仍按配置文件相对路径 `../ui/dist`。
2. **后端仓库根解析**：`commands/workspace.rs::project_document` 原以清单目录上一级为仓库根，新布局下清单在 `apps/desktop/tauri`，改为向上 3 层（`ancestors().nth(3)`）；否则「关于」页打开 LICENSE / 第三方许可的用例 `local_documents_are_frozen` 失败（`result.opened == false`）。

### 门禁（全绿）

| 层 | 命令 | 结果 |
|---|---|---|
| 前端类型 | `npm --prefix apps/desktop/ui run typecheck` | 通过 |
| 前端单测 | `npm --prefix apps/desktop/ui run test -- --run` | 53 文件 259 通过 / 0 失败 |
| 前端构建 | `npm --prefix apps/desktop/ui run build` | 通过 |
| 后端格式 | `cargo fmt --all -- --check`（`apps/desktop/tauri`） | 通过 |
| 后端静态 | `cargo clippy --workspace --all-targets -- -D warnings` | 通过（迁移后 `cargo clean` 重建，消除 `target/` 内旧绝对路径缓存） |
| 后端测试 | `cargo test --workspace --features integration-test` | 396+ 通过 / 0 失败 / 3 忽略 |
| 原型 jsdom | 5 个 `qa/*.test.mjs` | 315 通过 / 0 失败 |
| 原型浏览器 | 6 个 `qa/*.browser.mjs` | 415 通过 / 0 失败 |
| 真实打包 | `npx tauri build --target aarch64-apple-darwin`（仓库根） | 出 `.app` + `.dmg` |

制品（对应当前源码）：`apps/desktop/tauri/target/aarch64-apple-darwin/release/bundle/macos/OpenCodeX Desktop.app`（主程序 sha256 前 16 位 `aab380dcf2c10207`）、`…/bundle/dmg/OpenCodeX Desktop_0.1.0_aarch64.dmg`（sha256 前 16 位 `4301e7315ccd3fb1`）。

说明：本轮未做签名/公证/发布（授权边界）；Windows 及其他平台不在本轮范围。原型浏览器探针会重渲染 `…/05-原型/文档/截图/` 下的截图，本轮运行后已回退该二进制变化，不纳入本次改动。
