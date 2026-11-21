# ChatGPT内Codex名称验收边界

当前 revision: 4。下方保留 revision 1 合同，最新客户端复核见文末。

revision: 1；2026-11-18。需求AC-P24-09；用户明确实际使用ChatGPT内的本地Codex。显示/来源行为仍归[官方渠道显示合同](official-channel-display-r1.md)，实施状态唯一归[IMP-34](../IMP-OPENCODEX-DESKTOP-34.md)。

1. 验收对象是ChatGPT桌面本地Codex的实际模型选择器；终端、IDE、官方model/list或目录JSON不替代该客户端界面。当前已安装版本与工具限制见[目标回执](../../../.adg/evidence/0.2.24/channel-name-rollback-r1/client-target.json)。
2. 固定官方2.99＋最终Desktop候选在准确隔离配置下保存A→改名B→完整备份回退A。每阶段新官方进程读回/刷新后，由真实客户端新加载其模型目录，逐唯一slug核显示、人工名称优先、原始modelID与路由不变；不发送上游模型/业务同步请求。最终候选的客户端/应用重启与回退需同源证据，本机组件测试不能提前替签。
3. [官方本地目录说明](https://learn.chatgpt.com/docs/enterprise/roll-out-a-gateway)将model_catalog_json定义为启动加载文件、display_name定义为选择器标签，更换目录需重启；[Models说明](https://learn.chatgpt.com/docs/models?surface=app)说明ChatGPT桌面本地Codex共用config.toml。文档只确认机制，不证明本机客户端已读到新内容。
4. 当前CUA对ChatGPT目标以安全原因拒绝。不得用AppleScript、CGEvent、其它工具或另一个bundle别名绕过；不得自动修改日常config/catalog、拷贝真实凭据或重启当前ChatGPT宿主。只有产品确有已验证的隔离配置入口或独立测试用户环境，才由用户完成实际界面核对；不假设CLI的--profile旗可用于ChatGPT应用。未具备条件时如实保留未验，继续其它独立工作。
5. [本轮合成显示表](../../../.adg/evidence/0.2.24/channel-name-rollback-r1/client-target.json)和三份配置仅是可审查输入，不是将它们覆盖到日常目录的指令。预期自动标签“备份渠道 · vendor／custom”→“修改后渠道 · vendor／custom”→原标签；人工“人工名称”始终保持。成功截图/观察记录应来自上述实际客户端并绑定最终源/目录摘要，失败与未完不得覆盖。

## Revision 2 · 2026-11-21

本机只读核对的应用标识为 `com.openai.codex`，版本 `26.1111.71704`，构建 `29030`。上轮目标回执对应当时观察的版本。合成配置的 A → B → 回退 A 已通过，真实选择器仍待验。

官方文档确认桌面端在启动时加载本地模型目录。文档中的配置方案示例使用 CLI 的 `--profile`，桌面启动命令公开的选项仅包含工作区与安装下载来源。当前未确认这个桌面版本存在可独立选取测试模型目录的入口，也未确认有可用的独立测试用户。验收仍沿第 2、4 条执行。[模型目录加载](https://learn.chatgpt.com/docs/enterprise/roll-out-a-gateway) · [配置方案说明](https://learn.chatgpt.com/docs/config-file/config-advanced) · [桌面启动入口](https://learn.chatgpt.com/docs/developer-commands?surface=cli#codex-app)

## Revision 3 · 2026-11-21

只读核对构建 29030 的 `/Applications/ChatGPT.app/Contents/Resources/app.asar`。以下模块均位于 `.vite/build`。

| 模块 | SHA-256 | 观察 |
| --- | --- | --- |
| `early-bootstrap-631-9c86108a3befe979.js` | `3b5c2736da2003d5b52e8a19a59e307e53dc4c3e12c3d9f11e63ef900af41cfb` | 显式 `CODEX_ELECTRON_USER_DATA_PATH` 在请求 Electron 实例锁前设置 `userData`。 |
| `early-bootstrap-133-1770b931864f2bba.js` | `b1139f733dd5ef5b9bcafca560752bd363644d9fb9609407dcdb1dda3a319708` | 显式 Electron 数据路径下，登录 shell 环境加载前保存并在加载后恢复 `CODEX_HOME`。 |
| `early-bootstrap-824-71d2a741f6138352.js` | `be1ce76c13ab45bd2df4575e566a3cd8e8ff2315431a1e55d1ca398058d3b147` | 本地 app-server 的 stdio 子进程继承过滤后的环境，其中保留 `CODEX_HOME`。条件 daemon 分支从同一 HOME 解析控制 socket。 |

这提供了桌面配置隔离的候选入口。真实实例锁隔离、全部数据树隔离及实际模型目录加载仍待证实。验收沿第 2、4 条，维持真实选择器未验证。

## Revision 4 · 2026-11-21

使用原样官方 2.99.0 和当前 ChatGPT 随包的 Codex 后端，在私有 CODEX_HOME 完成三次官方保存读回和模型目录刷新。三个独立 app-server 均报告该私有 HOME。同一自动 slug 的标签依次为“备份渠道 · vendor／custom”“修改后渠道 · vendor／custom”“备份渠道 · vendor／custom”，人工名称保持。默认 Codex 和 Grok 树四阶段一致，验证限于隔离后端消费。[本机证据](../../../.adg/evidence/0.2.24/mac-private-codex-consumer-r1/result.md)

实际 Electron 实例隔离、ChatGPT 桌面选择器和最终 Desktop 候选仍按第 2、4 条验收。私有配置停后的读回只证明字节恢复。
