<p align="center">
  <img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/logo.png" width="120" alt="OpenCodeX Desktop" />
</p>

<h1 align="center">OpenCodeX Desktop</h1>

<p align="center">
  把 OpenCodex 的<strong>发现、启停、状态观测、模型配置、同步与扩展管理</strong>，收进一个桌面图形界面。
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
> 项目提供 macOS Apple Silicon 与 Windows x64 的公开测试版本。macOS 包使用 ad-hoc 签名，尚无 Developer ID 签名或 Apple 公证；Windows 安装包未做 Authenticode 签名。自更新资产的更新签名独立验证。实际公开版本与下载以 Releases 为准。

---

## 界面预览

<p align="center">
  <img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/overview-light.jpg" width="82%" alt="概览（浅色）">
</p>

<table>
  <tr>
    <td width="50%"><img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/overview-dark.jpg" alt="概览（深色）"></td>
    <td width="50%"><img src="https://raw.githubusercontent.com/gzers/opencodex-desktop/docs/governance-main/docs/04-%E9%A1%B9%E7%9B%AE%E8%B5%84%E6%96%99/04-%E5%93%81%E7%89%8C%E4%B8%8E%E5%9B%BE%E6%A0%87%E7%B4%A0%E6%9D%90/05-README%E7%B4%A0%E6%9D%90/panel-dark.jpg" alt="面板（深色）"></td>
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
| OpenCodex | 从 0.2.23 开始随 Desktop 附带完整 Node.js 26.8.1、npm 11.19.0 与维护运行时 2.97.0-desktop023.names.1，首次启动准备，无需另装 Node/npm 或单独导入运行包 |

### 下载

> [!NOTE]
> 下方链接指向当前公开的 Latest；候选分支与草稿 Release 不自动进入稳定通道。历史版本和各版已知限制请查看 Releases。

<p align="center">
  <a href="https://github.com/gzers/opencodex-desktop/releases/latest"><b>⬇ 前往最新版本下载</b></a>
</p>

- **最新版本（永远指向最新）**：<https://github.com/gzers/opencodex-desktop/releases/latest>
- 全部版本与历史：<https://github.com/gzers/opencodex-desktop/releases>

从 0.2.23 开始，Desktop 附带完整 Node.js 26.8.1、npm 11.19.0 和本项目维护的 OpenCodex 运行时 `2.97.0-desktop023.names.1`（基于上游 2.97.0，含官方 Web 面板资源）。首次启动在实际安装目录的 `runtime/` 下准备运行环境。维护修补是本项目的独立修订，不代表上游已发布或吸收这些修改。Desktop 更新保留已有 runtime；runtime 独立升级仍需明确确认，“联网安装”使用官方上游包。各版供应内容、校验信息和已知限制以对应 Releases 为准。

发布包由 GitHub Actions 在打 tag 后自动产出：

| 平台 | 文件 | 说明 |
|---|---|---|
| macOS Apple Silicon | `*.dmg` / `*.app` | ad-hoc 签名，无 Developer ID 签名或 Apple 公证 |
| Windows x64 | `*_x64-setup.exe`（NSIS）／ `*.msi` | 未签名测试构建 |

### macOS 安装

从 0.2.23 开始，macOS 使用独立的用户可写安装文件夹：

1. 下载并打开对应版本的 DMG，将顶层整个 **OpenCodeX Desktop 文件夹**复制到用户的 `~/Applications`，保留其内部结构。
2. 打开 `~/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app`。
3. 首次启动在同一文件夹内准备 `runtime/`；App 与 runtime 是同级目录，不放进配置 HOME。

因为当前测试包只有 **ad-hoc 签名，没有 Developer ID 签名或 Apple 公证**，macOS 可能会拦截。两种表现分别处理：

**表现 A：提示「无法打开，因为 Apple 无法检查其是否包含恶意软件」**
→ 右键（或按住 Control 点击）App 图标 →「打开」→ 在对话框中再次点「打开」。

