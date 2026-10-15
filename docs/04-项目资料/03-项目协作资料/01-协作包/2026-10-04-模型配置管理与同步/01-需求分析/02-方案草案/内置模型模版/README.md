# 内置模型模版：首批调研与配置建议

日期：2026-10-13（Asia/Shanghai）。项目：OpenCodex 桌面管理器。归属：模型配置管理与同步协作包。状态：候选资料已整理，初始模版的生产数据格式、初始化及升级逻辑尚未实现。

同日后续：[主原型检查更新交互](../../06-版本记录/20261013-模型模版时间版本与检查更新.md)已采用本资料的静态摘要，展示 17 族初始入口和 41 条精确 API ID 的更新 fixture。研究清单仍为 81 条；不将原型接入视为全部资料已可调用或生产初始库已实现。

用户要求扩充 GLM、HY、Doubao 等主流模型，并网上查证、提前整理。HY 在本轮指腾讯混元 Tencent HY；没有将它理解为所有国产模型的统称。

## 资料入口与范围

- [候选清单](model-seed-candidates.json)：17 个模型族、81 条型号 / 别名 / 权重身份记录，附 48 个官方来源及访问方式。每个型号分别记录服务范围、生命周期、上下文、最大输入 / 输出、模态、推理参数、工具调用和待核实项。
- [型号速查](型号速查.md)：由候选清单生成的可读表，方便评审。改数据时更新 JSON 后重新生成此表。
- [本机模型声明快照](本机模型声明快照.json)：2026-10-13T18:39:16+08:00 更新；24 个具名渠道、92 条目录记录、157 个公开模型形态 ID。含当前模型顺序、启停/白名单/目录可见性、配置声明与目录实际参数；同一 ID 在不同渠道分别保留。可读明细见[当前本机渠道与模型配置快照](当前本机渠道与模型配置快照.md)。相同 ID 不证明服务部署或能力相同。

首批面向对话、编码、推理、Agent，以及可输入图像 / 音视频的语言模型。独立生图、生视频、语音生成、embedding、rerank 不纳入本次通用模型模版。已出现但未核实的本机型号仍在快照中，不以删除或自动更名处理。

本机快照保留发现线索，包含疑似图像生成、embedding 和渠道占位名；它不是自动准入清单。例如 MiniMax-H3、Gemini image / embedding 等名称须先确认任务类型，再决定是否属于其他模型模块。不能因出现在本机配置中就加入通用对话模版。

清单是调研数据格式，不是已冻结的 model-family-templates.json 生产 schema，也不是可直接写入运行配置的导入包。数值为空表示本轮尚未确认，不能解释为不支持；保留官方 K/k/M 原始单位，只有精确整数才填写 exactTokens。上下文、最大输入、最大回答、思维链预算分别记录，不能相加或把最大输出设成默认输出。

## 首批覆盖

