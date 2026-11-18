# ChatGPT内Codex名称验收边界

revision: 1；2026-11-18。需求AC-P24-09；用户明确实际使用ChatGPT内的本地Codex。显示/来源行为仍归[官方渠道显示合同](official-channel-display-r1.md)，实施状态唯一归[IMP-34](../IMP-OPENCODEX-DESKTOP-34.md)。

1. 验收对象是ChatGPT桌面本地Codex的实际模型选择器；终端、IDE、官方model/list或目录JSON不替代该客户端界面。当前已安装版本与工具限制见[目标回执](../../../.adg/evidence/0.2.24/channel-name-rollback-r1/client-target.json)。
2. 固定官方2.99＋最终Desktop候选在准确隔离配置下保存A→改名B→完整备份回退A。每阶段新官方进程读回/刷新后，由真实客户端新加载其模型目录，逐唯一slug核显示、人工名称优先、原始modelID与路由不变；不发送上游模型/业务同步请求。最终候选的客户端/应用重启与回退需同源证据，本机组件测试不能提前替签。
3. [官方本地目录说明](https://learn.chatgpt.com/docs/enterprise/roll-out-a-gateway)将model_catalog_json定义为启动加载文件、display_name定义为选择器标签，更换目录需重启；[Models说明](https://learn.chatgpt.com/docs/models?surface=app)说明ChatGPT桌面本地Codex共用config.toml。文档只确认机制，不证明本机客户端已读到新内容。
4. 当前CUA对ChatGPT目标以安全原因拒绝。不得用AppleScript、CGEvent、其它工具或另一个bundle别名绕过；不得自动修改日常config/catalog、拷贝真实凭据或重启当前ChatGPT宿主。只有产品确有已验证的隔离配置入口或独立测试用户环境，才由用户完成实际界面核对；不假设CLI的--profile旗可用于ChatGPT应用。未具备条件时如实保留未验，继续其它独立工作。
5. [本轮合成显示表](../../../.adg/evidence/0.2.24/channel-name-rollback-r1/client-target.json)和三份配置仅是可审查输入，不是将它们覆盖到日常目录的指令。预期自动标签“备份渠道 · vendor／custom”→“修改后渠道 · vendor／custom”→原标签；人工“人工名称”始终保持。成功截图/观察记录应来自上述实际客户端并绑定最终源/目录摘要，失败与未完不得覆盖。