**表现 B：提示「已损坏，无法打开。你应该将它移到废纸篓」**（最常见的拦截）

右键「打开」**对这种情况无效**，需要在「终端」里清除这个 App 的隔离属性：

```bash
# 推荐：只移除「下载隔离」属性
sudo xattr -dr com.apple.quarantine "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"
```

如果上面这条执行后仍然被拦，再依次尝试：

```bash
# 1) 清除该 App 的全部扩展属性
sudo xattr -c "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"

# 2) 仍不行时，做一次本机 ad-hoc 重新签名
sudo codesign --force --deep --sign - "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"

# 3) 重新打开
open "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"
```

> 排查小抄：`xattr -l "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"` 查看属性，`spctl -a -vv "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"` 查看 Gatekeeper 判定。
> 若你直接从挂载的 DMG 内运行，请先把整个文件夹复制到 `~/Applications`，再执行上面的命令。

### Windows 安装

1. 下载 `*_x64-setup.exe`（或 `.msi`）并运行。
2. 若出现蓝色 **SmartScreen** 提示，点「更多信息」→「仍要运行」。
3. 默认程序目录为当前用户的 `%LOCALAPPDATA%/Programs/OpenCodeX Desktop`，runtime 在该目录内准备；明确选择的自定义安装目录同样有效。

> Windows 安装包未做 Authenticode 签名。已验证的普通／管理员、安装及数据保留场景见各版发布记录，不宣称全部设备矩阵通过。默认卸载保留数据；NSIS 明确选择清理时只处理核验自有根，外置 HOME 保留。

---

## 快速上手

1. **首次启动**：等待随附运行环境在安装目录内准备完成；设置显示实际安装、runtime、数据根和配置路径。无需扫描 PATH、旧 runtime 或旧数据目录寻找可用安装。
2. **启动 / 停止 / 重启**：概览页直接托管官方 `ocx start / stop / restart` 子进程，并展示端口、进程标识与最近错误。
3. **看状态**：状态拆成**运行 / 连接 / 操作**三个维度，可同时成立；动作按状态渲染，不出现无解释的禁用按钮。
4. **面板**：切到「面板」页查看官方 Web 面板（需 OpenCodex 正在运行）。
5. **诊断**：诊断中心提供官方 `ocx doctor` 只读摘要、日志（应用日志 / 调用日志，敏感值已掩码）与通知历史。
6. **扩展管理**：统一管理 **Skills** 与 **MCP**，可分发到 Codex、Claude、Gemini、Grok、Opencode、Hermes 六个客户端；写入前备份、原子替换、可逐客户端连接 / 断开。
7. **模型配置**：用数字版本、模型族及名称条件定义模版匹配规则。“读取现有渠道”自动匹配并生成草稿；核对后明确保存才写入运行配置。单模型可覆盖字段与协议，渠道不设统一协议；模版另存不可变版本，默认与采用分别确认。
8. **文件导入导出与同步**：按模版、渠道、型号、模型族或管理器对象选择范围，先预览差异再确认。使用新 v6 模型包与 ObjectsV2 对象格式，不回退旧格式；凭据须单独明确选择。加密可选，不加密时文件含明文凭据，口令不写入包。账户仅传受支持的公开信息，新设备重新登录。
9. **数据目录**：设置展示当前实际路径；明确切换用户聚合根或 `OPENCODEX_HOME` 后重启生效，不自动扫描、兼容或迁移旧目录。
10. **托盘与菜单**：托盘菜单用图标与颜色表达运行状态，并提供 macOS 原生菜单入口。
11. **可选命令行控制面**：启用后注册 `ocxd`，对运行中实例经本机 IPC 委托执行；**默认关闭**。


### 0.2.23 的目录与升级

程序和运行环境使用安装根 `I`；用户文件使用聚合根 `U`，两者分开：

