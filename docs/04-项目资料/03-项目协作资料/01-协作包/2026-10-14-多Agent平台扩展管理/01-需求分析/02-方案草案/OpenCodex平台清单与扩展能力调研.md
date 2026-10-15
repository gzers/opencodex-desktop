# OpenCodex 平台清单与扩展能力调研

核实日期：2026-10-14，Asia/Shanghai。状态：第一轮外部来源与项目基线分析。

## 1. 核实口径

本轮 npm latest 为 @bitkyc08/opencodex 2.84.0，发布元数据的 gitHead 为 6955343008e76e96d1c89bb32a9fc49f50e44fb9；对应源码 package.json 亦为 2.84.0。平台枚举以此固定提交为发布基线。

该版本连接概览有 **23 个客户端项**：18 个文件集成客户端，加 Codex、Claude Code、Claude Desktop、Grok Build、Cursor。API Keys 是单列的凭据入口，不计为客户端。这个数字描述连接概览目录，并非所有代理兼容客户端的穷尽清单，也不是 Desktop 已适配数量。

计数依据：[文件客户端清单](https://github.com/lidge-jun/opencodex/blob/6955343008e76e96d1c89bb32a9fc49f50e44fb9/gui/src/pages/integrations/integration-api.ts#L4)、[概览构建](https://github.com/lidge-jun/opencodex/blob/6955343008e76e96d1c89bb32a9fc49f50e44fb9/gui/src/pages/integrations/overview-clients.ts#L547)、[导出类型](https://github.com/lidge-jun/opencodex/blob/6955343008e76e96d1c89bb32a9fc49f50e44fb9/src/clients/config-export/contracts.ts#L204)。

分别核实：平台能否连接 OpenCodex 模型；自身是否支持 Skills / MCP；Desktop 是否已实现该落点的受控读写。只有模型接入证据时，新增扩展开关仍没有依据。

## 2. 发布基线中的客户端

“待核实”表示本轮尚未逐项确认扩展落点、版本、读写和生效方式，不表示平台不支持。既有六平台引用项目契约，本轮没有重新执行本机验收。

| 客户端 | 上游 ID | OpenCodex 接入依据 | Desktop 扩展分析状态 |
|---|---|---|---|
| Codex | codex | 原生接入 / 概览 | 既有六平台之一；CLI / App / SDK 不按品牌重复计数 |
| Claude Code | claude | 原生接入 / 概览 | 既有六平台之一 |
| Claude Desktop | claudeDesktop；原生控制 ID 为 claude-desktop | 独立概览项 / 原生控制 | 扩展范围、Skills 形态与账户限制待核实；与 Claude Code 分开 |
| Grok Build | grok | 原生接入 / 概览 | 既有六平台之一，既有文档称 Grok CLI |
| Cursor | cursor | 专门接入 / 概览 | 官方文档确认 Skills 与 MCP；Desktop 新适配待设计 |
| OpenCode | opencode | 文件集成 | 既有六平台之一；新版本行为仍须核实 |
| Pi | pi | 文件集成 | 扩展落点与 MCP 接入方式待核实 |
| OMP / Oh My Pi | omp | 文件集成 | 待核实，独立于 Pi 的目标 |
| Hermes Agent | hermes | 文件集成 | 既有六平台之一 |
| OpenClaw | openclaw | 文件集成 | 待核实 |
| Kimi Code | kimi | 文件集成 | 待核实 |
| Gajae Code / gjc | gajae | 文件集成 | 待核实；保留品牌、命令和 ID 的对应 |
| DeepSeek Harness | dsh | 文件集成 | 待核实；存在不同配置落点的线索 |
| MiniMax Code | mcode | 文件集成 | 待核实；与独立 mmx CLI 分开 |
| ZCode | zcode | 文件集成 | 待核实；模型 provider 存储不作为 MCP 存储依据 |
| Command Code | commandcode | 文件集成 | 待核实 |
| Prime Agent | prime | 文件集成 | 待核实 |
| Aside | aside | 文件集成 | 待核实；存在账户 / profile，不宜用品牌 ID 唯一标识落点 |
| Raycast | raycast | 文件集成 | 待核实；不预设有可投影的 Skills 目录 |
| omo / LazyCodex | omo | 文件集成及 Codex 角色相关配置 | 待核实；注意与 Codex 共用实际落点的关系 |
| Cline CLI | cline | 文件集成 | 官方资料确认 Skills 与 MCP；CLI / IDE 差异待核实 |
| Kilo | kilo | 文件集成 | 官方资料确认 Skills 与 MCP；版本与作用域待核实 |
| Factory Droid | droid | 文件集成 | 待核实 |

来源：[集成注册表](https://github.com/lidge-jun/opencodex/blob/6955343008e76e96d1c89bb32a9fc49f50e44fb9/src/integrations/registry.ts#L282)、[固定提交的集成文档](https://github.com/lidge-jun/opencodex/blob/6955343008e76e96d1c89bb32a9fc49f50e44fb9/docs-site/src/content/docs/zh-cn/guides/integrations.md)。文件集成描述模型 / provider 配置，不代表上游统一管理这些平台的 MCP。

## 3. 补充项与版本差异

| 项目 | 实际核实 | 本包处理 |
|---|---|---|
| Gemini CLI | 本项目既有六平台包含 gemini；2.84.0 的上述概览枚举没有该 ID | 保留既有目标，不能据此断言 Gemini 不可接入 OpenCodex |
| Qoder | 主分支观察提交 732fba65b4cb2ff35c3416a4102553cb36ec28bb 的文件清单新增 qoder | 登记为主分支新增观察，未计入 2.84.0 的 23 项 |
| GitHub Copilot 应用 | 2.84.0 文档说明手动模型接入 | 单列线索，不计入概览；与 github-copilot 模型提供方分开 |
| MiniMax 平台 CLI / mmx | 2.84.0 文档说明文本命令启动器，与 MiniMax Code 分开 | 单列线索，不直接成为扩展写入目标 |

来源：[Qoder 所在固定提交清单](https://github.com/lidge-jun/opencodex/blob/732fba65b4cb2ff35c3416a4102553cb36ec28bb/gui/src/pages/integrations/integration-api.ts)、[npm 2.84.0 元数据](https://registry.npmjs.org/@bitkyc08%2fopencodex/2.84.0)、[对应 package.json](https://github.com/lidge-jun/opencodex/blob/6955343008e76e96d1c89bb32a9fc49f50e44fb9/package.json)。只读取公共来源，未安装或运行上游包。

## 4. 三个新增候选的官方扩展依据

| 客户端 | Skills 依据 | MCP 依据 | 对管理形式的影响 |
|---|---|---|---|
| Cursor | 用户 / 项目 .cursor/skills；还读取 .agents 及 Claude / Codex 兼容目录 | 用户 ~/.cursor/mcp.json 与项目 .cursor/mcp.json，同名项项目优先 | 同一资源经多目录发现，投影与实际可见性分开 |
| Kilo | 用户 ~/.kilo/skills 与项目 .kilo/skills，支持共享 .agents 和额外路径 | 全局 ~/.config/kilo/kilo.jsonc 与项目配置；顶层 mcp；本地 command 是数组 | 不能套用 Codex / Claude schema；分开记录作用域 |
| Cline | 官方 Skills 仓库指向 ~/.cline/skills | CLI ~/.cline/data/settings/cline_mcp_settings.json，可由环境变量覆盖；IDE 从自身设置打开配置 | 同品牌的运行端与实际路径须单独确认 |

来源：[Cursor Skills](https://cursor.com/docs/skills)、[Cursor MCP](https://prod.cursor.com/help/customization/mcp)、[Kilo Skills](https://kilo.ai/docs/customize/skills)、[Kilo MCP](https://kilo.ai/docs/automate/mcp/using-in-kilo-code)、[Cline 官方 Skills](https://github.com/cline/skills/blob/main/README.md)、[Cline 官方 MCP](https://github.com/cline/cline/blob/main/docs/mcp/mcp-overview.mdx)。均于 2026-10-14 读取，为持续更新资料；没有绑定本机版本或执行读写、生效验证。

Cursor 将本地与云端范围分开，并自动发现共享 / 兼容目录。取消专用投影后，它仍可能发现同一共享源。Kilo 也说明共享目录发现。管理器需要显示“经共享目录可用”和具体受管投影，不能把移除一份链接显示成平台完全禁用。

## 5. 本项目现状与规模影响

本地 main 核实提交为 eae85322ebca3b755c034db97cbebf2ea5bf71fc。[扩展模块](https://github.com/gzers/opencodex-desktop/blob/eae85322ebca3b755c034db97cbebf2ea5bf71fc/apps/desktop/tauri/src/modules/extensions/mod.rs)将 ClientId / CLIENT_IDS 固定为六项，命令层使用用户级目标。新增平台涉及发现、适配器、目标身份、序列化和兼容读取，不能只追加图标。

既有契约见[集成与安全边界](../../../../../../02-项目核心/集成与安全边界.md)。[唯一总原型](../../../../../01-项目支撑资料/01-产品原型/原型/index.html#extensions)的 Skills 固定目标列为 62 px，间隔 8 px：六列约 412 px，按同样方式扩为 23 列约 1602 px，仅目标区已超 1180 px 默认窗口。900 px 规则仍固定六列。这是 CSS 计算的布局预测，未制作 23 列原型。

建议用“23 项发布目录 + 既有 Gemini + 后续候选”检验容量，另用 50 个模拟目标覆盖同平台多实例增长；模拟目标数量不构成真实平台支持承诺。

## 6. 后续核实

逐平台记录产品 / 运行端、版本、用户 / 项目 / profile 范围、Skills 搜索与优先级、MCP schema 和传输、凭据引用、导入 / 移除、生效方式及 Desktop 适配状态。

先用既有六平台核对新工作流，再以 Cursor、Kilo、Cline 的差异核实新增适配。其他项继续调查，待核实项显示原因，不提供可执行写入开关。这个顺序是设计验证建议，不是产品范围或排期。
