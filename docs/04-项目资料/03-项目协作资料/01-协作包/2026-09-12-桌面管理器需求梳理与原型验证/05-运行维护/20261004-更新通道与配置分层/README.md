---
id: MNT-OPENCODEX-DESKTOP-20261004-02
object_kind: maintenance.record
state: analysis-registered
title: 更新通道失效与配置分层规划
summary: 分析官方 OpenCodex 与本项目桌面管理器两条更新通道的实际状态，确认桌面自更新通道不生效、官方面板更新对托管/自定义前缀不安全，并提出通道与配置分层规划；拟修复版本备注 0.1.2。
source_refs:
  - CODEX-THREAD:01a1047b-e13d-7241-acc1-279626511d94
  - REL-OPENCODEX-DESKTOP-01
---

# MNT-OPENCODEX-DESKTOP-20261004-02 更新通道失效与配置分层规划

## 登记与版本

| 字段 | 值 |
|---|---|
| 所属协作包 | COL-LOCAL-20260912-01，承接首个 0.1.0 测试版的原协作包（运行维护阶段） |
| 登记日期 / 来源 | 2026-10-04，用户提问与文档写入授权（线程 01a1047b-e13d-7241-acc1-279626511d94） |
| 受影响版本 | **0.1.1**（当前可见事实：桌面管理器自更新通道不生效、官方卡片只读；证据见下） |
| 拟修复版本备注 | **0.1.2**（尚未修改软件版本号、实施源码、回归或发布） |
| 当前状态 | 分析已登记；仅登记两条更新通道的现状、根因与规划方向。未实施、未验证、未发布 |
| 与 MNT-01 关系 | 独立记录。可考虑合并进 0.1.2，但**不并入** 0.1.1（0.1.1 已冻结 F-01/03/04/05/06/07/09/13） |
| 正式运行事实 | 实施与发布结果以 `docs/03-开发实施/RUN-\*` 为准，本记录不预写执行结果 |

本记录晚于 [MNT-OPENCODEX-DESKTOP-20261004-01](../20261004-白屏与托盘左键菜单/README.md) 登记，编号顺延为 02。软件版本 0.1.1 / 0.1.2 均为测试版维护备注，不等同于需求原型的 V0.1～V0.9。**本记录不重开协作包、不改变 `COL-LOCAL-20260912-01` 的 open 状态，也不改动任一旧需求 / 实施对象状态。**

## 现象与范围

用户在 0.1.1 安装实例上看到两处版本卡片：

- 「OpenCodex 版本」：显示 `v2.69.0 · 未发现 npm 全局安装`，点击“检查更新”不产生远端结果。
- 「桌面管理器版本」：显示 `v0.1.1 · 稳定通道`，点击“检查应用更新”得到“更新下载或连接失败；已保留当前版本”，标签为「失败」。

用户同时在**官方面板内部**看到可用更新（`已安装 2.69.0 / 最新 2.77.0`），并在本地实测 `@bitkyc08/opencodex` 的 `dist-tags.latest = 2.77.0`、`preview = 2.76.0-preview.20261003`。因此问题是三条链路各自不同，而不是单一故障：

1. 官方 OpenCodex 的版本/更新链路；
2. 桌面管理器自身（Tauri）的自更新链路；
3. 官方面板内置更新器与桌面管理器“托管前缀 / 自定义前缀”的兼容性。

## 分析与结论

### 一、官方 OpenCodex 版本卡片（桌面管理器内）

桌面管理器的这张卡**不做远端更新查询**：前端 `loadOfficialProject()` 调用后端 `official_project_facts`，后端经受控来源只执行 `ocx --version`，用输出填充“当前版本 / 安装形态”。它既不访问 npm registry，也不比较远端版本，所以“未发现 npm 全局安装”和“检查更新”名不副实——按钮实际只是“重新读取本地版本”。这与设计口径一致（桌面管理器不接管官方更新事务，只做备份与引导），属**产品语义缺口**，非崩溃。

### 二、桌面管理器自身自更新通道

**当前完全不生效**，三个原因叠加：

