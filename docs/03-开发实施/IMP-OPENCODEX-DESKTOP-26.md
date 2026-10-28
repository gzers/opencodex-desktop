# 0.2.17 Windows安装可靠性与全链验收

2026-10-28，Asia/Shanghai。状态：in_progress。用户已明确授权WP-89～95文档治理、开发、测试、发布，目标至stable。

基线v0.2.16/d2c08dd7，代码沿ocx-022-model-selector/codex/0.2.17-windows-install，治理在docs/governance-main；并行对话/树/WebSocket原型和协作内容不纳入本轮。

范围与验收归[DMD AC-WIN-017-01～06](../01-需求管理/需求/DMD-OPENCODEX-DESKTOP-MANAGER.md)，当前清单沿[WP89～95](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.16-bugs.md)。先修路径/安装事务及探针，再工具发现/文档/能力说明，再全量与Windows实际流程；不增加新的核对页、保存计划或通知入口。

安装前只读验证路径与目标归属；新包在临时区校验，旧包和入口保留至最终激活与来源记录提交。落位或后续失败有保护地恢复，回滚失败明确保留材料。进程内补偿不宣称突然断电双文件原子性。探针有界零退出/版本/超时，不忽略官方真实错误。

版本管理器只读受控路径/环境声明，有界扫描版本目录，不执行shell配置或盲扫PATH。许可与第三方声明纳入真实bundle资源；WindowsCLI仍禁用，只修“不存在IPC”的过时说明。

WP95以清单逐项审查原生覆盖：隔离Windows runner实际安装/故障、默认值、令牌、文件读回、NSIS/MSI和更新包；host Win10/11、混合DPI、PAC企业证书等不能由单一runner证明，保留未执行边界及可重复检查，不能用mock自称全域通过。不给用户日常配置或活跃Codex强制重启，不发付费生成请求。

检查名固定为review:windows-path、review:install-transaction、review:version-probe、review:tool-discovery、review:packaged-docs-cli、review:native-windows、review:regression-release。完整本地门禁、双平台CI和实际安装器/签名/匿名stable完成并回写后关闭。

## 实施启动记录

代码分支 codex/0.2.17-windows-install 已从 v0.2.16 合并点建立，首轮实现提交 57578f895。已落地：扩展盘符/UNC 前端路径身份、安装目标无副作用校验、受管 manifest 归属、入口激活失败回滚、非零版本探针拒绝、Windows 工具声明路径、包内许可资源和 CLI 能力文案。专项 runtime 99 项、安装设置前端 47 项、文档命令 8 项通过；Windows 原生新增测试待 CI 执行。

当前提交只代表实施阶段，不能提前关闭 WP-89～WP-95；实际 Windows 原生、普通权限和发布证据仍是必要门槛。

代码已完成首轮实现提交 57578f895，包含扩展路径、安装目标归属、激活失败回滚、零退出版本探针、资源文档、工具声明路径及 CLI 边界文案。专项 runtime 99 项、前端 987 项、文档资源命令 8 项、后端串行全量 1287 项、clippy 和前端 build 已通过；Windows 原生新增用例与双平台 CI 待收口。