| 目录 | 默认位置与用途 |
|---|---|
| 安装根 `I` | macOS 为实际 `.app` 的父目录，默认 `~/Applications/OpenCodeX Desktop`；Windows 为实际 exe 的父目录，默认 `%LOCALAPPDATA%/Programs/OpenCodeX Desktop`；自定义安装位置以实际路径为准 |
| 运行环境 | `I/runtime/`，包含 Node/npm、维护运行时及面板资源 |
| 用户聚合根 `U` | 当前实际用户的 `~/.codex-desktop`；macOS、Windows 及不同登录用户各自独立 |
| 管理器数据根 `D` | `U/desktop`，保存管理器状态、日志与备份 |
| OpenCodex HOME `H` | `U/opencodex/home`，配置文件为 `H/config.json` |

升级前请自行备份原 OpenCodex 的 `config.json`。新目录初始化后，将备份放到设置显示的**实际配置文件路径**，再进入模型配置“读取现有渠道”，核对自动模版匹配草稿并明确保存。应用不自动迁移旧目录或回退读取旧格式。

使用同步的设备应全部升级；用户确认完整本机来源后，再按对应发布说明重建空同步目录。同步工具不会代替用户删除生产远端目录。

### 配置文件与设置

版本固化的默认值与用户选择分开保存，避免升级覆盖用户偏好：

| 文件 | 用途 | 位置 |
|---|---|---|
| `preferences.defaults.json` | 版本固化默认偏好（构建期校验后嵌入应用） | 随应用发布，用户不直接编辑 |
| `runtime.defaults.json` | 版本固化运行策略（超时、保留、运行入口、更新端点等） | 随应用发布，用户不直接编辑 |
| `network.defaults.json` | 版本固化网络行为策略（超时、重试、UA、TLS 约束） | 随应用发布，用户不直接编辑 |
| `manager-state/preferences.json` | 用户偏好（界面修改的通道、缩放、主题、外观等） | 数据根内，`0600` |
| `manager-location.json` | 当前用户选定的聚合根定位 | 固定 `~/.codex-desktop/desktop/` 内 |
| `manager-state/data-root.json` | 活动数据根与实例 HOME 的配置 | 活动 `D` 内 |

覆盖与生效：用户偏好优先于版本默认；缺失新增字段按默认读取，非法或损坏的偏好会显式失败并保留原件，不会被静默重置或覆盖。界面修改的字段即时生效或标注待重启；导出 / 同步只传输允许迁移的偏好投影，机器路径和运行状态留在本机，资源凭据单独显式选择。

主题：选择以后端偏好为事实源；启动早期先用本机缓存渲染首屏，随后由偏好回读覆盖，旧缓存不会反向覆盖新选择。

更新通道：稳定 / 测试通道与「自动检查更新」解耦；检查、安装与后台调度共用同一通道来源，切换通道会作废旧候选。应用自身更新已指向本仓库 Releases 的真实清单端点（稳定通道读 `latest.json`，测试通道读固定 `beta` tag）；签名校验通过后才会安装，失败保留当前版本。

网络代理：通用设置可选择无代理 / 自动（读系统代理）/ 手动 HTTP 或 SOCKS5 并配置例外列表；应用自更新、官方远端查询与托管安装共用该代理。本版不含代理鉴权，不使用系统钥匙串。

配置格式：应用版本与配置 schema 版本分别管理；偏好按域保存。0.2.23 使用新配置与对象格式，旧路径和旧格式不自动转换；遇到不支持的格式应保留原件，按对应发布说明处理。

测试隔离：开发与自动化测试使用独立沙箱身份（独立数据根、凭据服务与实例身份），不会写入日常使用目录；沙箱根缺失时启动即停止，不回退到日常配置。

---

## 功能特性