1. **端点占位**：`apps/desktop/tauri/tauri.conf.json` 的 `plugins.updater.endpoints` 仍是 `https://updates.example.invalid/opencodex-desktop/{stable,beta}.json`，`updater.check()` 必然连接失败，界面落在“更新下载或连接失败”。
2. **公钥占位**：`pubkey` 解出来是 `untrusted comment: signature from tauri-cli`，不是真正的 minisign 公钥。即便端点可达并存在 `latest.json`，签名校验也不通过，“签名校验通过后才安装”路径不可达。
3. **无制品/签名管线**：`tauri.conf.json` 未设 `bundle.createUpdaterArtifacts`，`.github/workflows/release.yml` 未注入 `TAURI_SIGNING_PRIVATE_KEY`，CI 不产出 `latest.json` 与 `.sig`，服务端没有可更新的对象。

结论：`AC-09` 要求的 stable/beta 自更新在实现上尚未成立，与 `IMP-05` §469–474、`REL-01` 第 146 行的“仍为占位”记录一致。

### 三、官方面板更新为何外部做不到

面板内的“更新 opencodex”弹窗由**官方代理后端**驱动：`GET /api/update/check`、`GET /api/update/status`、`POST /api/update/run`。其 `check` 从**当前运行包路径**推断安装方式（`detectInstall`），再执行 `npm view @bitkyc08/opencodex@<tag> version`；`run` 由 `src/update/job.ts` 起 worker 走事务式安装。桌面管理器**外部**做不到，原因分三层：

- 桌面管理器自更新走 Tauri 占位端点（见上），不是同一条链路；
- 官方卡片只执行本地 `ocx --version`，从不请求远端；
- 桌面管理器刻意不读 PATH、不解析 npm 全局前缀，官方更新只暴露“复制 `ocx update`”的引导，不代跑。

### 四、官方面板更新对“托管 / 自定义前缀”是否安全

**不安全，语义错位**。官方更新器面向“标准 npm / pnpm 全局安装”设计，对桌面管理器的托管前缀（`<数据根>/runtime/opencodex/node_modules/@bitkyc08/opencodex`）与自定义登记前缀有以下问题：

- `npm` 分支把**当前运行包的兄弟目录**当作用户前缀；托管布局中包外侧直接是 `node_modules`、没有 `lib/node_modules`，与其 `staging` / `sweepUpdateLeftovers` 的布局预期不符。
- 其 `installArgs = npm install -g <pkg>@<tag>` **不带 `--prefix`**，会落到它自己的全局前缀，而不是托管前缀或管理里登记的 `custom` 前缀。
- `pnpm` 分支走 pnpm 全局存储与 shim，自定义前缀基本失效。
- 它不经过桌面管理器的约束：不写 `.runtime-manifest.json`、不做离线包 / 代理 / `--ignore-scripts` 收敛，也不重启由桌面壳托管的代理。

因此官方面板更新可作“标准全局安装”的更新通道，**不能**当作 desktop 自定义路径的更新通道。

### 五、两条更新的边界不可合并

- **桌面管理器自更新**（Tauri Updater）：更新的是应用本体（`OpenCodeX Desktop.app`）。
- **OpenCodex 本体更新**（npm 官方包）：更新的是托管前缀内的 `@bitkyc08/opencodex`。

`DMD` 已明确两套更新彼此独立、界面须让用户分清对象。二者的许可证、签名与回滚策略也不同，不能合并成一条通道。

## 配置分层审视

现有配置存在结构性问题，比“字段多”更值得规划：

1. **扁平化**：`Preferences` 为单层 28 字段，前后端各一份 DTO 再加 UI 类型，**三处手写映射**，校验集中在一条长 `allowed()` 白名单；新增字段要同时改多处。
2. **语义耦合**：`app_update_channel` 用单个字段同时编码“通道”和“频率”（`stable-24h` / `beta-6h` / `manual`），切频率必然切通道。
3. **死代码与双源**：`UpdateChannel::endpoint()` 定义了 stable/beta 两个 URL，但真实请求由 Tauri 插件按 `tauri.conf.json` 的静态 `endpoints` 发起，`endpoint()` 全仓无人调用；配置通道只影响状态投影，**不改变实际查询端点**。端点硬编码且重复。
4. **用户偏好与运行默认混淆**：更新端点、registry、npm 发现路径属“有默认值、可受控覆盖”的运行配置，与“用户偏好”混在同一份文件里。

## 规划方向（待评审）

