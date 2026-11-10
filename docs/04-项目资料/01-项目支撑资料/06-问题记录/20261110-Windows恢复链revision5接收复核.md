# Windows恢复链revision5接收复核

2026-11-10。接收用户“已按revision5完成并推送”的回交。开发基线Desktop0.2.23、修复目标0.2.24。本轮为Git原始字节、实际执行源码、原始阶段和验收合同复核，没有在Mac重跑Windows，也没有集成test原型到生产。

## 接收身份与实际检查

- 文档/证据：docs/official299-windows-proof，c80208ee14c44eed3e6a73c0511f09f064927e69。
- test交付：codex/official299-windows-proof，cba03b42151843c753f25f53ca71a6b2b31dccef；本轮唯一官方服务实际源码66afa61133d7654d3a2dfd172f49845bfb5272c3。后继离线复核/权限差额/清理提交不冒充实际执行源。
- 采用文档基线1f35d520f74ac261f32c407f038e8f07a21c15fd，revision5；已采用两锁例外，不扩大到其它对象。
- 70份原始文件7,270,802字节逐项大小/SHA256核对，10项执行源摘要与Git原字节一致。当前实际候选、候选测试和CLI三文件与Mac交接完全相同，旧producer-r4源码保持原字节。
- 独立运行交付离线复核，exit0，summary与原件完全一致。接收方另逐项验证39次阶段原字节/长度/SHA256及保护ledger，重新执行结构分析，输出与Windows原件完全一致；独立核对五路径原字节/适用权限、原catalog中的gpt-5.4确实恢复及最终树只有两个允许路径新增。
- 原HMAC secret省略，复核仅核对实际执行源码/原句柄身份和签名回执一致性，不声称Mac重算秘密签名或再次原生验收。

原始结果见[Windows受控缓存恢复链补验](20261110-Windows官方299受控缓存恢复链补验结果.md)。接收证据位于.adg/evidence/0.2.23/post-release-20261110/windows299-r5-receive-review/，原证据目录windows-native-r5不可变。

## 本轮合同判定

**本次隔离单用户技术可行性合同通过。** 原始完整树相等仍false，只排除已采用两个稳定锁的合同为true；旧revision1/2/4失败和额外严格权限检查失败全部保留。

| 要求 | 本轮证据/结果 |
| --- | --- |
| 小原型先行 | Windows候选39项及CLI桥接排他/输出create-only反例通过，原有原语按实际依赖复用；服务前冻结最多1轮。 |
| 原30秒启动 | 15339ms，唯一完成行/启动回执、原身份/Job/端口/官方health/ready一致。 |
| 原5秒状态 | 4ms，同原句柄代次与官方健康就绪核对。 |
| 原始正常退出与全部监听 | 官方performed原回执；原句柄exit0、Job Active0/完整成员列表空、保留子进程均退出；3个实际监听52966/53066/58823正常Stop后及最终读回为空，无强杀补通过。 |
| 新缓存分支 | 原cache缺席，固定原包/官方startup-cache、独立启动前scope、ready/pre/official-stop原字节/ID/owner/DACL连续及ready catalog一致，当前原生context和阶段文件受保护；同秒表实际context时间5220ms。 |
| 5路径及协调 | 当前轮实际取得官方claim/N/K，原生排他批次5项完成并释放协调；helper原句柄exit0/Job空。审计完整后像先封存。 |
| 精确原件恢复 | Codex config.toml、opencodex-catalog.json恢复启动前原字节/owner/DACL（原生gpt-5.4未被样例预过滤）；models_cache.json、opencodex-catalog-audit.jsonl、Grok备份恢复原缺席。Agent原集合/字节/owner/DACL恢复，用户定义保持。 |
| 当前轮两个锁独立证明 | 同新SID的两个新进程分别实际取得/释放claim、owner；各原句柄exit0/Job空，文件ID和owner/DACL不变，Stop内约6126/6368ms分别完成。 |
| 完整树 | 所有原文件名/长度/摘要一致；只多codex/.opencodex-native-main.claim.sqlite及codex/.opencodex-native-main.owner.sqlite，原raw比较false与例外合同true分别保留。 |
| 原10秒完整Stop | 总6425ms，包含官方退出、Agent/5路径补偿、上下文/CLI/协调、两锁重获、最终全树及原生读回；无新秒表或额外恢复时间。原始trace单调且实际到达所有原未到达步骤。 |
| 原始后像 | 六阶段均有实际内容，39次原字节/长度/摘要一致；host-compensated本轮真实到达。finally标签仍叫failure-finally，但本轮accepted:true，不将标签误认执行失败。 |
| 测试资源清理 | 当前SID1029账户/Profile/目录、Job/原进程、3监听、临时包体及准备目录原生读回清理通过；原token/hProfile卸载，非Special且未Loaded门保持，无强卸hive/注销日常会话。 |

## Grok主配置权限差额如何处理

Windows后继额外检查要求“所有已声明原件的SDDL逐字相同”，Grok主config不满足，原失败和当时核验提交另存。独立复核显示其原字节/owner保持，DACL由继承本SID/SYSTEM FullControl变为受保护的仅本SID FullControl，没有新增第三方principal。

当前交接对Grok主config要求原字节恢复及安全权限，精确权限回放约束针对Agent及五路径原件；revision2/4/5正文没有把所有客户端文件SDDL逐字相等作为同一门。因此按实际合同接受本次单用户通过，同时明确：**Grok主config权限收紧，不能写成“全部权限完全保持”。** 额外严格检查失败不抹去，不把它改写成通过，也没有为新增文件引入例外。如果产品后续需要保留已有的合法第三方权限，应在生产设计/验收中明确处理，本轮仅使用本SID/SYSTEM合成私有目录，不外推任意用户ACL兼容。

## 可闭合与仍未完成的边界

可闭合的是：原样官方2.99配合Desktop侧独立测试宿主，在一个真实新普通用户的受保护合成事务里，能够满足身份/就绪/正常退出、Agent/5路径精确恢复、官方协调、当前轮锁释放/重获、全部实际监听、仅两锁最终差额及原30/5/10秒预算。这解决此前单用户技术可行性连续失败，不要求继续第六轮无假设重跑。

仍未完成：该机制尚在test原型，生产apps未接入；一般日常文件的写者归属/冲突/持久崩溃恢复边界需要产品实现，不能以合成scope证明任意用户文件。未做新官方适配后的双SID同时运行，不替代真实多用户原生验收；安装器/签名原包、名称手动GUI入口/刷新重启回退、0.2.23v2来源变化无损重核等独立0.2.24条目继续原计划。产品开发、全量或发布状态不由本次accepted:true自动升级。

下一步应整理唯一0.2.24实施清单，将本轮技术原型来源/合同/权限差额及实现边界承接到相应开发项。保留成功及全部失败原件，按后续实际修改的依赖选择原生复验；此刻不再次派Windows、不开双SID/安装器/全量或自动合并test到main。
