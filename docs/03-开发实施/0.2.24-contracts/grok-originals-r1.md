# 0.2.24 Grok原件范围与明确用户Profile绑定

revision: 1；2026-11-12。归IMP-34 P24-04，承接[阶段字节原件](client-tree-bytes-r1.md)。代码main 6d34836ed29b4f3b22be25abe88c5e7fdb2aacaf；accepted=false、非冻结候选，Codex三件实际补偿/完整恢复许可仍未接。

## 官方实际路径和修改点

从固定未改官方2.99 tgz核对src/grok/inject.ts：resolveGrokHome优先显式参数/GROK_HOME，再HOME对应.grok。inject在目录缺席时跳过，普通写点为config.toml及config.toml.bak-opencodex；已有备份不覆盖，stop移除自有管理块。受控Mac runner清空继承GROK_HOME并固定ProcessContext用户HOME，所以本批绑定该明确Profile的.grok，不从Codex目录猜实际Profile，不扫描旧目录或另一用户位置。此合同不冒称终端直接运行官方CLI并传外部GROK_HOME的范围已经由Desktop捕获。

## 帧和字节归档

原件帧schema4新增profileRoot原生身份及GrokSnapshot，状态为absent或directory(tree)。捕获缺席时对原Profile目录no-follow双查，只有明确ENOENT为缺席，不创建.grok；普通文件/链接/无权限/错误用户都拒绝。已有目录沿原PinnedDirectory和描述符相对完整有界树捕获，包含普通配置、已有备份和所有后代（含Agent文件）。源变化时失败，不把晚到内容当原件或改用户权限。journal与两客户端根重叠则拒绝。

Original/Ready/PreStop/Stopped均把Grok树及普通字节对象先保存到原attempt，再发布frame；沿原内容SHA/私有0400/O_EXCL/fsync/读回/缺失损坏拒绝合同。每个阶段允许记录实际不同状态，例如原本缺目录、之后用户建立目录；原阶段不会因此补齐为存在。archived_grok_file只从独立原阶段取回，当前文件变更或另一Profile不能重定向原件。已有schema1/2/3均拒绝，不迁移或自动补档。

TreeSnapshot明确区分Codex协调根与普通客户端树。只有Codex根的精确claim/sidecar才是metadata-only协调条目；Grok中的同名普通文件必须按普通内容归档，不能借用Codex例外被静默跳过。各角色通过必需字段绑定；重开时Grok不能伪装成有Codex协调语义的树。原3件恢复内容策略仍只处理Codex三件；本批没有让它删除/改写Grok文件，也没有把Codex N/K锁当Grok写者锁。

生产StartupSession.begin/open调用begin_for_profile/open_for_profile显式传ProcessContext Profile；旧两参数便利入口仅cfg(test)存在，用现有夹具已知父目录当合成Profile，不进入正式构建，不能当旧版本路径fallback或运行时推断。

## 验证

87项相关检查通过：4 Grok/8树/4字节归档/22原件/20恢复机械/9会话/20索引；子进程probe不另计，原有专用现场/原包技术入口的ignored不当通过。覆盖缺Grok根不创建、后来出现保持不同阶段、原配置/旧备份/内嵌Agent/同名协调文件独立读回、当前源后来修改、symlink/非目录/不可读拒绝、另一Profile或丢失字节不修复。树角色变化的现有归档/恢复消费者一起检查。首Clippy测试权限字面量0改0o000，原stderr保留，不抑制规则；最终编译/格式/全目标严格Clippy通过。一次rg表达式错误仅工具输出，不伪称已存独立stderr。

真实原包使用全新私有Profile，先放有效[model.native_fixture]配置、已有旧备份和.grok/agents/AGENTS.md，未启动实际Grok/Agent客户端或发模型请求。运行结果：

| 项目 | 实际结果 |
| --- | --- |
| Start | 16473ms，原30秒，真实Ready；grokConfigChangedDuringStart=true。 |
| Status | 1102ms，原5秒。 |
| Stop含双树阶段归档和计划登记 | 9813ms，原10秒，原内核exit0/运行记录缺席；仍返回client_original_restore_pending。 |
| Grok配置字节 | 原79字节→Ready/PreStop8655字节→Stopped79字节，原SHA恢复一致；旧备份及内嵌Agent原字节保持。 |
| 独立档案 | originalGrokConfigBackupAndAgentArchived=true，四schema4帧/Profile/双树/独立索引revision6/plan/stage匹配；全部15个内容对象SHA/长度/0400/uid/单链接核对。 |
| 新collector | 801ms，Stopped+Required、正确D/H、运行port/PID空、三动作门槛false。 |

这是普通官方Stop恢复本轮合成Grok配置的证据，加上Desktop独立保存原件的证据；不代表Desktop实际Grok恢复引擎/写者许可或所有Grok情况已完成。首用备份创建、外部根/合法链接/大树等仍待对应范围，Codex catalog/cache/audit尚未恢复，原完整assert保持exit101/accepted=false。Stop9813ms已贴近原10秒，后继实际apply和完整证明仍须共用原预算，不能再开10秒。

## 证据、清理与剩余

.adg/evidence/0.2.24/grok-originals-r1/保留官方源SHA/合同片段、纯回归/首lint/最终结果、实际前后像/trace、四阶段及Profile/双树/内容对象匹配表、固定合成Grok原配置/旧备份/Agent字节、源码/binary与manifest。最终原生轮8件源码逐件匹配，纯回归差异仅测试字面量等价写法。仅合成输入，动态生成对象不作原字节公开输出，不含日常完整凭据或runtime secret/admin token。

完整Darwin TCP表及自有镜像核对原主端点/进程缺席，新N/K/history数据库逐个排他取得/释放后删除。程序副本/Profile/Grok/Codex全部blob/frame/索引/stage/test binary及核对后的重复回执根清理，内核拒绝日常HOME/外网；未改日常配置/Shell/正式安装/官方2.99，无模型或真实WebDAV业务请求，未出新.app/DMG或发布。

继续受控进程集合/监听/写者许可、Codex三件及完整客户端实际补偿、显式继续/回退、外部Agent/自定义根/缺Codex根/大树链接/flags/跨卷/通用图与旧revision4首错；WindowsW0 HOME/Profile/W1/W2、NAME、复杂System代理、官方发布验证与固定候选保持IMP-34完整范围，目标active。
