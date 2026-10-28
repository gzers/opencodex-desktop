# 0.2.17 Windows安装可靠性与全链验收

2026-10-28，Asia/Shanghai。状态：in_progress。用户已明确授权WP-89～95文档治理、开发、测试、发布，目标至stable。

基线v0.2.16/d2c08dd7，代码沿ocx-022-model-selector/codex/0.2.17-windows-install，治理在docs/governance-main；并行对话/树/WebSocket原型和协作内容不纳入本轮。

范围与验收归[DMD AC-WIN-017-01～06](../01-需求管理/需求/DMD-OPENCODEX-DESKTOP-MANAGER.md)，当前清单沿[WP89～95](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.16-bugs.md)。先修路径/安装事务及探针，再工具发现/文档/能力说明，再全量与Windows实际流程；不增加新的核对页、保存计划或通知入口。

安装前只读验证路径与目标归属；新包在临时区校验，旧包和入口保留至最终激活与来源记录提交。落位或后续失败有保护地恢复，回滚失败明确保留材料。进程内补偿不宣称突然断电双文件原子性。探针有界零退出/版本/超时，不忽略官方真实错误。

版本管理器只读受控路径/环境声明，有界扫描版本目录，不执行shell配置或盲扫PATH。许可与第三方声明纳入真实bundle资源；WindowsCLI仍禁用，只修“不存在IPC”的过时说明。

WP95以清单逐项审查原生覆盖：隔离Windows runner实际安装/故障、默认值、令牌、文件读回、NSIS/MSI和更新包；host Win10/11、混合DPI、PAC企业证书等不能由单一runner证明，保留未执行边界及可重复检查，不能用mock自称全域通过。不给用户日常配置或活跃Codex强制重启，不发付费生成请求。

检查名固定为review:windows-path、review:install-transaction、review:version-probe、review:tool-discovery、review:packaged-docs-cli、review:native-windows、review:regression-release。完整本地门禁、双平台CI和实际安装器/签名/匿名stable完成并回写后关闭。

## 实施与当前验证

首轮实现57578f895之后，dcc7c96b4补齐了来源登记、入口和包体同一安装事务，真实零退出/64KiB/超时与后代清理的版本探针、扩展路径分隔符、版本管理器有界发现、资源许可实际内容；22f34c82加入正式旧版MSI/NSIS经生产stable更新入口的OTA验收脚本。所有实现仅在代码分支；原型未新增样式与页面。

本地最终前端987通过、10既有跳过，clippy/typecheck/build通过；后端完整串行结果及21项原型/浏览器/真实文件门禁在证据目录，不将macOS结果当作Windows原生结果。第一轮Windows CI的扩展路径rename错误123已保留原日志，按真实原因修正混合分隔符；增强版CI继续执行，尚未确认正式包/OTA或stable。

原生验收使用隔离临时目录、真实Node/npm与官方固定版本包，含联网升级、热缓存离线重装、实际Windows锁回滚、默认/恢复默认向导及普通账户令牌。OTA在0.2.17成为stable后执行真实旧版更新与重开，不以签名文件检查代替应用替换。发布后的验证失败须记录并处理，不能提前关闭目标。

Windows 10/11物理设备、混合DPI、真实SMB/企业代理PAC、企业证书等保留环境边界；runner结果只声明实际执行的范围。历史审计与旧版通过结论不改写为本版全域通过。