| 能力 | 说明 |
|---|---|
| **运行环境供应** | 从 0.2.23 开始随 Desktop 附带完整运行环境，首次启动准备到实际 `I/runtime`；独立升级须确认，不依赖 PATH 扫描或旧目录回退 |
| **运行托管** | 直接托管官方启停子进程；不安装或管理 launchd，不接管 `codex-shim` |
| **状态观测** | 运行 / 连接 / 操作三维状态，可同时成立；按状态渲染动作 |
| **日志与诊断** | 日志只读、敏感值掩码；展示官方 `ocx doctor` 的只读摘要；代理离线时明确说明实时日志不可用 |
| **官方面板** | 以运行时子 WebView 承载官方 Web 面板，**不重绘、不注入**；面板资源随运行时附带 |
| **通知中心** | 与「诊断中心 → 通知历史」共用同一份数据，支持详情与单条删除 |
| **模型配置** | 数字版本、模型族/名称条件与不可变模版版本；读取只生成匹配草稿，保存前比较、备份和读回；协议与覆盖按模型设置 |
| **制模助手** | 本机执行模型、公开资料和持续纠偏；逐字段接受到缓冲；外部任务按范围/基准/锁预检并另存候选，不自动采用或应用 |
| **文件导入导出** | v6 / ObjectsV2 选区导入导出，不回退旧格式；凭据单独明确选择，加密可选，不加密时文件含明文凭据，口令不入包；接收先比较再确认 |
| **WebDAV 同步** | 使用可信 HTTPS 服务，不依赖 ETag，以文件、提交和图关系校验；明确选择上传/下载，本机更新前备份，冲突先处理，失败后核验原操作。远端存储、本机应用、本机内容关系、远端图关系四项分别报告；关系以本次观测为准 |
| **扩展管理** | Skills 发现 / 导入 / 更新 / 卸载 / 恢复；MCP 导入 / 新增 / 编辑 / 删除；多客户端落点 |
| **外观** | 高 / 中 / 低三档视觉特效；概览背景光场（WebGL 网格渐变 + 颗粒着色器，环境不支持时回退 CSS 极光）与玻璃材质 |
| **双升级通道** | Desktop 更新验签且保留已有 runtime；runtime 独立升级另行确认，联网安装仍用官方上游包 |
| **托盘与菜单** | 托盘动作限定在管理器自有域；提供 macOS 原生菜单入口 |
| **CLI 控制面（可选）** | `ocxd` 默认关闭；macOS 启用后经本机 IPC 委托运行实例。Windows 产品偏好仍禁用 CLI |

### 明确不做的事（安全边界）

- 不重写 OpenCodex 核心，不绕过官方 API。
- 不接管 npm，不替代官方 `ocx update` 的更新事务。
- 不接管 `codex-shim`，不安装或管理 launchd。
- 模型配置从实际已保存来源生成草稿，明确保存前先比较、校验和备份，保存后读回；模版库、草稿与运行配置各自独立，协议按模型设置。
- 不自动覆盖外部 provider 托管的配置；只检测、解释并在用户确认后引导官方 `restore`。
- 密钥不进入仓库、Markdown、日志或命令行参数；未选凭据不导出。用户明确选择未加密导出时，按选择生成含明文凭据的可读文件。
- WebDAV 远端一律视为**不可信**；TLS 校验失败默认拒绝，**不提供「忽略证书」选项**。

---

## 常见问题与排错

<details>
<summary><b>macOS 提示「已损坏，无法打开」</b></summary>

只有 ad-hoc 签名、未经 Apple 公证的包可能被 Gatekeeper 隔离。右键“打开”对“已损坏”的情况无效，执行：

```bash
sudo xattr -dr com.apple.quarantine "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"
```

