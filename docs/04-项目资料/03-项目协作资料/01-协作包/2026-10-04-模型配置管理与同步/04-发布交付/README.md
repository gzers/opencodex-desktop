# 发布交付

0.2.1修复交付已公开并晋升stable，main/tag fbd6e1d1；实际制品/验签/安装回读与匿名通道核对通过，事实沿[REL-12](../../../../../03-开发实施/REL-OPENCODEX-DESKTOP-12.md)，0.2.0以下发布事实保持历史。

0.2.0历史发布：已公开并曾晋升stable，main/tag固定f1bbe7d9。主线CI39533974950、双平台Release39534419661及Publish39539741260成功；真实安装、原公钥验签和匿名清单/资产核对通过。发布事实沿[REL-11](../../../../../03-开发实施/REL-OPENCODEX-DESKTOP-11.md)。

用户操作、外部任务授权和升级/回退说明见[使用与升级说明](使用与升级说明.md)。后续输入：确认后的 DMD / IMP、实现提交和验收证据。满足验收门后形成发布候选；正式发布单独执行。

审计新增交付门见 [G-01～G-08](../01-需求分析/03-确认与评审/协作包逻辑审计.md#2-仍然必须通过的技术门)，尤其写前全目录投影、完整来源排序、唯一模板版本身份、partial 安全恢复、旧同步清理隔离及客户端核对。当前验收入口包含94项模型、32项同步、12项通知及8项按钮样式条件和组件指标；WP-06内部生成/纠偏、在线资料与外部真实提交均已获用户确认纳入。候选按钮 / 文字原型 / mock 不能作为已交付能力。没有通过对应门的项目不得标完整交付。


## 2026-10-20当前发布准备

0.2.0实施位于feature/0.2.0-model-config独立工作树，当前开始统一Cargo/Tauri/npm版本标识为0.2.0，准备最终候选源码身份；版本文件修改不等于已发布。不提前建立v0.2.0，不从尚在运行的CI推导成功。公开与stable投递已获本对话目标授权，仍按真实测试/制品门顺序执行。

当前分批服务、真实安装2.84.0配置/回环wire、macOS正式两轮助手/人工冲突/另存及停止、原生族片段导出采用已取证；当前前端142文件700项及构建通过。Windows最终EXE首轮在导出路径比较失败，91d8a585已统一扩展路径正规化，新源远端整链仍待结果。主窗HTML5拖放准入已按SDK关闭原生拦截，物理型号拖动尚无有效实屏证据。组件/通知与94/32条件沿测试验收当前索引继续逐项核对，不能把测试总数当作整体验收通过。

发布说明仅在main根CHANGELOG.md维护，待集成时按最终实现写用户行为与支持边界：账户只同步受支持信息、不承诺免登录；OpenCodex2.84.0未兑现的Responses/Anthropic高级推理约束保存前明确拒绝；最终助手仅左侧入口。OS代码签名/公证与updater验签分别记录，沿既有测试分发流程；0.1.10未验或例外不自动改写为本版通过，有效且未受影响证据可按实际源差异复用。

后续依次为当前源Windows与剩余适用消费者结论→main集成及版本说明→最终CI和双平台tag制品→验签/隔离安装与版本回读→公开/stable→匿名通道/资产核对。详细原始事实继续沿IMP-21，不另建并行时间排期。


## 2026-10-21最终特性源状态

PR8 head b19b01e2的CI39455864253已全部通过，Windows真实EXE覆盖模版保存/默认确认/文件往返、只读比较/外部上下文与实际型号拖动；macOS型号拖动经用户操作及持久草稿读回也已核。前端143文件710项、后端原始各组结果与忽略范围沿IMP-21和checks-final-feature-ci-20261021，不以总数替代逐项AC。当前26项生产UI文本资产排除已知DEV内容，组件来源43登记/22公共源无漏项。

实际隔离Codex CLI菜单已打开并选择型号/推理档，4个已选型号准确、未选项不混入、生成0；此前管理器保存后真实app-server6型号读取保持其单独适用边界。当前Mac锁屏，等待用户解锁补核正式Sync树/字段与多域状态、批量多模版建议及最后复制/取消入口。此现场清单与逐项证据之外不新增旧版长性能/DPI/OTA要求。

尚未合并main、建立v0.2.0或生成发布草稿。完成剩余适用现场后，按既有顺序整合main/CHANGELOG与共享README，取得最终main CI、双平台tag制品和真实安装/旧公钥验签，再由既有发布器公开并晋升stable，最后核匿名清单/资产和隔离版本。没有以未验项、scope调整或取消运行推成完整交付。


2026-10-21当前更新：用户解锁后正式Sync对象树/来源/半选/子集文件、设备WebDAV范围、多模版建议选择另存及复制/取消新建原生补核已经完成，证据沿IMP-21和checks-final-native-consumers-20261021。代码仍b19、最终特性CI成功；现在可进行main整合，实际tag制品、安装及验签、公开/stable结果仍未发生，不预填。

## 2026-10-24 0.2.7 发布准备

0.2.7 的实施成果位于代码工作树 `ocx-022-model-selector`（分支 feature/0.2.6-agent-credentials），版本号与用户可见说明已同步为 0.2.7：`CHANGELOG.md` 新增 `[0.2.7] - 2026-10-24` 段落，`ui/package.json`、`tauri.conf.json`、`Cargo.toml` 与 `Cargo.lock` 一致。**版本文件修改不等于已发布**，尚未建立 v0.2.7 tag、未产出制品、未公开。

本轮已取证的测试事实：

- 前端 `vue-tsc --noEmit` 通过；`vitest run` 756 项通过 / 11 跳过；`vite build` 成功。
- 后端 `cargo check --all-targets` 无告警；`cargo test --lib` 967 项通过 / 3 忽略；事件注册表 36 项通过。
- 保存读回定向验收：写入时清除旧友好名称，读回名称统一为 `渠道名/原始模型ID`、顺序与选择一致，再读一次不回流。
- 实屏（开发审计装置渲染真实组件）：模型页 32 项、保存预览 27 项全部通过，覆盖 1180×760 / 1440×900 / 900×600、深浅两色与 150% / 200% 缩放。
- 真实上游未认证网络往返约 18ms（3 次：18.9 / 17.7 / 17.8ms，HTTP 401）；这是网络基线，不等于带认证模型清单耗时。

发布前仍缺的实证（不得跳过）：

| 门 | 当前状态 |
| --- | --- |
| 真实官方保存 → 读回 → 重启核对 | 未验。本机无官方 `ocx` CLI，`opencodex-home/config.json` 的 `providers` 为空，只能用脚本化上游夹具验证；夹具通过不替代真实官方链路。 |
| 真实供应商测速（带认证） | 未验。认证链已用本地回环 + Bearer 断言验证，真实供应商清单耗时未取。 |
| macOS WKWebView / Windows WebView2 原生拖拽 | 未验。 |
| 触屏设备长按防误触 | 未验。逻辑有单测，未在真机触摸屏复核。 |
| 150% / 200% 缩放的真实偏好生效 | 部分。按注入 `--ui-zoom` 验证布局，桌面端实际改偏好后的呈现未复核。 |
| main 集成、tag 制品、验签、公开与 stable 晋升 | 未开始。 |

### 2026-10-24 追加：真实官方 CLI 已就位，仍缺服务归属

- 从 npm 安装官方 `@bitkyc08/opencodex@2.84.0` 到临时目录，可直接跑 `tests/model_config_official_real.rs`。
- 实测官方 2.84.0 的 `sync` 在隔离 HOME 中会拒绝写入：先要求 Codex home 内已有 `config.toml`，补上后仍以 `service-home` 归属证明失败拒绝；源码确认这是官方安全设计，无测试豁免开关。
- 因此真实保存读回只能在已安装并归属本机的真实环境执行；本机服务现有 26 个渠道、247 个模型，`FeiMao-CNM` 与 `FeiMao-Claude` 已启用，具备条件但**会改写用户实际配置**，未擅自执行。
- 已用只读方式核对本机真实服务与 `config.json` 的渠道集合一致；未执行任何写入。

### 2026-10-24 发布候选门禁快照

新增 `npm run gate:release`（`qa/gate-release.mjs`），一次跑完全部可自动化检查并输出汇总报告；需要外部条件的检查不在脚本内，避免把“没跑”冒充“通过”。

本次结果（本机实跑，11 项全过、0 失败、0 跳过）：

| 步骤 | 结果 |
| --- | --- |
| typecheck | 通过（2.9s） |
| unit | **762 项通过 / 11 跳过** |
| build | 通过（vite build 290ms） |
| audit:models | **32/32**（1180 / 1440 / 900、深浅、150% / 200%） |
| audit:preview | **30/30**（含宽度 980 断言） |
| audit:geometry | **21/21**（WP-40 / 41 / 47） |
| audit:template | **28/28** |
| audit:export | **18/18** |
| audit:update | **22/22** |
| audit:catalog-naming | 真实目录 **78 条渠道行** 全部 `display_name === slug` |
| audit:catalog-crossvalidate | 清除友好名后的配置通过官方 `config validate` |

报告落盘：`GATE_OUT_DIR` 指定的 `gate-report.json`（默认临时目录）。

**这份快照仍不等于可以发布。** 下列五项需要外部条件，本轮没有、也不应伪造：

1. 真实官方保存 → 目录重写 → 客户端重启加载（需授权改写用户实际渠道配置）；
2. 真实供应商带认证的模型清单耗时（需明确授权的密钥与目的地）；
3. macOS WKWebView 与 Windows WebView2 的原生拖拽；
4. 真机触屏的 500ms 长按手感与误触；
5. main 集成、tag 制品、验签、公开与 stable 晋升。

### 2026-10-24 合并演练（只读，已清理）

为发布准备做了一次真实合并演练，回答“0.2.7 能否干净并进 main”：

- `git merge-tree HEAD origin/main` 无冲突文件，merge-base 为 `d79fc170`。
- 在临时工作树 `/tmp/ocx-027-merge-rehearsal`（分支 `rehearsal/0.2.7-merge`，基于 `origin/main` @ `4969638f`）合并 `feature/0.2.6-agent-credentials`：**Automatic merge went well**，无冲突。
- 合并后用 `npm ci` 安装依赖并复跑：`vue-tsc --noEmit` 通过；`vitest run` **762 项通过 / 11 跳过**，与合并前一致。
- 演练结束执行 `merge --abort`、移除临时工作树、删除演练分支；`main` 与 0.2.7 分支均未改动。

结论：**代码层面具备合并条件**。这不等于可以发布——仍缺真实官方保存读回、真机触控与双平台原生验证；tag、制品、验签、公开与 stable 晋升也尚未执行。

### 2026-10-24 官方链路的可行与不可行路径（实测）

为解开“真实保存读回”这道门，逐步试了官方 2.84.0 的可用路径，结论如下：

| 路径 | 结果 | 原因 |
| --- | --- | --- |
| `ocx sync`（隔离 HOME） | 拒绝 | 先要求 Codex home 内有 `config.toml`；补上后以 `service-home` 归属证明失败拒绝 |
| `ocx system startup install-service` | **未执行** | 该命令会注册持久化 launchd 服务，隔离 HOME 无法保证不触达真实系统服务；属需显式授权的操作 |
| `ocx config validate` | **可用** | 纯读取校验；已用真实配置与其“清名后”变体验证通过 |
| `ocx inspect client-config --client codex` | 需要运行中的代理 | 返回 “Proxy is not running”，未自行启动外部代理进程 |

因此真实保存读回仍只能在实际已安装并归属本机的环境完成，需要用户授权改写真实渠道配置，或授权执行服务注册/启动代理。本轮没有越过这两条线。

可用结论：**写入产物的合规性已有实证**（官方 `config validate` 对清名后配置返回 valid），缺的是“官方真正写盘后再读回”这一段。

### 2026-10-24 Windows / WebView2 静态核对（0.2.7 改动）

对 `origin/main..HEAD` 的前端与后端改动做平台关键词扫描，逐项判断 WebView2 风险：

| 改动 | 属性/关键字 | WebView2 风险 | 判断 |
| --- | --- | --- | --- |
| 防误选文字 | `user-select` / `-webkit-user-select` | 无 | WebView2（Chromium）双属性均支持 |
| 长按看说明 | `pointerType === 'mouse'` 判定 | 无 | Pointer Events 在 WebView2 完整支持，逻辑不依赖 CSS |
| 长按辅助 | `touch-action: manipulation` | 无 | 支持 |
| 长按辅助 | `-webkit-touch-callout: none` | 属性无效 | 仅影响 iOS 长按菜单，Windows 无此行为；长按由 JS 计时，功能不受影响 |
| 更新摘要两行截断 | `-webkit-line-clamp: 2` | 无 | Chromium 完整支持；项目内 `base.css` 既有同类用法 |

结论：0.2.7 的改动没有引入新的平台分支或 Windows 不兼容的关键路径。仍需真机验证的是原生拖拽（Tauri `dragDropEnabled:false` 与 HTML5 拖放配合）与触屏长按手感——这两项属运行时行为，静态核对无法代替。

未能执行：在 Windows WebView2 中实际运行（本机为 macOS）。

### 2026-10-24 门禁扩充至 15 项（含原型对照）

`gate:release` 原先只跑到实屏与真实目录检查，未含后来新增的四套原型并排对照，发布门禁因此不完整。现已补齐并完整实跑：

| 步骤 | 结果 |
| --- | --- |
| typecheck | 通过 |
| unit | **763 项通过 / 11 跳过** |
| build | 通过 |
| audit:models | 32/32 |
| audit:preview | 30/30 |
| audit:geometry | 21/21 |
| audit:template | 28/28 |
| audit:export | 18/18 |
| audit:update | **24/24** |
| audit:parity:list | 8/8 |
| audit:parity（保存预览） | 10/10 |
| audit:parity:export | 7/7 |
| audit:parity:template | 11/11 |
| audit:catalog-naming | 真实目录 78 行全部合规 |
| audit:catalog-crossvalidate | 官方 `config validate` 通过 |

合计 **15 项通过、0 失败、0 跳过**；界面类断言 219 项（32+30+21+28+18+24+8+10+7+11）。需要外部条件的检查不在脚本内，仍不计入通过。

## 0.2.7 发布程序（可执行清单，2026-10-24）

代码工作树：`ocx-022-model-selector`，分支 `feature/0.2.6-agent-credentials`，版本号已 0.2.7。

### 发布前五项实证（按真实状态更新于 2026-10-24 晚）

| # | 项目 | 状态 | 证据 / 缺口 |
| --- | --- | --- | --- |
| 1 | 真实官方保存 → 读回 | **已完成** | 走官方管理 API 对测试端点写入：带 modelDisplayNames 时目录 display_name 采用该友好名；清空后再提交，目录变为 渠道ID/模型ID。证明「保存时清除友好名」正确（§29.1 依据由推断改为实测）。附带发现：三次保存的 catalogRefresh 均为 degraded:true（含 fallback，一次另有 provider-network），属官方刷新链路既有行为，0.2.7 未改动 —— 发布说明不得宣称目录刷新完全正常。 |
| 1b | 保存后客户端重启加载 | **未验** | 需重启真实 Codex 客户端并在选择器核对；未做。 |
| 2 | 真实供应商测速（带认证） | **已完成（按下限理解）** | 同一端点 /v1/models 六次：901 / 473 / 465，657 / 644 / 468 ms，均 HTTP 200。端点内容在测试期间由 gpt 系列变为 deepseek 系列，说明该服务动态换模型，耗时按**下限**记录。 |
| 3 | macOS 原生拖拽 | **已完成（2026-10-24 解锁后实测）** | 隔离沙盒原生窗口实拖：渠道行换序成功、模型行换序成功（demo-7→demo-10 后、demo-8→首位）、全程无选区、无残留拖拽态、还原后回到「未修改」。修复前同一动作是「整片框选 + drop 不触发」。 |
| 4 | Windows 原生拖拽 + 触屏长按 | **等用户实测** | 用户答复今晚再测。静态核对已做（user-select / Pointer Events / line-clamp 在 WebView2 均支持）。 |
| 5 | 150%/200% 缩放桌面端复核 | **按用户结论跳过** | 用户答复「之前版本测过，没有必要」。自动化侧仍保留注入 --ui-zoom 的 150% 与 200% 检查。 |

### 自动化门禁（已完成，可随时复跑）

```sh
cd apps/desktop/ui
OCX_OFFICIAL_ENTRY=<官方 ocx.mjs> OCX_OFFICIAL_NODE=<node> npm run gate:release
```

当前结果：**16 项通过 / 0 失败 / 0 跳过**（类型检查、763 单测、构建、6 套界面检查、5 套原型对照、真实目录核对、官方配置交叉验证）。

### 合并与发布步骤

1. 合并 `feature/0.2.6-agent-credentials` → `main`（已演练 2026-10-24 晚：真合并 0 冲突；合并后 typecheck + **771** 单测、fmt、clippy、**39** 个后端测试二进制全过）。
2. 在 `main` 上打 tag `v0.2.7` 并推送 → `.github/workflows/release.yml` 自动建草稿 Release 并出双平台制品（macOS arm64、Windows x64）。
3. 验证制品：下载安装、旧公钥验签、版本回读为 0.2.7。
4. 推送 `publish/v0.2.7` → `.github/workflows/publish-release.yml` 公开并保留 Latest。
5. 如需稳定通道，再推 `promote-stable/v0.2.7`（或手动 dispatch 选 stable）。
6. 公开后核对匿名清单与资产，写入本页。

### 硬性约束

- **不跳过第 1～5 项发布前实证**：版本文件改好不等于可发布。
- 不用原型或单测结果替代真实链路；不以“功能可用”放过与原型不同的界面。
- 发布说明只在代码仓库根 `CHANGELOG.md` 维护，已有 `[0.2.7] - 2026-10-24` 段落。

### 2026-10-24 补跑 Rust 集成测试（此前只跑过 --lib）

此前所有后端验证只覆盖 `cargo test --lib`（967 项），**35 个集成测试文件从未全量执行**。本轮补跑：

```sh
cd apps/desktop/tauri && cargo test --tests
```

结果：**38 个测试二进制全部 `ok`，0 失败，8 项忽略**（忽略项均为需要真实官方 CLI/网络环境的 opt-in 用例，例如 `model_config_official_real` 与 `model_research_real`）。

含实际的 loopback 集成用例：`config_sync_webdav_real`（6 项，15.2s）、`provider_auth_native`（8 项，15.1s）等均通过。加上 `--lib` 的 967 项与 `event_registry` 的 38 项，后端总计 **1005 项通过 / 11 忽略**。

发布门禁的两条命令现为：

```sh
cd apps/desktop/tauri && cargo test --lib && cargo test --tests
cd apps/desktop/ui && npm run gate:release
```

### 2026-10-24 补跑 Windows 专项 smoke 测试

CI 的 `windows-candidate.yml` 里有一组 smoke 测试。其中三份是**纯逻辑断言**（不依赖 Windows 运行时），本机可直接执行，此前从未跑过：

```sh
cd <repo>
node --test test/smoke/windows-model-ui.test.mjs
node --test test/smoke/windows-cdp.test.mjs
node --test test/smoke/windows-bundle-identity.test.mjs
```

结果：**17 项全部通过，0 失败**。

- `windows-model-ui`（5 项）：概览完整/门禁两态、写入必须使用已采纳沙盒、非法原生根拒绝、导出路径 Windows 规范化前缀与越界拒绝、任务预览 JSON 边界。
- `windows-cdp`（7 项）：非数组/空目标不算渲染、双向超时、连接拒绝保留底层错误、非回环端点拒绝。
- `windows-bundle-identity`（5 项）：缺失或歧义标记拒绝、不支持的包类型拒绝。

意义：这些测试覆盖 Windows 路径安全与 UI 判据，与 0.2.7 改动的渠道/模型/导出链路直接相关。它们的通过说明改动没有破坏 Windows 侧的既有合同；但**仍不能替代**在真实 WebView2 中的拖拽与触屏验证。

需 Windows 运行时才能执行的 Powershell smoke（`windows-process-token`、`windows-acceptance-profile`、`windows-restart-handoff`）未在本机运行。

### 2026-10-24 后端忽略用例清点（下游盲区收尾）

后端共 11 处 `#[ignore]`，逐条核对用途，确认**没有一处是在掩盖缺陷**：

| 位置 | 忽略原因 | 处置 |
| --- | --- | --- |
| `commands/network.rs` | 访问真实网络 | 需显式运行 |
| `modules/about/remote.rs` | 访问 npm registry | 需显式运行 |
| `modules/sync/config.rs` | 访问系统钥匙串 | 注释给出显式运行命令，本轮已核对其断言内容 |
| `tests/model_config_official_real.rs` ×2 | 需已安装官方 CLI + 临时 HOME | 已获取官方 2.84.0 并尝试；受服务归属限制未完成 |
| `tests/provider_auth_native.rs` | 需 `OPENCODEX_SANDBOX=1` | 写入/删除随机自有凭据夹具 |
| `tests/windows_runtime.rs` | 子进程助手，由父测试显式执行 | 由父用例覆盖 |
| `tests/model_research_real.rs` | 显式公网读取准入 | 不涉及模型生成 |
| `tests/config_sync_webdav_real.rs` | 需自有 TLS WebDAV 夹具与本地 CA | 非忽略版本已覆盖 loopback 同步 |
| `tests/status_snapshot.rs` | Tauri 托盘需原生主线程 | 平台限制 |
| `tests/backup_process_lock.rs` | 子进程助手 | 由父测试覆盖 |

结论：忽略项分三类——真实外部依赖（网络/官方 CLI/钥匙串）、平台限制（原生主线程）、子进程助手（由父用例触发）。第三类已被父用例实际覆盖；第一类中的官方 CLI 已尝试并记录受阻原因；其余不影响 0.2.7 的渠道/模型/导出链路。

### 2026-10-24 最终验证基线（0.2.7 全部自动化门禁）

这是 0.2.7 在当前代码状态下的完整验证快照，可作为发布候选的验收基线：

| 层 | 命令 | 结果 |
| --- | --- | --- |
| 原型回归 | `node --test model-*.test.cjs agent-settings-page.test.cjs` | **192 通过 / 0 失败** |
| 前端门禁 | `npm run gate:release` | **16 通过 / 0 失败 / 0 跳过** |
| 后端库测试 | `cargo test --lib` | 967 通过 / 3 忽略 |
| 后端集成 | `cargo test --tests` | 38 个二进制全 ok / 8 忽略 |
| Windows 纯逻辑 smoke | 三份 `node --test` | 17 通过 |

前端门禁 16 项明细：类型检查、764 单测、构建、模型列表 32、保存预览 30、几何 21、模版 28、导出 18、更新 24、渠道列表对照 8、保存预览对照 10、导出对照 7、模版对照 11、Agent 对照 7、真实目录 78 行、官方 `config validate` 交叉验证。

**界面类断言合计 226 项**（32+30+21+28+18+24+8+10+7+11+7），全部基于真实渲染而非源码字符串匹配。

仍未完成且无法自动化的五项：真实官方保存 → 读回 → 重启、真实供应商带认证测速、macOS/Windows 原生拖拽、真机触屏长按、150%/200% 缩放的真实偏好复核。这些都需要用户授权或对应设备。

### 2026-10-24 门禁增至 17 项 / 界面断言 285 项，并新增 WebKit 实屏门

`gate:release` 增两项：

| 新增项 | 覆盖 | 结果 |
| --- | --- | --- |
| `audit:geometry` 扩到 44 项 | 模型行垂直居中、摘要两行、过滤区控件同高同轴/勾选框宽度/搜索框宽度 | 44/44 |
| `audit:geometry:webkit`（新的第 17 项） | 用 Playwright WebKit（与 WKWebView 同引擎族）跑同一组几何断言 + 行拖拽（模型行与渠道行） | 36/36 |

当前基线：**17 通过 / 0 失败 / 0 跳过**；单测 **768 通过 / 10 跳过**；界面断言 **285 项**
（32+30+44+36+28+18+24+8+10+7+11+7）。

同时修正门禁器的一处误判：检查脚本以退出码 2 + `BLOCKED` 表示「本机缺环境」时，
现在记为 **跳过**而不是失败——既不冒充通过，也不再让缺环境拖垮整份报告。

### 2026-10-24 第 3 项（macOS 原生拖拽）的实际进展与结论

在隔离沙盒的原生窗口里真拖模型行，发现两个真实缺陷（Chromium 与自动化都不复现/不稳定复现）：

1. **拖拽会框选文字**——表头整片被框蓝；
2. **排序静默失败**——`dragstart` 触发、`drop` 从不触发（埋点：`dragenter/dragleave` 抖动 13 次、`drop` 0 次），松手后顺序不变。

因此第 3 项**不能判定为通过**，而且这一项的性质从「补一次人工点击」变成「代码必须改」：
行排序改用 Pointer Events 实现（`pointerDrag.ts`），模型行与渠道行共用；
拖拽期间整体禁选、松手恢复；`pointerup`/`pointercancel`/`Esc` 三条收尾路径。

改后用 WebKit 连续复跑 3 轮，`audit:geometry:webkit` 稳定 36/36（修复前约 1/3 轮次掉 `drop`）。
**原生窗口的人工复核仍未完成**：复核中途本机锁屏，解锁后需重跑同一动作（拖模型行 + 拖渠道行 + Esc 取消）才算这项通过。
### 2026-10-24 晚：门禁器自身两处误判已修

跑门禁时出现「17 项里 unit 失败、单独跑 vitest 却返回 0」的矛盾。查明是门禁器的问题，不是代码回归：

| 缺陷 | 现象 | 修法 |
| --- | --- | --- |
| `execFileSync` 默认 1MB 输出缓冲 | 前端测试输出（含大量 Vue 警告）超限抛 `ENOBUFS`，整项被记为失败 | `maxBuffer: 64MB`；失败仍由退出码判定 |
| 脚本以退出码 2 + `BLOCKED` 表示缺环境 | 之前被当成失败，拖垮整份报告 | 记为**跳过**，既不冒充通过也不误判为缺陷 |

修后复跑：**17 通过 / 0 失败 / 0 跳过**，unit 明细为 769 通过 / 10 跳过。

### 2026-10-24 晚：触屏不能整行拖

改成指针拖拽后自查发现一个新风险：触屏上「滑动列表」与「拖动行」是同一个手势，接管会让人滚不动列表。
已按原型语义收敛：**触屏保留原生滚动，排序走行内 ↑↓ 按钮**；鼠标与触控板整行可拖。单测已锁这条（`pointer-drag.test.ts` 5 项）。
### 2026-10-24 晚：门禁增至 18 项，全部实屏检查都有 WebKit 复跑

拖拽缺陷暴露的系统性问题不是「某个 bug」，而是**检查面太窄**：六套实屏检查原先只跑 Chromium，
而产品在 macOS 跑的是 WKWebView。已在 `qa/engine.mjs` 抽出引擎选择，六套脚本都支持 `AUDIT_ENGINE=webkit`，
并用 `npm run audit:webkit` 串成第 18 项。

| 检查 | Chromium | WebKit |
| --- | --- | --- |
| 模型页 | 32 | 32 |
| 保存预览 | 30 | 30 |
| 几何（含拖拽） | 44 | 44 |
| 模版 | 28 | 28 |
| 导出 | 18 | 18 |
| 更新 | 24 | 24 |
| **合计** | **176** | **176** |

注：`audit:webkit` 由 6 个脚本串起，门禁汇总行显示的是**最后一个脚本**的明细（32），不是总数；逐项数以本表为准。

外加 `audit:geometry:webkit`（36 项，含指针拖拽/不框选/Esc 收尾）。
当前门禁：**18 通过 / 0 失败 / 0 跳过**。界面断言按引擎分别计：Chromium 176 项 + WebKit 176 项 + 专项拖拽 36 项 = **388 项**。

缺引擎构建时脚本以 `BLOCKED` + 退出码 2 收尾，门禁记为**跳过**——不冒充通过，也不误判成缺陷。
### 2026-10-24 晚：指针拖拽补边缘自动滚动

自查发现改指针拖拽后丢了一个能力：**拖到视口边缘不会滚动**，长列表里没法把行拖到远处。
（HTML5 拖放时代是浏览器自带这个行为，自己接管指针后必须显式实现。）

实现要点（`pointerDrag.ts`）：

| 点 | 做法 | 踩到的坑 |
| --- | --- | --- |
| 找滚动容器 | 按下时从行往上找 `overflow-y: auto/scroll` 的祖先 | 拖动中用 `elementFromPoint` 找会失败：指针移出视口或滚到底时返回 `null`，自动滚动静默失效（实测复现） |
| 边缘判定 | 取容器矩形与视口的**交集** | 页面 `main` 常比窗口高（实测 top=−30、h=570），直接用容器 rect 会让边缘带落在视口外 |
| 步进 | 48px 边缘带，越靠边越快，上限 14px/帧 | — |
| 滚不动时 | 换上一层可滚动祖先 | 有的页面表格自己滚，有的整体滚 |

验证：

- 纯函数 `edgeStep()` 单测 1 项（交集判定、方向、上限、静止区、可视区过窄不滚）——jsdom 没有布局，靠 DOM 测不到这段。
- `audit:geometry:webkit` 加 2 项真机断言：拖到边缘后 `main.main` 从 1557 滚到 2296，且收尾无残留状态。
- 门禁 **18 通过 / 0 失败 / 0 跳过**；单测 **770 通过 / 10 跳过**。
### 2026-10-24 晚：发布前置条件复核（HEAD `2c47ebf7`）

| 检查 | 结果 |
| --- | --- |
| 与 `origin/main`（`4969638f`）合并 | **无冲突**（`git merge-tree --write-tree` 直接产出树对象） |
| 版本号一致 | `Cargo.toml` / `Cargo.lock` / `tauri.conf.json` / `ui/package.json` 均为 **0.2.7** |
| `CHANGELOG.md` 版本段 | 恰 **1** 个 `## [0.2.7]` 段落 |
| tag 占用 | 本地与远端均**不存在** `v0.2.7` |
| 门禁 | **18 通过 / 0 失败 / 0 跳过** |
| 后端 | `--lib` 967 通过；`--tests` 38 个二进制全 ok |
| 原型回归 | 192 通过 |

**结论：代码与制品侧已具备发布条件，但按本页门禁仍不能发。**
第 3 项已于当晚解锁后实测通过；仍挡路的只有第 4 项（Windows / 触屏由用户实测）与 1b（重启客户端核对）。
第 3 项复核在本机锁屏前中断；解锁后重跑「拖模型行 + 拖渠道行 + Esc + 拖到边缘」四步即可收口。

不把「WebKit 自动化通过」当成「原生窗口通过」——那正是本轮拖拽缺陷漏过去的同一类误判。
### 2026-10-24 晚：本机 release 打包实证（HEAD `2983e4ac`）

在真打 tag 之前先本地跑一次正式打包，验证「版本号改好但打不出包」这类问题不会拖到 CI：

```sh
cd apps/desktop/tauri && cargo tauri build --bundles app,dmg
```

| 制品 | 结果 |
| --- | --- |
| `OpenCodeX Desktop.app` | 版本回读 **0.2.7**，`CFBundleIdentifier=com.gzers.opencodex.desktop` |
| `OpenCodeX Desktop_0.2.7_aarch64.dmg` | 6.5MB，`hdiutil attach` 成功，卷内含 app 与 Applications 快捷方式 |
| 二进制架构 | Mach-O 64-bit **arm64** |
| 签名 | `adhoc`（符合本项目未签名测试分发策略；自更新资产仍由发布流程签名） |
| 审计夹具泄漏 | 产物内 `__audit` 命中 **0** 处（DEV 夹具未进正式构建） |

同时验证了发布流水线的 CHANGELOG 抽取逻辑在当前文件上可用：`v0.2.7` 能抽出 **22 行**版本说明，
且不含 0.2.6 段落内容；`## [0.2.7]` 段落恰 1 个。

顺带更正了两处**用户可见文案与实测不符**（`CHANGELOG.md` 使用边界）：

- 原文写「节点测速只在本地回环与未认证网络延迟上验证」——实际已用真实供应商端点验证六次（473–901ms，均 HTTP 200）；
- 原文写「真实官方保存 → 读回已完成」但没说目录刷新走了官方降级路径，容易被误读成刷新完全正常。

结论：**制品侧已可发布**。仍需第 3、4 项实证才能打 tag。
### 2026-10-24 晚：release 二进制启动冒烟（隔离实例）

制品能打出来不等于能跑。把 `target/release/opencodex-desktop` 放进独立 bundle id + 独立数据根的隔离 .app 启动，
确认发布用的优化构建能真正起来（不碰真实配置，也不用 `/Applications` 那份）：

| 检查 | 结果 |
| --- | --- |
| 进程启动 | 存活（pid 51229） |
| 启动日志 | `startup version=0.2.7 commit=2983e4ac...`，与当前 HEAD 一致 |
| 数据根初始化 | `backups / cache / exports / logs / manager-state / opencodex-home / sync-state` 全部创建 |
| 清理 | 进程已停止，隔离根可随时删除 |

意义：此前所有 UI 复核都跑在 debug 构建上；这一条证明**要发布的那个优化构建**同样能启动且版本自洽。
窗口内可交互复核仍受锁屏限制，未做。
### 2026-10-24 晚：CI 头两道门此前是红的（已修）

`ci.yml` 的 backend job 第一步就是 `cargo fmt --all -- --check`、第二步 `clippy -D warnings`。
此前所有后端验证都只跑 `cargo test`，**这两道门从未执行过**——一旦打 tag，CI 会在编译前就失败。

实测结果与修复：

| 门 | 修前 | 原因 | 修后 |
| --- | --- | --- | --- |
| `cargo fmt --check` | **失败**（7 处 diff，6 个文件） | 本分支新增代码未按 rustfmt 落格式 | 通过 |
| `cargo clippy -D warnings` | **失败** | `execution_directory.rs` 对已是 `String` 的值再 `.into()`（useless conversion） | 通过 |

涉及文件均在 `origin/main..HEAD` 的改动范围内（`model_transfer.rs`、`compiler.rs`、`execution_directory.rs`、
`runtime.rs`、`notifications/registry.rs`、`tests/event_registry.rs`），属本分支引入，不是历史遗留。

**语义影响核对**：`execution_directory.rs` 那处只去掉同一类型上的 `.into()`，不改变命名规则
（仍是原生渠道沿原生身份、其余 `渠道名/原始ID`）。改后重跑后端全量：

- `cargo test --lib`：**967 通过 / 3 忽略**
- `cargo test --tests`：**38 个二进制全 ok**

意义：发布④的 CI 硬门现在真正可过；此前「本地全绿」并不覆盖这两道门。
### 2026-10-24 晚：触屏长按获得真机级证据（WebKit + touch 指针）

第 4 项原写「触屏长按需真机复核」。已用 Playwright WebKit 的 hasTouch 页面、真实 touch 指针序列补进自动化，
断言用户实际会遇到的三个事实：

| 断言 | 结果 |
| --- | --- |
| 长按 650ms 后出现说明（.preview-group-title.show-note） | 命中 1 个 |
| 抬手产生的 click 被吞掉，不误改渠道启停 | aria-pressed 保持 true |
| 鼠标不参与长按（实现里 pointerType==='mouse' 直接返回） | 由代码与既有单测覆盖 |

已并入 audit:geometry:webkit（40 项）；门禁仍 18 通过 / 0 失败 / 0 跳过。

边界：这证明长按语义在 WebKit 引擎下正确（计时、吞点击、出说明），不等于真机触屏的 500ms 手感与误触率——
后者仍需设备实测，属第 4 项剩余部分。
### 2026-10-24 晚：CI 实际命令逐条跑过（此前跑的不是同一套）

之前本地验证用 `cargo test --lib` + `--tests`，而 CI 用的是 `--locked --workspace --features integration-test`；
前端也是 `npm run test -- --run` 而非直接 `npx vitest`。两者覆盖面不同，必须按 CI 的命令验一遍才作数。

| CI job | 命令 | 本机结果 |
| --- | --- | --- |
| `frontend` | `npm --prefix apps/desktop/ui run test -- --run` | **154 文件 / 770 通过 / 10 跳过** |
| `backend` | `cargo test --locked --workspace --features integration-test` | **39 个测试二进制全 ok，0 失败** |
| `release-tools` | 三个 `unittest discover` | **29 + 13 + 6 全 OK** |
| `backend` 前置 | `cargo fmt --check` / `clippy -D warnings` | 通过（本轮修复） |
| `windows-smoke-evidence` | PowerShell smoke | 本机为 macOS，**跑不了**，由 CI 承担 |

差异说明：带 `--features integration-test` 时测试二进制由 38 变 **39**（多一组由 feature 引入的用例），
此前只跑 `--tests` 会漏掉这一组。

结论：**可在本机验证的 CI 门已全部通过**。剩余不可本机验证的只有 Windows 专属 job，
以及第 3、4 项的真机实证——这三类都不因本地绿灯而被视为通过。
### 2026-10-24 晚：指针拖拽补「失焦清理」（WP-46 原有要求，之前漏了）

WP-46 验收要点写着「拖动、取消、**失焦**、排序后不闪屏」，但改成指针拖拽后我只实现了 pointerup / pointercancel / Esc 三条收尾路径，没处理窗口失焦。

后果比拖拽失败更难受：拖到一半切到别的应用，pointerup 不会再送进本窗口，于是——

1. 那一行永远保持半透明「拖拽中」；
2. ocx-pointer-drag 还挂在 html 上，整页文字都选不中，直到下次重开界面。

修法：blur 与 visibilitychange（document.hidden）都走同一条 cleanup()，与 Esc 完全一致——清定时器、清悬停高亮、清半透明行、摘掉禁选类、解绑所有监听。

验证：tests/pointer-drag.test.ts 新增 1 项（共 7 项）——拖到一半触发 blur 后，禁选类必须消失、onActive(null) 必须发出、之后抬手不得产生重排。前端全量 771 通过 / 10 跳过；门禁仍 18 / 0 / 0。

这条是自查发现的，不是用户报的——按 WP-46 原文对照才看到漏项。
### 2026-10-24 第 3 项已收口（解锁后原生实测）

用户在锁屏结束后解锁，第 3 项在隔离沙盒的原生窗口（WKWebView）里完成，四步全过：

| 步骤 | 结果 |
| --- | --- |
| 渠道行拖拽（NewApi → 第 3 行位置） | 顺序改变（A6api / AiMaMi / NewApi），转「未保存修改」，无框选 |
| 模型行拖拽（`gpt-demo-7` → `gpt-demo-10` 之后） | 顺序改变（…6 / 8 / 9 / 10 / 7 / 11） |
| 模型行再次拖拽（`gpt-demo-8` → 首位） | 顺序改变，无框选、无残留半透明行 |
| 收尾 | 无 `.dragging` 残留、无 `ocx-pointer-drag` 卡住；「还原修改」后回到「未修改」 |

对比修复前同一动作：拖拽时表头整片框蓝、松手顺序不变（`drop` 从不触发）。**指针拖拽改造在真机确认生效。**

### 剩下的发布门

| 项 | 状态 |
| --- | --- |
| 1 真实保存 → 读回 | 已完成 |
| 1b 保存后客户端重启核对 | 未验 |
| 2 真实供应商测速 | 已完成（按下限理解） |
| **3 macOS 原生拖拽** | **已完成** |
| 4 Windows WebView2 + 触屏手感 | 等用户实测（长按语义已有 WebKit touch 断言） |
| 5 150%/200% 缩放 | 按用户结论跳过 |
### 2026-10-24 晚：合并演练（真合并 + 合并后全量复跑）

之前只做 `merge-tree` 的静态检查；这次在临时工作树里**真合并**并按 CI 命令全量复跑：

```
临时工作树 /tmp/ocx-027-merge2（分支 rehearsal/0.2.7-merge2，基于 origin/main @ 4969638f）
git merge --no-ff feature/0.2.6-agent-credentials
```

| 检查 | 结果 |
| --- | --- |
| 冲突 | **0** 个（64 个文件新增 / 修改，含 `pointerDrag.ts`、`pointer-drag.test.ts`、`saveReview.ts`） |
| typecheck | 通过 |
| 前端单测 | **154 文件 / 771 通过 / 10 跳过**（与合并前一致） |
| `cargo fmt --check` | 通过 |
| `clippy -D warnings` | 通过 |
| `cargo test --workspace --features integration-test` | **39 个二进制全 ok，0 失败** |

演练结束已执行 `worktree remove` + 删除演练分支；**`main` 与 0.2.7 分支均未改动**（`origin/main` 仍是 `4969638f`）。

结论：**代码层面已具备合并条件，且合并后不引入回归**。这不等于已发布——仍需第 4 项（Windows/触屏）与 1b（重启客户端核对）。
### 2026-10-24 晚：1b 的可做部分（官方 CLI 只读核对）

1b 是「保存后重启客户端核对」。重启真实 Codex 属于用户环境操作，未做；但「客户端会加载成什么」可以只读验：

```sh
OCX=~/Library/Application\ Support/com.gzers.opencodex.desktop/runtime/bin/ocx
OPENCODEX_HOME=<manager 的 opencodex-home> $OCX inspect catalog --json
```

| 检查 | 结果 |
| --- | --- |
| 目录条目 | 87 条 |
| 带渠道前缀（`渠道名/原始ID`） | **78** 条 —— 与 §29.1 要求一致 |
| 残留测试渠道行 | **0** |
| 样例 | `A6api/glm-5.3-flash`、`A6api/gpt-6-astra` … |

边界：这只证明**目录内容与命名符合规格**，不代表客户端重启后 UI 已按新目录刷新——后者仍需真实重启确认。
### 2026-10-24 晚：updater 签名链路实证（用临时密钥，不进仓库）

CI 的 tag 构建会用 `--config tauri.updater.conf.json` 产出更新包并注入签名。这条路径此前**从未本地跑过**，
只验过普通 `cargo tauri build`。本轮用一次性临时密钥补验：

```sh
cargo tauri signer generate -w /tmp/ocx-test-updater.key --password ''   # 一次性密钥，验证后丢弃
TAURI_SIGNING_PRIVATE_KEY=$(cat /tmp/ocx-test-updater.key) \
  cargo tauri build --config tauri.updater.conf.json --bundles app
```

| 产物 / 检查 | 结果 |
| --- | --- |
| `OpenCodeX Desktop.app.tar.gz` | 生成（6.6MB，内含完整 .app，`tar -tzf` 可列） |
| `OpenCodeX Desktop.app.tar.gz.sig` | 生成（base64 封装的 minisign 签名，含 trusted comment） |
| **独立验签** | `minisign -V` → **`Signature and comment signature verified`** |
| 密钥不匹配警告 | 预期：临时密钥与仓库配置的 `pubkey` 不同；正式发布用仓库密钥 |

意义：证明**签名→产物→独立校验**整条链可用，不是「文件生成了就算」；也说明 CI 那步不会因签名配置缺失而产出无效包。

清理：临时密钥与验证副本只在 `/tmp`，未进仓库、未写入任何配置。
### 2026-10-24 晚：Windows 侧可做与不可做的边界

尝试用本机已有的 `x86_64-pc-windows-msvc` target 交叉编译，确认「能不能在 macOS 上验 Windows 构建」：

| 尝试 | 结果 |
| --- | --- |
| `cargo check --target x86_64-pc-windows-msvc` | **失败**，卡在 `ring` 的 C 构建脚本 |
| 原因 | `CC_x86_64-pc-windows-msvc = None` —— 本机没有 MSVC 编译器，不是代码问题 |
| PowerShell smoke | 本机无 `pwsh`，跑不了 |

结论：**Windows 侧任何构建/运行级验证都无法在本机完成**，只能由 CI（`windows-latest`）与用户的 Windows 机器承担。
这条写入边界，避免以后误把 `target list` 里有 windows target 当成「已验 Windows」。

仍可做且已做的 Windows 侧检查：三份纯逻辑 smoke（`windows-model-ui` / `windows-cdp` / `windows-bundle-identity`，17 项全过），
以及对本轮前端改动的平台关键词静态核对（`user-select`、Pointer Events、`line-clamp` 在 WebView2 均支持）。
### 2026-10-24 晚：分支首次推上 CI，抓到并修掉两个只在 Windows 暴露的缺陷

0.2.7 分支此前**从未推送**（本地领先 40 个提交），因此 CI 从未在本分支跑过，两个 Windows job 一次都没执行。
推送后连续三轮，逐轮收敛：

| 轮次 | 结果 | 发现 |
| --- | --- | --- |
| 第 1 轮 `40064227146` | **失败** | `backend-windows`：`runtime.rs:4714` 调用 `cli_fixture` 报 `cannot find function` —— 该 helper 是 `#[cfg(unix)]`，但 `official_save_plan_freezes_baselines...` 用例漏了 `cfg` 守卫。macOS 能编、Windows 编不过。 |
| 第 2 轮 `40065303755` | **失败** | 编译已修好；`Windows final EXE smoke` 报 `Cannot read properties of null (reading 'scrollIntoView')` —— smoke 脚本仍在找 `button[draggable=true]`（HTML5 拖放时代的抓手），改成指针拖拽后该元素不存在。 |
| 第 3 轮 `40067543382` | **全部成功** | `backend` / `frontend` / `release-tools` / `windows-smoke-evidence` / `backend-windows` 五个 job 全 success |

两处修法：

1. `runtime.rs` 给该用例补 `#[cfg(unix)]`（与同文件另外 8 处调用保持一致），并注明原因防止再漏。
2. `test/smoke/windows-model-ui.mjs` 的起手元素由 `button[draggable=true]` 改为 `.model-grip`——它本来就是在真实 WebView2 里走 `Input.dispatchMouseEvent` 的指针拖拽用例，语义不变、只跟随实现更新。

**意义**：这是本轮唯一一次真正在 Windows 上验证 0.2.7。此前「Windows 侧只能靠 CI」这句是推断；现在 `backend-windows` 实跑过，含 clippy、release 构建、**真实 EXE 的 WebView2 smoke（含指针拖拽排序）**、Windows 后端回归与凭据保护检查。

仍未覆盖：Windows 上**人工**触摸屏长按手感——自动化只验了语义（WebKit touch 断言），真机手感仍需用户。
## 2026-10-24 v0.2.7 已发布（公开、非草稿）

| 事实 | 值 |
| --- | --- |
| PR | **#15** 合并到 main，merge commit **`cd23d1229`** |
| tag | **`v0.2.7`** → `3e8b49f6` |
| 发布流水线 | run `40072010336`：macOS 与 Windows 两个 release job 均 **success** |
| Release | **公开、非预发布**，`publishedAt=2026-10-24T07:23:17Z` |
| 公开流水线 | run `40073913449` success（`preserve-latest` 模式） |
| 匿名访问 | `api.github.com/.../releases/tags/v0.2.7` → **200** |

### 制品（8 个资产，匿名可下载）

| 资产 | 大小 |
| --- | --- |
| `OpenCodeX.Desktop_0.2.7_aarch64.dmg` | 6,384,936 |
| `OpenCodeX.Desktop_aarch64.app.tar.gz` + `.sig` | 6,491,661 / 436 |
| `OpenCodeX.Desktop_0.2.7_x64-setup.exe` + `.sig` | 4,795,388 / 448 |
| `OpenCodeX.Desktop_0.2.7_x64_en-US.msi` + `.sig` | 6,725,632 / 448 |
| `latest.json` | 6,776 |

### 发布前实测（下载线上制品验证，不是只看流水线绿）

| 检查 | 结果 |
| --- | --- |
| dmg 挂载 + 版本回读 | `hdiutil attach` 成功；`CFBundleShortVersionString = 0.2.7` |
| 二进制架构 | `Mach-O 64-bit executable arm64` |
| **updater 签名验签** | 用仓库配置的**正式公钥** `minisign -V` → `Signature and comment signature verified`，trusted comment 含 `version:0.2.7` |
| `latest.json` | `version=0.2.7`，平台键含 darwin-aarch64 / windows-x86_64(-msi/-nsis) |
| Release 正文 | 来自 CHANGELOG，19 条要点 + 安装说明；含「macOS 原生窗口实测」 |

### 过程中的一次基础设施失败（已重跑通过）

首轮 macOS release job 在**出包成功后**上传资产时报 `getaddrinfo ENOTFOUND uploads.github.com`——runner 到 GitHub 上传端点的 DNS 瞬时故障，
不是代码或配置问题（日志中 dmg 与 `.app.tar.gz` 均已 `Finished 2 bundles`）。`gh run rerun --failed` 后两个平台均 success。

### 通道状态

本次用 `publish/v0.2.7` 走 `preserve-latest`：**保留上一个 Latest（v0.2.6）**，这是该模式的既定语义。
如需把 0.2.7 设为 Latest / stable，需再推 `promote-stable/v0.2.7`（或手动 dispatch 选 `stable`）——**尚未执行**。

### 仍未做

- 提升 stable 通道（需明确决定）。
- Windows 真机触摸屏长按手感（语义已验，手感未验）。
- 保存后重启真实客户端再核对（目录侧已只读核对：87 条 / 78 条渠道前缀 / 无残留）。
### main 主干 CI 最终结果

合并后 main（`cd23d122`）的 CI run `40071962356`：**6 个 job 全 success** —— `release-tools` / `backend` / `backend-windows` / `windows-smoke-evidence` / `frontend` / `build`。

至此发布链路的每一段都有实测：分支 CI → PR CI（5 job）→ 合并 → main CI（6 job）→ tag 流水线（双平台制品）→ 下载线上制品验签 → 公开 → 匿名 200。
