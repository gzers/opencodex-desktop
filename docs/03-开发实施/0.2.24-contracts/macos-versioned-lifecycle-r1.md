# 0.2.24 当前版本化程序的Mac实际生命周期检查

revision: 1；2026-11-11。归IMP-34 P24-00/04/05；代码main336647e3ef1c031d88375f275ef88b7a7d3f00df，承接program-consumers-r2及macos-first-supply-native-r1。诊断accepted=false、非冻结候选，不发布。

## 产品缺陷及本批修正

正常供应后RuntimeHandle已按current.json指向versions/2.99.0，但commands::execute_lifecycle仍绕过该来源，直接调用平铺resolve_managed_installation(root)，在执行官方进程之前报manage backups/file not found。受控当前格式夹具的生产动作回归先失败、篡改源反例仍拒绝，原件保留。

本批改为同一个context.runtime.managed_program_identity完整动作准入，再核对installation_root和当前入口一致；不使用轻量观察缓存，不扫描或接管旧平铺路径。Node被改动仍在端口写入/进程启动前拒绝。生产命令仍是原ensure，没有在这次修正中改成另一种启动策略，官方2.99也未修改。2项相关边界检查通过；最终编译、格式、全目标严格Clippy通过。

## 固定实际输入与隔离

官方2.99.0、tgz SHA b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c；完整资源descriptor SHA e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3。实际Rust库测试二进制调用生产process_action_with_runner、当前RuntimeHandle、真实ControlledProcessRunner及生产归属/停止查询，使用每次新建/private/tmp/ocxd024-versioned-lifecycle-*/私有profile和版本程序根。不是最终.app/DMG，也不当新OS SID。

内核拒绝日常HOME读取、Keychain、外网及夹具外写入；只允许loopback服务、自有根、按当前私有CODEX_HOME摘要精确限定的catalog/native/history协调数据库及各自sidecar。native首次采用临时文件也只开放该单一摘要前缀和格式，不开放整个账号协调目录。外网及日常文件读拒绝已实际验证。配置/密钥均为显式合成输入，模型地址是未调用的loopback:1；无真实推理、业务同步、日常配置或Shell修改。

## 同轮完整结果

最后一轮native-attempt3，实际执行输入为main7be8d2658加本修正及当时测试源原件，来源摘要/补丁保留。后继代码提交包含测试证据捕获增强；不能倒称新捕获钩子已在这轮执行。

| 项目 | 结果 | 实际事实 |
| --- | --- | --- |
| 当前版本化来源及实际启动 | pass | 4453ms，返回Started，原30秒不变。 |
| 真实运行状态 | pass | 1096ms，受保护记录/官方签名同连接观察得到ready=true、PID23512、port50298；原5秒不变。 |
| 生产正常停止 | pass，限服务停止合同 | 2372ms，生产停止结果Stopped、运行及PID记录移除；原10秒不变，无强杀补通过。 |
| Codex config.toml原字节 | pass | 启动时实际改变，停止后与原输入字节一致。 |
| opencodex-catalog.json原字节 | fail | 停止后不等于原输入。 |
| models_cache.json原缺席 | fail | 停止后未恢复原缺席。 |
| opencodex-catalog-audit.jsonl原缺席 | fail | 停止后未恢复原缺席。 |
| Grok配置备份 | 仅缺席保持 | 实际目标为.grok/config.toml.bak-opencodex，未测试先改变再恢复，不能标完整Grok还原通过。 |
| 全客户端恢复/整版验收 | fail/未完成 | 尚缺Desktop恢复事务，不能以正常服务停止替代全部文件还原。Agent/全树/双SID未覆盖。 |

完整生命周期诊断仍exit101/accepted=false；已通过项保留原范围，不把最后三个文件差额从合同删除。该轮原生结果记录逐项字节相等布尔与日志；其before/after原字节在当轮夹具删除前未输出，明确保留这一证据局限，不拿前两轮postimage冒充本轮。最终test入口已补五个显式合成文件的before/after-start/after-stop原字节和摘要捕获，且增加私有临时根/不得覆盖既有配置前置；这只是后继测试改进，编译/Clippy通过，未无假设重跑整个服务链。

## 前置失败及归因

- 早期sandbox网络地址拼写、证据父目录漏建、端口检查被禁止，均在实际服务启动前失败；源错误/首错保留，不放宽产品。
- native-attempt1真正启动进程但官方readiness失败。独立foreground原包诊断打印native-write-locks采用临时文件EPERM；停止也返回success=false。只是测试写集遗漏，不能当官方服务可忽略未就绪。
- native-attempt2精确补native协调后实际启动通过，但同步状态调用在裸测试线程没有Tokio句柄，返回Unreachable而未发HTTP。另在原已退出夹具执行官方restore诊断，明确history worker unable to open database，证明停止还缺精确history协调权限；不把该诊断当原Stop通过。
- native-attempt3只修测试运行时前置、history精确路径及真实Grok目标，沿同一产品修正核对全部结果。没有改官方源码、断言或30/5/10预算，也没有重复已通过的纯来源反例。

## 清理、证据与后续

证据.adg/evidence/0.2.24/macos-versioned-lifecycle-r1/分目录保存所有原失败、foreground/restore诊断、每轮summary、实际源码摘要/runner、管理器trace及最终检查。所有自有程序进程组/实际镜像已不在；首轮停止失败后进程随后自行退出，不能冒称那轮正常Stop通过。lsof因无关已挂载SMB卷告警不被当缺席证明（只省略无关挂载完整路径，保留原输出摘要及警告/拒绝结论）；独立完整Darwin TCP表framing和原主端点缺席核对通过。剩余精确新协调数据库成功取得/释放SQLite排他事务后清理，全部自有profile/程序根/副本/协调文件与重复临时回执已移除。未触碰正式.app或日常服务。

下一批必须先实现Desktop客户端恢复事务：启动前原件/缺席、当前代次与官方变更归属、只补偿确定的后像、官方协调下的持久恢复/冲突拒绝，覆盖目录/cache/audit及实际适用的Agent/Grok范围。不能只在Stop后看见文件就覆盖、只以HASH/mtime宣称是官方写入、或删除审计/缓存来凑通过；原30/5/10预算和两锁有限例外保持。完成可审查实现及反例后再跑一次受影响完整恢复链，已通过启动/状态不另开无假设轮次。WindowsW0/W1新贡献尚未到达，产品接线/W2、NAME、System代理及发布门禁仍按IMP-34继续。