仍不行就 `sudo xattr -c`，再不行做一次 `sudo codesign --force --deep --sign - "$HOME/Applications/OpenCodeX Desktop/OpenCodeX Desktop.app"`。详见 [macOS 安装](#macos-安装)。
</details>

<details>
<summary><b>Windows 提示 SmartScreen / 未知发布者</b></summary>

当前是未签名构建。点「更多信息」→「仍要运行」即可；正式签名在后续发布计划中。
</details>

<details>
<summary><b>概览页显示「未发现 OpenCodex」</b></summary>

从 0.2.23 开始运行环境随 Desktop 附带。先查看安装目录内 runtime 的准备状态与错误，确认安装目录可写；以设置显示的实际运行入口为准，不扫描旧目录或全局安装回退。
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
<summary><b>数据目录不可写 / 切换未生效</b></summary>

在设置核对实际聚合根和 `OPENCODEX_HOME`；显式切换后需重启。目标目录应可写，且与安装目录分离。应用不会自动搬迁或扫描旧目录；旧配置按上面的升级步骤手动备份、放置并读取。
</details>

<details>
<summary><b>WebDAV 同步报 TLS / 证书错误</b></summary>

出于安全设计，**本项目不提供「忽略证书」选项**。自签证书请在本机信任链中正确安装，或改用受信任的端点。
</details>

<details>
<summary><b>系统弹出钥匙串 / 凭据授权</b></summary>

WebDAV 连接口令和已保存的内容保护口令位于 `D/manager-state/credentials/`：macOS 使用 `0600` 私有文件、`0700` 目录；Windows 使用当前用户作用域的 DPAPI 保护。模型私有 Key 沿用其已绑定来源和原生凭据存储。文件原操作恢复所用的本机密封 key 另存原生钥匙串，服务名为 `opencodex.file-transfer-recovery.v2`；系统授权提示可能来自模型 Key 或该恢复 key 的访问。
</details>

<details>
<summary><b>日志与数据在哪</b></summary>

两平台默认都使用当前实际用户的 `~/.codex-desktop`：管理器数据在 `desktop/`，OpenCodex 配置在 `opencodex/home/config.json`，日志在活动 `D/logs/`。设置显示选定路径；不同用户各自独立，程序与 runtime 留在实际安装目录。偏好备份不包含面板完整配置、会话或登录态。
</details>

<details>
<summary><b>应用自动更新没反应</b></summary>

在软件更新中手动检查并查看具体错误和网络设置。稳定通道读取本仓库 Latest 清单；没有更高候选时不会安装。自动检查只检查和通知，不自动下载或重启；安装必须通过更新签名验证。
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

推送版本tag触发Release工作流：提取对应版本说明、构建签名更新资产并创建草稿预发布，Windows在临时runner核验实际安装器。核对制品后运行Publish Release，填写版本tag并选择stable或preserve-latest；发布器验签后公开并检查匿名清单。

### 分支模型

| 分支 | 承载 |
|---|---|
| `main` | **代码**：`apps/`、`.github/`、`test/` 与仓库根配置 |
| `docs/governance-main` | **文档与治理产物**：`docs/`、`.adg/` |

文档记录实现状态时只引用实现分支与真实证据，不复制源码。

---

## 项目状态与已知限制

- 已有公开测试版本，实际稳定投递以 Releases Latest 为准；候选开发和草稿 Release 不等于已公开。
- 需求、原型与实施契约已定稿并通过评审门；13 个能力域均已实现，多轮桌面端真机验收已完成。
- **已知限制**：macOS 为 ad-hoc 签名，无 Developer ID 签名或 Apple 公证；Windows 无 Authenticode 签名，Intel macOS 构建未开启，Windows CLI 禁用。账户只传受支持的公开信息，新设备需登录。保存不等于 Codex 客户端已加载，需按入口核对；历史版本的未验项与资源例外以各版发布说明为准。

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
- 兼容性说明与原型参考使用 OpenCodex 的名称、版本、链接及界面素材；从 0.2.23 开始安装包另附完整运行环境和维护运行时，相关上游软件及面板资源仍适用各自许可，维护修订不等同于上游发行。
- OpenCodex 的源代码、名称、商标及其他权利归其权利人所有；使用官方软件请以官方仓库的 `LICENSE`、`NOTICE` 与服务条款为准。
- 本项目自身代码与素材采用 **MIT License**（见 [`LICENSE`](LICENSE)）。两套许可**彼此独立**。
- 仓库 `docs/` 下收录少量**官方来源素材**（官方页面快照、官方界面截图、官方标识），仅用于原型还原与兼容性参考；这些素材**不属于**本项目 MIT 授权范围，权利归其权利人所有，不得脱离本项目再分发或作商标性使用。
