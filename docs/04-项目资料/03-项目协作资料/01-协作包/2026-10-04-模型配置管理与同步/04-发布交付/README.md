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
| 3 | macOS 原生拖拽 | **缺陷已修复，原生复核待补** | 真机复核发现两个真实缺陷：拖拽框选文字、drop 从不触发（排序静默失败）。已改为指针拖拽，WebKit 连续 3 轮 36/36。原生窗口人工复核在锁屏前未完成，解锁后需重跑同一动作才算通过。 |
| 4 | Windows 原生拖拽 + 触屏长按 | **等用户实测** | 用户答复今晚再测。静态核对已做（user-select / Pointer Events / line-clamp 在 WebView2 均支持）。 |
| 5 | 150%/200% 缩放桌面端复核 | **按用户结论跳过** | 用户答复「之前版本测过，没有必要」。自动化侧仍保留注入 --ui-zoom 的 150% 与 200% 检查。 |

### 自动化门禁（已完成，可随时复跑）

```sh
cd apps/desktop/ui
OCX_OFFICIAL_ENTRY=<官方 ocx.mjs> OCX_OFFICIAL_NODE=<node> npm run gate:release
```

当前结果：**16 项通过 / 0 失败 / 0 跳过**（类型检查、763 单测、构建、6 套界面检查、5 套原型对照、真实目录核对、官方配置交叉验证）。

### 合并与发布步骤

1. 合并 `feature/0.2.6-agent-credentials` → `main`（已演练：无冲突、合并后 typecheck + 763 单测通过）。
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
挡路的只有第 3 项（macOS 原生窗口人工复核）与第 4 项（Windows / 触屏由用户实测）。
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