| 模型族 | 已登记重点 | 主要依据与适用范围 |
|---|---|---|
| GLM | 5.3、5.3 Flash / FlashX、5.2、5 | [Z.ai 5.3](https://docs.z.ai/guides/llm/glm-5.3)、[Flash](https://docs.z.ai/guides/vlm/glm-5.3-flash)；基础版与多模态版分别配置 |
| HY / 混元 | Hy3、Hy4 preview；旧 2.0 / T1 归档 | [TokenHub 模型列表](https://intl.cloud.tencent.com/zh/document/product/1300/78934)、[原平台下线公告](https://cloud.tencent.com/document/product/1729/131925)；旧平台与 TokenHub 分开 |
| Doubao / Seed | 2.1 Pro / Lite / Turbo、Evolving、2.0 Lite / Mini；旧 Pro / Code Preview 兼容登记 | [方舟模型列表](https://docs.volcengine.com/docs/ark/model-list?lang=zh)；保留日期型号，不以用户 ep 接入点作全局 ID |
| Qwen | 3.8 Max / Flash、3.7 Plus / Flash；旧 3.7 Max、3.6、3.5、Coder | [百炼文本生成](https://help.aliyun.com/zh/model-studio/text-generation-model)；地域、快照和协议控制方式待部署核对 |
| DeepSeek | V4.1 Flash 对应 deepseek-flash、V4 Pro、重定向旧名 | [模型表](https://api-docs.deepseek.com/quick_start/pricing/)、[思考模式](https://api-docs.deepseek.com/guides/thinking_mode/)；显示版本和 API ID 分开 |
| Kimi | K3、K2.6、K2.7 Code | [K3](https://platform.kimi.ai/docs/guide/kimi-k3-quickstart)、[K2.6](https://platform.kimi.ai/docs/guide/kimi-k2-6-quickstart)；必需思考与可关闭思考分别记录 |
| MiniMax | M3.1 Flash Preview、M3、M2.7 / highspeed、旧 M2.5 | [模型调用](https://platform.minimax.io/docs/guides/text-generation)；预览版访问范围及 highspeed 身份单独记录 |
| ERNIE | 5.0、5.0 Thinking Preview | [千帆模型列表](https://intl.cloud.baidu.com/en/doc/qianfan/s/7m95lyy43-intl-en)；本轮仅索引摘要可见，接口字段继续核对 |
| Step | 3.5 Flash、3.7 Flash、5 Preview | [3.5](https://static.stepfun.com/blog/step-3.5-flash/)、[5 Preview](https://www.stepfun.com/step-5-preview)；5 Preview 暂登记显示名称，不猜 API ID |
| MiMo | V2.5、V2.5 Pro | [小米官方介绍](https://mimo.xiaomi.com/mimo-v2-5)、[API 思考回传](https://platform.xiaomimimo.com/docs/en-US/usage-guide/passing-back-reasoning_content)；Pro 不直接复制普通版能力 |
| OpenAI GPT | 6 Astra、6.1 Sol、6 Luna、旧 6 Sol；其他本机 GPT ID 待逐项补证 | [Astra](https://developers.openai.com/api/docs/models/gpt-6-astra)、[Sol](https://developers.openai.com/api/docs/models/gpt-6.1-sol)、[Luna](https://developers.openai.com/api/docs/models/gpt-6-luna)；工具接口和原生 effort 按型号约束 |
| Claude | Fable 5.1、Opus / Sonnet / Haiku 5.5；旧 Opus / Sonnet 兼容登记 | [模型概览](https://platform.claude.com/docs/en/models/overview)；原厂 API ID、渠道别名和 Bedrock 等部署 ID 不混用 |
| Gemini | 3.8 Flash、3.5～3.7 Flash、3.5 Flash-Lite、3.1 Pro 对应本机 ID | [3.8 Flash](https://ai.google.dev/gemini-api/docs/latest-model)、[思考配置](https://ai.google.dev/gemini-api/docs/thinking/)；Interactions 与 GenerateContent 参数分别核对 |
| Grok | 4.7、4.6、4.5 | [4.7 模型页](https://docs.x.ai/developers/models/grok-4.7)；Fast 服务档不生成虚构的型号后缀 |
| Mistral / Devstral | Large 4、Medium 3.5、Devstral 2 | [Large 4](https://docs.mistral.ai/models/mistral-large-4-0)、[目录](https://docs.mistral.ai/inference)；本轮先登记名称，精确 API ID 待补齐 |
| Gemma | 4 E2B / E4B / 12B / 26B A4B / 31B | [官方模型卡](https://ai.google.dev/gemma/docs/core/model_card_4)；权重身份与 Ollama tag、实际部署窗口分开 |
| Llama | 4 Scout、4 Maverick | [官方介绍](https://ai.meta.com/blog/llama-4-multimodal-intelligence/)；权重上限不直接用作第三方服务默认值 |

预览、快速迭代、兼容、重定向、原厂已下线和仅显示名称均为独立状态。81 条是本次登记量，不代表 81 条都已具备可发请求的精确 ID、全部字段或当前账户访问权。旧模型仅在有明确资料时判定原厂下线；第三方仍保留同名时另核对，不自动关停用户配置。

## 与当前使用配置的对照

快照记录的是配置声明和本地目录展示，未发送生成请求、额度探测或登录操作。它可以提供人工偏好和代理行为线索，不能单独证明厂商能力。

| 对象 | 本机观察 | 官方依据 | 初始库处理建议 |
|---|---|---|---|
| GPT-6 Astra / 6.1 Sol | 当前目录变体为 128,000、272,000、872,000；Sol 另见 1,000,000；目录含 ultra | Astra / Sol 页面为 1,050,000，上游 effort 不含 ultra | 保留各来源有效窗口及人工默认；原厂上限与本机 cap 分开；ultra 归执行层，不加入原生 effort |
| GLM-5.3 | 本机 128,000、text + image，档位含 medium / xhigh / ultra，默认 high | [5.3 文档](https://docs.z.ai/guides/llm/glm-5.3)为纯文本、1M、low / high / max，默认 max | 列出模态、窗口与枚举差异；不把本机宽泛字段扩成原厂能力，也不覆盖人工默认 high |
| HY3 | 本机 128,000、text + image、较完整的通用档位 | TokenHub 表为 256k，推理表枚举 none / low / high | 留待渠道复核；不能从模型名或代理协议推导视觉能力 |
| DeepSeek V4 Pro | 本机 128,000、text + image；目录含 ultra | 原厂模型表不支持视觉，原生思考档 low / high / max | 明确原厂与渠道包装的差异；不把此型号当视觉模型预置 |
| MiniMax M3 | 本机为小写 minimax-m3；配置 none / high 与目录 high / max / ultra 不同 | 原厂列表为 MiniMax-M3；上下文 1,000,000；[百炼](https://help.aliyun.com/zh/model-studio/text-generation-model)同名部署列 192k | ID 大小写与服务范围待绑定；能力按部署保存，不以一个全局常量覆盖 |
| Claude Fable | 本机出现 claude-fable-5.1 等写法 | 当前原厂精确 ID 为 claude-fable-5-1 | 保留本机名称，要求渠道映射；不能只因相似就自动改写请求 ID |

以上差异是待核对项，不直接认定本机配置错误。例如较小的窗口可能是用户 cap、代理兼容值或渠道限制；目录提供的 image 也可能来自包装层，必须实际验证才能确认。

## 初始默认版本的组织建议

1. 给每个族提供一个有来源的初始 revision，作为初始模版库的一部分；具体型号差异按精确 ID / 子项登记，不给整个 GLM、GPT 或 HY 族强套一组窗口、模态和 effort。
2. 先固定来源、检查日期、服务范围和匹配身份，再整理可自动填入的字段。名称已知但 API ID 未确认的记录只参与研究列表，不生成可执行匹配。
3. 族通用层未确认字段保持继承。具体型号已确认的能力可以作为候选建议，但须经 Desktop 字段与适配器校验后再编译。配置上限、原厂能力和运行路径限制分别保存。
4. 首次初始化、用户复制为新版本、内置库增量更新、显示差异与采用流程在开发时接入；已有用户修改和精确 revision 引用保留。收录不自动启用模型，不自动换主版本或渠道引用。
5. Fast 不统一用 supported=true：OpenAI service tier、Grok 同模型速度档、MiniMax highspeed 独立型号、Flash 名称以及本机派生行分别解释。只确认速度快不代表支持统一 service_tier 参数。
6. 精确能力采用前要确定协议。支持 OpenAI Chat 形状不表示全部支持 Responses；支持工具调用也不保证该模型在两个接口都能带 tools。Google 原生接口、Anthropic 及各厂商 extra_body 字段需单独映射。

本节是供评审的实现建议。用户已要求提前扩充整理，尚未据此冻结全部初始化 / 更新交互或生产 schema。本轮不修改唯一主原型的六组 fixture，不将研究资料作为已接入的默认模版。

## 来源与验证边界

来源清单及 URL 在候选 JSON 的 sources 中，逐条带 page 或 indexed-excerpt。page 表示本轮取得正文；indexed-excerpt 表示只取得官方页面的搜索索引摘要，可能落后于正文。腾讯部分文档、千帆及若干动态页面正文访问失败，已如实保留该级别；不以开发者社区转载或第三方聚合站补齐未确认字段。

2026-10-13 首批调研时完成（下列 122 条为首次采集统计，当前快照统计见资料入口）：候选及快照 JSON 解析、48 个来源引用完整性、81 条型号 / 服务范围唯一性、推理默认与枚举关系、已归档条目的默认排除、122 条快照的字段白名单检查均通过；本轮相关 Markdown 本地链接 72 处有效。两个本机输入读取前后 SHA256 一致，未改运行配置。没有执行真实模型请求、原型浏览器 QA、配置应用或生产验收。

下一步按资料中的待核实项补齐型号详情、精确 API ID、原生参数及来源单位，再将可采用项转换为 WP-01 冻结的数据格式。优先处理 GLM / HY / Doubao 与现有本机声明的冲突；与开发一起验证协议、窗口、模态和推理字段。不需要等整个模型模块开发完才准备内容，也不把资料整理完成当作功能实现。