> 以下为规划方向，**尚未实施，也不构成 0.1.2 的范围冻结**。

1. **按域分组**：`appearance / lifecycle / backup / extensions / sync / update / cli / advanced`；每域各自默认值、校验、迁移。`preferences.json` 只放用户偏好。
2. **通道与频率解耦**：拆为 `channel: stable|beta` 与 `checkInterval: manual|6h|24h`，界面文案随之调整。
3. **分离内置默认配置**：端点 / registry / 超时收进受版本管理的默认配置模块（可显式覆盖）；**公钥必须保持编译期受信任常量**（端点可配置，公钥不可），删掉 `endpoint()` 死代码，让配置真正驱动 `updater_builder().endpoints(...)` 或 registry 查询。
4. **偏好加 schema_version + 每域迁移**：不再只靠 `serde(default)` 兜底，避免旧文件被误判为损坏。
5. **收敛单一事实源**：Rust 域类型驱动 DTO，或用快照测试锁住三处映射，降低漂移。

### 官方版本卡片的产品选择（待用户决定）

- **A**：把按钮文案改为“重新读取本地版本”，承认它只做本地事实（最小改动，语义诚实）。
- **B**：新增只读远端查询（受控 `npm view @bitkyc08/opencodex@latest version` + `dist.integrity`，或 GET registry，带超时与掩码），与 `runtime.json` 的 `resolved_version` 比较出“有可用更新”。

### 官方更新的推荐技术路线（待评审）

- **查询**：复用桌面管理器已有能力，新增只读版本查询（推荐），或在运行期读官方代理的 `GET /api/update/check`（只读，最省事，但需管理 token）。
- **安装**：复用现成 `install_runtime`（`source=registry, version=latest, prefix=当前登记前缀`），沿用自定义前缀、离线包、代理、`.runtime-manifest.json`、重启提示等既有安全路径；**不**代理官方 updater 去改托管前缀。
- **桌面自更新**：补齐真实 `endpoints` + 真实 `pubkey` + CI `createUpdaterArtifacts` 与 `TAURI_SIGNING_PRIVATE_KEY`，产出签名后的 `latest.json` / `.sig`。
- **边界**：不接管 npm 包管理器写全局前缀；查询（只读）与安装（写）分开，符合 `AC-08`。

## 0.1.2 候选范围（未冻结）

| 编号 | 候选事项 | 类别 |
|---|---|---|
| U-01 | 桌面自更新端点为真实端点 + 真实公钥，可配置但有编译期默认 | 更新通道 |
| U-02 | 发布管线产出签名 `latest.json` / `.sig` | 更新通道 / CI |
| U-03 | 官方版本卡片：改名（本地事实）或新增只读远端查询 | 产品语义 |
| U-04 | 外部安装官方更新：复用 `install_runtime`，支持托管 / 自定义前缀 | 更新通道 |
| U-05 | 删除 `endpoint()` 死代码，端点 / 公钥单一事实源 | 配置 |
| U-06 | 配置按域分文件 / 分层，偏好加 schema_version 与迁移 | 配置 |
| U-07 | 通道与频率解耦 | 配置 / UI |
| U-08 | 三处偏好映射收敛为单一事实源 | 配置 |

## 文件索引与后续处理

- [问题来源记录](问题来源记录/README.md)：用户诉求摘录与登记授权、本轮证据来源。
- [分析索引](分析/README.md)：分析阅读顺序。
- [更新通道现状与证据](分析/01-更新通道现状与证据.md)：三条链路、根因与可核对路径。
- [配置分层规划建议](分析/02-配置分层规划建议.md)：分组、解耦、默认配置分离与迁移。
- [0.1.2-候选范围与待决策](分析/03-0.1.2-候选范围与待决策.md)：候选清单、待决策与限制。

## 运行边界与外部关联

本记录只承载分析、证据与规划，不复制源码事实、不预写执行结果。实施后真实结果回写 `docs/03-开发实施/` 的 `RUN-\*` 与 `REL-\*`；本记录保留分析与冻结边界。为减小引用风险，源码证据以路径 + 行号描述，不内联长段源码。相关长期事实见 `docs/02-项目核心/`、`DMD-OPENCODEX-DESKTOP-MANAGER` 与 `IMP-OPENCODEX-DESKTOP-05`。

