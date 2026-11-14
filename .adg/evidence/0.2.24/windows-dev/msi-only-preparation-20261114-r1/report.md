# MSI 专项执行准备，未取得真实卸载结果

2026-11-14；TEAM revision10 / IMP-34 revision136；承接 main fd2538e4dae124b7c521b5b84b7ca5b5c6eeef6b、docs/governance-main 242f46e71e79501e11610c9a7aa7d6d7bc2e9fd8。环境接收和 MSI-only 合同仍适用。代码 e2560f9047e453157bcc71948904fbf199a0540a 已推送 codex/024-windows-native；实际编译/发现命令及制品摘要见同目录机器回执。

待证伪假设：独立 ignored MSI-only 用例可在不重跑终端/官方消费者的条件下，准备真实安装前提、pending Enable 失败保留和继续后正常卸载两项事务观察。此轮仅完成入口准备，不能证实两项真实事务行为。hypothesis.json 在编译和脚本验证前保存。

用例使用原独立开发测试 MSI，固定 bytes / SHA / ProductCode；不重建、不复制大资源、不更改包条件或安装范围。read-only Product 表查询核版本0.2.24、独立产品身份和无 ALLUSERS。程序实际安装来源由原 MSI 落盘和原 EntryPin 验证，最小合成镜像仅提供 Enable 前提，不执行官方脚本。

真实 /x 的失败路径要求1603、MSI仍登记、程序卷/文件ID/摘要、PATH、pending/config、D/H哨兵不变，日志须表明 OCX023RevokeTerminalCli 在 RemoveFiles 前失败；前后像独立采集。继续原 Enable 后要求真实 /x 成功、精确 PATH 还原、程序/终端登记/pending移除、D/H/live runtime保留。账户控制器不在产品登记仍在或未知时删除账户/Profile，清理未完使控制器失败。原断言、20分钟上限、未知拒绝保持。

| 独立项 | 本轮状态 | 证据/边界 |
| --- | --- | --- |
| MSI-only MSVC 编译 | 通过 | build02 exit0，保留42条warnings；no-run；build01为前一版快照，build02为最终哨兵前后像版 |
| 精确 ignored 入口发现 | 通过 | discovery.json：1 test, 0 benchmarks，exit0；仅 list，不执行用例 |
| 三份 PowerShell 解析 | 通过 | script-checks-final.json，0 parse errors |
| 日常主机保护 | 通过 | daily-host-guard.json；缺配置路径亦在读取前拒绝，无账户/MSI |
| 原 MSI 身份/摘要 | 通过 | package-facts.json；316784640字节，9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282 |
| 允许 per-user MSI 的隔离环境 | 未取得 | 复用上一轮只读盘点，不重复枚举/安装 |
| 真实 MSI 安装前提 | 本轮未执行 | 不在日常主机探测策略许可 |
| 真实 MSI pending 失败保留 | 未执行 | 无合规隔离环境 |
| 真实 MSI 正常卸载 | 未执行 | 无合规隔离环境 |
| 终端/PowerShell/官方2.99/H读取 | 复用既有通过 | w1d-native-entry-20261113 / out04，未受本轮测试入口变更影响 |
| PATH恢复/schema3/NSIS宏 | 复用既有通过 | 10格恢复及4类冲突、out05、NSIS5格；未重跑 |
| W2/双SID/发布 | 未执行 | 不在本轮范围 |
| allAccepted | false | 准备可用不等于MSI通过或整版收口 |

本轮没有新运行时首错。证据核验命令首报 Python 换行转义 SyntaxError，提交前即停止；改用 chr(10) 后核验5份实际编译源未变化并记录Git blob，见 tooling-first-error.json。保护验证中的主机拒绝是预期结果；原 out03 真实安装1625/当时有效 DisableMsi=1的失败保持，不称为当前注册表显式1。上轮四视图策略键缺席也不证明安装许可。仍缺用户提供的已允许普通用户per-user MSI隔离机/VM连接，或具体Windows镜像、可用资源及创建/销毁范围。按照环境接收合同，未获此输入不自行创建VM/改机器策略/提高测试令牌。

最终编译测试 EXE 为91992576字节，SHA256 712a6e8e5bfbb607433b3b17bce77c05523f678058dba755cba679bdd447e65c。只运行 --exact 名称 --ignored --list，不执行测试或制作release包。source-worktree-manifest绑定实际执行字节，source-staged-manifest单独绑定Git blob；不把CRLF/LF摘要差异说成逐字相同。

清理：本轮未创建测试账户/Profile、VM、安装/卸载进程，未写真实用户PATH、日常配置或官方2.99。新文件为许可写集内源码和证据；复用的 w1c-product 构建缓存与原MSI继续留存，未重试历史清理拒绝。原4个W0失败目录、installer-w1、W1-C/W1-D根、schema/空目录及Public盘点根均沿原未清事实保持。没有声称全机无残留或全机进程清洁。未绕过任何自动审批拒绝。
