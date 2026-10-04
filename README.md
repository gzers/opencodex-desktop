<p align="center">
  <img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/05-README%E7%B4%A0%E6%9D%90/logo.png" width="120" alt="OpenCodeX Desktop" />
</p>

<h1 align="center">OpenCodeX Desktop</h1>

<p align="center">
  把 OpenCodex 的<strong>发现、启停、状态观测、配置迁移、同步与扩展管理</strong>，收进一个桌面图形界面。
</p>

<p align="center">
  <a href="https://github.com/gzers/opencodex-desktop/releases/latest"><img src="https://img.shields.io/github/v/release/gzers/opencodex-desktop?include_prereleases&sort=semver&label=release" alt="Latest release"></a>
  <img src="https://img.shields.io/badge/license-MIT-green.svg" alt="MIT">
  <img src="https://img.shields.io/badge/platform-macOS%20%7C%20Windows-lightgrey.svg" alt="Platform">
  <img src="https://img.shields.io/badge/Rust-1.89%2B-orange.svg" alt="Rust 1.89+">
  <img src="https://img.shields.io/badge/Node-%E2%89%A524-339933.svg" alt="Node 24+">
</p>

<p align="center">简体中文 · <a href="README.en.md">English</a></p>

> [!IMPORTANT]
> 本项目是**独立第三方桌面管理器**，不是 OpenCodex 官方项目，与官方及其权利人不存在隶属、授权或背书关系。它只做图形化托管，**不重写 OpenCodex 核心、不绕过官方 API**。详见[归属与许可](#归属与许可)。
>
> 项目仍处于**测试阶段，尚未发布正式版本**；现有产物是 **未签名测试包（macOS Apple Silicon 为主）**，仅用于体验与内测，不构成正式公开发布。

---

## 界面预览

<p align="center">
  <img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/05-README%E7%B4%A0%E6%9D%90/overview-light.jpg" width="82%" alt="概览（浅色）">
</p>

<table>
  <tr>
    <td width="50%"><img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/05-README%E7%B4%A0%E6%9D%90/overview-dark.jpg" alt="概览（深色）"></td>
    <td width="50%"><img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/05-README%E7%B4%A0%E6%9D%90/panel-dark.jpg" alt="面板（深色）"></td>
  </tr>
  <tr>
    <td align="center"><sub>概览 · 深色</sub></td>
    <td align="center"><sub>面板（内嵌官方 Web 面板）· 深色</sub></td>
  </tr>
</table>

> 应用界面目前**只提供中文**。深浅色随系统外观切换，并提供高 / 中 / 低三档视觉特效。

---

## 下载与安装

### 系统要求

| 项 | 要求 |
|---|---|
| 操作系统 | macOS（Apple Silicon，正式支持）／ Windows x64（测试构建） |
| OpenCodex | 需已在本机安装官方 OpenCodex（`ocx`）。管理器**只发现、不代装**；缺失时界面会给出安装引导 |

### 下载

> [!NOTE]
> 本项目仍在**测试阶段，尚无正式版本**，所以这里不写死任何版本号。下面用的是**永远指向最新发布**的链接：一旦发布正式版本，它会自动跳到最新版；在此之前它会落到 Releases 列表页。

<p align="center">
  <a href="https://github.com/gzers/opencodex-desktop/releases/latest"><b>⬇ 前往最新版本下载</b></a>
</p>

- **最新版本（永远指向最新）**：<https://github.com/gzers/opencodex-desktop/releases/latest>
- 全部版本与历史：<https://github.com/gzers/opencodex-desktop/releases>

发布包由 GitHub Actions 在打 tag 后自动产出：

| 平台 | 文件 | 说明 |
|---|---|---|
| macOS Apple Silicon | `*.dmg` / `*.app` | 未签名、未公证 |
| Windows x64 | `*_x64-setup.exe`（NSIS）／ `*.msi` | 未签名测试构建 |

### macOS 安装

1. 下载 `.dmg`，双击打开，把 **OpenCodeX Desktop** 拖入「应用程序」。
2. 从「应用程序」里打开。

因为当前是**未签名、未公证**的测试包，macOS 可能会拦截。两种表现分别处理：

**表现 A：提示「无法打开，因为 Apple 无法检查其是否包含恶意软件」**
→ 右键（或按住 Control 点击）App 图标 →「打开」→ 在对话框中再次点「打开」。

**表现 B：提示「已损坏，无法打开。你应该将它移到废纸篓」**（最常见的拦截）

右键「打开」**对这种情况无效**，需要在「终端」里清除这个 App 的隔离属性：

```bash
# 推荐：只移除「下载隔离」属性
sudo xattr -dr com.apple.quarantine "/Applications/OpenCodeX Desktop.app"
```

如果上面这条执行后仍然被拦，再依次尝试：

```bash
# 1) 清除该 App 的全部扩展属性
sudo xattr -c "/Applications/OpenCodeX Desktop.app"

# 2) 仍不行时，做一次本机 ad-hoc 重新签名
sudo codesign --force --deep --sign - "/Applications/OpenCodeX Desktop.app"

# 3) 重新打开
open "/Applications/OpenCodeX Desktop.app"
```

> 排查小抄：`xattr -l "/Applications/OpenCodeX Desktop.app"` 查看属性，`spctl -a -vv "/Applications/OpenCodeX Desktop.app"` 查看 Gatekeeper 判定。
> 若你直接在「下载」里从 dmg 内运行、而未拖入「应用程序」，请先拖入再执行上面的命令。

### Windows 安装

1. 下载 `*_x64-setup.exe`（或 `.msi`）并运行。
2. 若出现蓝色 **SmartScreen** 提示，点「更多信息」→「仍要运行」。

> Windows 版当前是**未签名测试构建**：代码已完成跨平台改造并能通过 CI 编译门禁，但尚未做真机安装 / 渲染核验，功能与观感可能不完整。

---

## 快速上手

1. **首次启动**：概览页会显示 OpenCodex 的发现结果与运行状态。若未发现 OpenCodex，按页面引导先安装官方 OpenCodex。
2. **启动 / 停止 / 重启**：概览页直接托管官方 `ocx start / stop / restart` 子进程，并展示端口、进程标识与最近错误。
3. **看状态**：状态拆成**运行 / 连接 / 操作**三个维度，可同时成立；动作按状态渲染，不出现无解释的禁用按钮。
4. **面板**：切到「面板」页查看官方 Web 面板（需 OpenCodex 正在运行）。
5. **诊断**：诊断中心提供官方 `ocx doctor` 只读摘要、日志（应用日志 / 调用日志，敏感值已掩码）与通知历史。
6. **扩展管理**：统一管理 **Skills** 与 **MCP**，可分发到 Codex、Claude、Gemini、Grok、Opencode、Hermes 六个客户端；写入前备份、原子替换、可逐客户端连接 / 断开。
7. **配置迁移与同步**：全量导出进口令保护的加密容器；WebDAV 同步在本地加密后上传，覆盖前备份，冲突先入待处理。
8. **数据目录**：在设置里选择或切换数据根与 `OPENCODEX_HOME`（引用优先、迁移可选、失败回滚）。
9. **托盘与菜单**：托盘菜单用图标与颜色表达运行状态，并提供 macOS 原生菜单入口。
10. **可选命令行控制面**：启用后注册 `ocxd`，对运行中实例经本机 IPC 委托执行；**默认关闭**。


### 配置文件与设置

版本固化的默认值与用户选择分开保存，避免升级覆盖用户偏好：

| 文件 | 用途 | 位置 |
|---|---|---|
| `preferences.defaults.json` | 版本固化默认偏好（构建期校验后嵌入应用） | 随应用发布，用户不直接编辑 |
| `runtime.defaults.json` | 版本固化运行策略（超时、保留、发现路径、更新端点等） | 随应用发布，用户不直接编辑 |
| `manager-state/preferences.json` | 用户偏好（界面修改的通道、缩放、主题、外观等） | 数据根内，`0600` |
| `manager-state/data-root.json` | 本机数据根定位与结构版本 | 数据根内 |
| `manager-state/config-migrations/` | 配置格式自动转换的原件备份与迁移记录 | 数据根内，仅转换发生时出现 |

覆盖与生效：用户偏好优先于版本默认；缺失新增字段按默认读取，非法或损坏的偏好会显式失败并保留原件，不会被静默重置或覆盖。界面修改的字段即时生效或标注待重启；导出 / 同步只传输允许迁移的偏好投影，机器路径、凭据与运行状态不进入同步。

主题：选择以后端偏好为事实源；启动早期先用本机缓存渲染首屏，随后由偏好回读覆盖，旧缓存不会反向覆盖新选择。

更新通道：稳定 / 测试通道与「自动检查更新」解耦；检查、安装与后台调度共用同一通道来源，切换通道会作废旧候选。应用自身更新的真实端点与签名仍在后续发布计划中，当前未接通。

配置格式自动转换：应用版本与配置 schema 版本分别管理。读取旧格式时自动完成已发布的迁移链并保留原件备份；遇到高于当前应用支持的版本会拒绝写入并保留原件，需要时可用迁移前备份恢复（会丢弃迁移后的新更改）。

测试隔离：开发与自动化测试使用独立沙箱身份（独立数据根、凭据服务与实例身份），不会写入日常使用目录；沙箱根缺失时启动即停止，不回退到日常配置。

---

## 功能特性

| 能力 | 说明 |
|---|---|
| **安装发现** | 只发现、展示、校验已有的安装形态；**不管理 npm**，不代装、不卸载、不解析依赖 |
| **运行托管** | 直接托管官方启停子进程；不安装或管理 launchd，不接管 `codex-shim` |
| **状态观测** | 运行 / 连接 / 操作三维状态，可同时成立；按状态渲染动作 |
| **日志与诊断** | 日志只读、敏感值掩码；展示官方 `ocx doctor` 的只读摘要；代理离线时明确说明实时日志不可用 |
| **官方面板** | 以运行时子 WebView 承载官方 Web 面板，**不重绘、不注入**、不打包官方素材 |
| **通知中心** | 与「诊断中心 → 通知历史」共用同一份数据，支持详情与单条删除 |
| **配置迁移** | 全量导出进**口令保护的加密容器**；导入前校验、自动备份、显式确认、失败回滚 |
| **WebDAV 同步** | 内容**客户端加密**后上传；覆盖前生成可恢复备份；冲突先入待处理，不静默覆盖 |
| **扩展管理** | Skills 发现 / 导入 / 更新 / 卸载 / 恢复；MCP 导入 / 新增 / 编辑 / 删除；多客户端落点 |
| **外观** | 高 / 中 / 低三档视觉特效；概览背景光场（WebGL 网格渐变 + 颗粒着色器，环境不支持时回退 CSS 极光）与玻璃材质 |
| **双升级通道** | OpenCodex 本体走官方 `ocx update` 引导；应用自身更新独立（签名校验、重启生效、失败回滚，更新端点当前仍为占位） |
| **托盘与菜单** | 托盘动作限定在管理器自有域；提供 macOS 原生菜单入口 |
| **CLI 控制面（可选）** | `ocxd` 命令，**默认关闭**；启用后作为运行中实例的客户端经本机 IPC 委托执行 |

### 明确不做的事（安全边界）

- 不重写 OpenCodex 核心，不绕过官方 API。
- 不接管 npm，不替代官方 `ocx update` 的更新事务。
- 不接管 `codex-shim`，不安装或管理 launchd。
- **不写入 provider、路由、模型映射等 OpenCodex 核心运行配置**——这类变更一律走官方 CLI。
- 不自动覆盖外部 provider 托管的配置；只检测、解释并在用户确认后引导官方 `restore`。
- 任何凭据不得进入仓库、Markdown、日志、命令行参数或未加密文件。
- WebDAV 远端一律视为**不可信**；TLS 校验失败默认拒绝，**不提供「忽略证书」选项**。

---

## 常见问题与排错

<details>
<summary><b>macOS 提示「已损坏，无法打开」</b></summary>

未签名包被 Gatekeeper 打上 `com.apple.quarantine` 隔离属性。右键「打开」对这种情况无效，执行：

```bash
sudo xattr -dr com.apple.quarantine "/Applications/OpenCodeX Desktop.app"
```

仍不行就 `sudo xattr -c`，再不行做一次 `sudo codesign --force --deep --sign - "/Applications/OpenCodeX Desktop.app"`。详见 [macOS 安装](#macos-安装)。
</details>

<details>
<summary><b>Windows 提示 SmartScreen / 未知发布者</b></summary>

当前是未签名构建。点「更多信息」→「仍要运行」即可；正式签名在后续发布计划中。
</details>

<details>
<summary><b>概览页显示「未发现 OpenCodex」</b></summary>

管理器只发现、不代装。请先安装官方 OpenCodex（`ocx` 可用），再回到概览页重试。前置条件（Node / npm）缺失时页面会给出对应引导。
</details>

<details>
<summary><b>面板页空白 / 加载失败</b></summary>

面板承载的是**官方 Web 面板**，需要 OpenCodex 正在运行且端口可达。请先在概览页启动 OpenCodex，再打开面板页；代理离线时面板不可用属预期行为。
</details>

<details>
<summary><b>实时日志不可用</b></summary>

实时日志依赖运行中的代理。代理离线时界面会明确标注不可用；本地历史日志与「打开日志目录」仍然可用。
</details>

<details>
<summary><b>数据目录不可写 / 迁移失败</b></summary>

切换数据根或 `OPENCODEX_HOME` 前会做可写性与空间检查，并在迁移前强制备份；失败会自动回滚并保留原目录。请确认目标目录是真实目录（非符号链接）、位于系统保护目录之外且有写权限。
</details>

<details>
<summary><b>WebDAV 同步报 TLS / 证书错误</b></summary>

出于安全设计，**本项目不提供「忽略证书」选项**。自签证书请在本机信任链中正确安装，或改用受信任的端点。
</details>

<details>
<summary><b>系统弹出钥匙串 / 凭据授权</b></summary>

同步与迁移相关凭据存放在 macOS 钥匙串（服务名 `OpenCodex Desktop`），首次访问会请求授权，属预期行为。
</details>

<details>
<summary><b>日志与数据在哪</b></summary>

默认数据根为 `~/Library/Application Support/OpenCodex Desktop/`，日志在其下的 `logs/`。设置页可直接打开数据目录与日志目录。
</details>

<details>
<summary><b>应用自动更新没反应</b></summary>

应用自身更新的端点当前仍是占位地址，尚未接通；请手动下载新版本覆盖安装。
</details>

---

## 面向开发者

### 架构概览

```text
┌─────────────────────────────────────────────┐
│ 前端界面层（Vue 3 + TypeScript）                │
│ 概览 / 面板 / 拓展 / 日志与诊断 / 设置 / 托盘     │
└──────────────────┬──────────────────────────┘
                   │ 命令调用（Tauri IPC）
┌──────────────────▼──────────────────────────┐
│ 管理器后端（Rust）                              │
│ 发现 · 进程托管 · 状态 · 日志 · 迁移 · 同步       │
│ 扩展管理 · 更新 · 数据根 · CLI/IPC · 通知         │
└──────────────────┬──────────────────────────┘
                   │ 子进程 / 文件 / 网络
┌──────────────────▼──────────────────────────┐
│ 外部依赖                                        │
│ ocx CLI · 官方 Web 面板 · 各客户端配置文件        │
│ WebDAV 远端 · 应用自身更新通道                   │
└─────────────────────────────────────────────┘
```

**硬边界：后端独占外部访问。** 前端不直接触碰文件系统与网络，所有外部访问经后端模块。

- **技术栈**：Tauri v2 + Rust 后端；Vue 3 + TypeScript + Vite + Pinia 前端；原生 CSS Design Token（不引入 UI 框架）。
- **前端结构**：按功能切片组织——`app/`（装配与外观）、`features/<域>/`（自带 IPC 与业务组件）、`components/`（`ui` / `layout` / `patterns`）、`routes/`（页面只做组合）、`stores/`、`navigation/`、`lib/`、`contracts/`。
- **后端模块**：`MOD-01` 发现、`MOD-02` 进程托管、`MOD-03` 状态、`MOD-04` 日志与诊断、`MOD-05` 面板、`MOD-06` 数据根、`MOD-07` 迁移、`MOD-08` 同步、`MOD-09` 扩展管理、`MOD-10` 更新、`MOD-11` 通知、`MOD-12` CLI/IPC、`MOD-13` 托盘与菜单。
- **受控写入**：用户动作 → 前置校验 → 备份（失败即阻断）→ 临时区写入 + 原子替换 → 重新校验 → 成功 / 回滚；写入持有跨进程文件锁并检测外部改写。

更完整的架构、领域模型与契约见 [`docs/02-项目核心/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/02-项目核心)。

### 目录结构

```text
apps/desktop/ui/        # 前端（Vue 3 + TypeScript + Vite）
apps/desktop/tauri/     # 后端（Rust + Tauri v2）
.github/workflows/      # CI（ci.yml）与发布（release.yml）
test/                   # 本机验收测试脚本与源码（软件副本不入库）
```

### 本地开发

```bash
# 依赖：Rust 1.89+、Node >= 24
npm --prefix apps/desktop/ui ci
npm install --global @tauri-apps/cli@2.11.4

# 前端开发服务器
npm --prefix apps/desktop/ui run dev

# 连接前端运行完整桌面应用（另一终端）
cd apps/desktop/tauri && tauri dev
```

### 构建与打包

```bash
npm --prefix apps/desktop/ui ci
cd apps/desktop/tauri
tauri build --target aarch64-apple-darwin   # macOS Apple Silicon；产物在 target/<triple>/release/bundle/
```

Windows 在 Windows runner 上执行 `tauri build` 即产出 NSIS / MSI（`bundle.targets` 为 `"all"`）。

### 测试门禁

```bash
# 前端
npm --prefix apps/desktop/ui run typecheck
npm --prefix apps/desktop/ui run test -- --run
npm --prefix apps/desktop/ui run build

# 后端（必须在 apps/desktop/tauri 下执行）
cd apps/desktop/tauri
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace --features integration-test
```

### 发版流程

1. 在 `CHANGELOG.md` 写一个 `## [X.Y.Z] - 日期` 段落（版本号**不带** `v`）——它是发布说明的事实源。
2. 对齐 `apps/desktop/tauri/tauri.conf.json` 与 `Cargo.toml` 的版本号。
3. 打 tag 并推送：

```bash
git tag vX.Y.Z
git push origin vX.Y.Z
```

推送 `v*` tag 会触发 `.github/workflows/release.yml`：先按 tag 从 `CHANGELOG.md` 抽取版本说明，再由 `tauri-action` 构建并创建 Release（**默认草稿 + 预发布**，确认无误后手动取消草稿）。

### 分支模型

| 分支 | 承载 |
|---|---|
| `main` | **代码**：`apps/`、`.github/`、`test/` 与仓库根配置 |
| `docs/governance-main` | **文档与治理产物**：`docs/`、`.adg/` |

文档记录实现状态时只引用实现分支与真实证据，不复制源码。

---

## 项目状态与已知限制

- **仍处于测试阶段，尚未发布正式版本**。`0.1.0` 为未签名测试包：macOS Apple Silicon 为主，Windows x64 为未签名测试构建。
- 需求、原型与实施契约已定稿并通过评审门；13 个能力域均已实现，多轮桌面端真机验收已完成。
- **已知限制**：无代码签名与公证；无自动更新；Windows 运行期测试与实机核验未完成；Intel macOS 构建未开启；非 macOS-arm64 平台按授权边界另行处理。

---

## 文档

- 项目事实入口：[`docs/README.md`](https://github.com/gzers/opencodex-desktop/blob/docs/governance-main/docs/README.md)
- 需求权威（DMD）：[`docs/01-需求管理/需求/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/01-需求管理)
- 项目核心（架构 / 领域模型 / 契约）：[`docs/02-项目核心/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/02-项目核心)
- 开发实施（IMP / REL 门禁）：[`docs/03-开发实施/`](https://github.com/gzers/opencodex-desktop/tree/docs/governance-main/docs/03-开发实施)
- 变更日志：[`CHANGELOG.md`](https://github.com/gzers/opencodex-desktop/blob/main/CHANGELOG.md)

> 治理与需求文档以中文为准。

---

## 归属与许可

- 本项目是**独立的桌面管理器**，与 OpenCodex 官方项目及其权利人不存在隶属、授权或背书关系。
- 仅在兼容性说明与原型还原的必要范围内使用 OpenCodex 的名称、版本、链接与界面参考素材；**不复制或重新分发其源代码与构建产物**。
- OpenCodex 的源代码、名称、商标及其他权利归其权利人所有；使用官方软件请以官方仓库的 `LICENSE`、`NOTICE` 与服务条款为准。
- 本项目自身代码与素材采用 **MIT License**（见 [`LICENSE`](LICENSE)）。两套许可**彼此独立**。
- 仓库 `docs/` 下收录少量**官方来源素材**（官方页面快照、官方界面截图、官方标识），仅用于原型还原与兼容性参考；这些素材**不属于**本项目 MIT 授权范围，权利归其权利人所有，不得脱离本项目再分发或作商标性使用。
