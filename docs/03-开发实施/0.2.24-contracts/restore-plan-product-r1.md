# 0.2.24 停止后恢复计划的准备与独立引用

revision: 1；2026-11-12。归IMP-34 P24-04，承接macos-current-session-product-r1、macos-client-restore-race-r1。代码main d47a431dc9519d4fd0b46e2800f171988c0d5a3e。本批完成真实Stop后的计划准备/独立登记，未开放实际客户端恢复许可；accepted=false、非冻结候选。

## 当前产品输出

应用Active::stopped沿原绝对10秒及同一NativeLease，先封存真实CompletedStop对应的Stopped阶段并独立提交，然后调用StartupSession::prepare_restore，按原Original/Ready/PreStop/Stopped阶段生成原件发布计划，最后将唯一plan id/SHA256提交到当前会话索引。结果为RestorePrepared，保留原完整References，不清原件、不解除新启动门、不写客户端恢复成功outcome。commands入口仍准确返回client_original_restore_pending。

准备调用前后核对同一context/data、原受保护运行记录摘要、原进程出生代次、已完成普通Stop证明和原截止时间；runtime-port必须仍明确缺席。prepare只在已有Stopped原件会话上运行，未知场景不生成新停止基线。Pending沿既有内容规则保留原件暂存及固定计划；只在已知唯一plan名称下暂存本来就持有的原字节，不替换/删除三件实际客户端目标，也不执行apply。许可完整性不足时不能将原计划准备API当作写盘授权。

本轮早期曾草拟仅凭单PID正常退出签发MutationPermit及apply的接线，复核确认其不足以证明完整受控进程集合/监听/写者归属后，在提交和真实原生测试前撤回。最终MutationPermit没有生产构造器/default/反序列化，恢复核心源文件本轮无变更。没有用单个内核exit0、私有JSON或audit PID冒充整组生命周期证书。

## 独立声明与辅助观察

Original scope新增启动前独立声明的base路由候选；只接受当前静态明确selectedModels、liveModels=false、defaultAliases=false且无已知别名/combo/custom/disabled复杂图。provider键合法、模型ID有界、编码后唯一。模型内斜杠按官方2.99 providers/slug-codec的routedSlug映射为连字符；native/family与native-family碰撞时返回未知，不能按slash启发式授权。动态/不完整/复杂图明确未知并拒绝准备，不从生成后的catalog反推候选。

这只是后继完整图支持前的准备层，不是取消动态/别名/Fast或其它渠道范围。最终完整功能仍须对应官方真实图/别名和derived模型。现有共享内容规则要求独立候选与实际Ready中全部非原native行精确覆盖；未声明的Fast/effort派生项仍整批拒绝。

就绪live新增实际观察到的父进程Generation（同uid/PID/出生秒微秒），在捕获前后重核服务→parent关系及父代次，替代停止后再查已退出PID。prepare使用这些已保存的服务/parent PID做audit辅助一致性检查；它们是实际观察，不是原句柄/Job所证明的完整retained成员集合，不能据此签发MutationPermit。原件与live新必需字段不为旧开发frame补写。

audit路径保持明确Mac用户根替换，对敏感/token样式片段或超过256UTF16单位的路径拒绝，不尝试不完整secret清洗后默许。该辅助投影还不是完整官方redactUserPath接口适配，正式全部路径范围仍待补齐。audit从来不是文件写者权限证书。

## 独立索引引用与旧结构拒绝

索引结构升级为user_version=2，Head显式增加restorePlan(id,sha256)，RestorePrepared仅可由Stopped及相同原件References提交，revision单调增加。已有plan、旧expected文档/不同revision、非法UUID/hash、未完成阶段或rebase原件均拒绝。其余阶段不能携带plan；RestorePrepared不能走普通Failed清除/重建原引用。重开仍按索引独立摘要验证原plan及全部原阶段，禁止扫描原件目录挑选新plan。

schema1是开发中前一结构，明确拒绝，不迁移、不猜字段、没有默认补齐。用户已采用不做旧版兼容/自动迁移，本批不修改旧目录或现有日常索引。尚无Complete或清会话入口，准备失败/中断在独立引用提交前的现场仍保留未确认；后继显式恢复必须按原独立引用/事务处理，不能从preparing文件推导新授权。

## 验证与本轮发现

21项Session/Index检查通过，包括plan独立引用只提交一次/重开保持原References、早期/非法plan拒绝、旧schema不迁移、实际进程互斥/突退及旧引用保护；原20项恢复机械回归通过，本批没有重写其生产算法。静态编码碰撞/未知图、audit有限投影、父Generation格式和原件绑定也覆盖在定向检查中。最终编译、格式及全目标严格Clippy通过。

早期错误仅有tool原输出及preflight分类、没有全量原stderr文件的边界明确记录，不能称所有首错原字节都归档。一次独立cargo运行继承的系统TMPDIR不位于测试固定/private/tmp，随后隔离运行又拒绝硬编码/private/tmp逃出夹具写集；最终fixture从任务TMPDIR创建，并检查原canonical临时父目录和本fixture前缀。没有放宽产品owner/ACL/链接/真实账户或native断言。旧失败输出保留。

真实产品第一轮：Start15598ms、Status1094ms、正常Stop6061ms，但计划未产生，索引Stopped/revision5。现场确认启动前Scope有明确base路由，官方随后补写fastRows=true使旧独立声明函数返回未知；四阶段字节与元数据本身满足共享内容规则。按原索引独立摘要核对原件后，以既有只读回放入口检查实际Mac四阶段，内容规则通过。该回放入口函数名/固定旧输出仍含Windows字样，本次输入明确是Mac数据投影，没有Windows原生执行或新Windows验收。

原样官方configSchema和slug-codec纯只读接口另核实：省略fastRows与true实际等价，编码结果Team_One/native-family-5.5、碰撞识别true。首个空providers探针被defaultProvider规则拒绝，保留首错；后继用自有有效合成配置验证，不改官方Schema。

修正只接受fastRows合法布尔规范化对独立base路由的等价，不放行任何未声明derived行。新私有profile增量复验复用首次已准入的原样镜像并再次完整核验，不重复约70秒供应：

| 项目 | 最终真实结果 |
| --- | --- |
| 应用Start含原件/Ready提交 | 13413ms，原30秒，真实Started。 |
| Status | 1105ms，同原PID53348/port61203，ready=true，原5秒。 |
| Stop含原正常exit0/Stopped/计划准备与独立提交 | 8491ms，原10秒；运行/PID记录缺席。 |
| 恢复计划 | RestorePrepared/revision6；唯一plan原件/四phase SHA与独立索引引用读回匹配；当前进程内重新open并验证同一plan。原件stage原字节/原生inode匹配。 |
| 客户端恢复 | config.toml原字节保持；catalog/cache/audit仍未恢复，原assert保留false。 |
| 动作/整版 | client_original_restore_pending；整轮exit101/accepted=false，不把计划准备通过冒完整恢复。 |

固定官方2.99 tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c，descriptor e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3，准确actual source/binary摘要保留。第二轮为明确规范化假设的增量检查，第一轮原目录不覆盖；未调宽预算或因变成fail就删断言。

## 留证、清理与剩余

证据.adg/evidence/0.2.24/restore-plan-product-r1/含两轮真实产品前后像/索引SQLite/原事件文档、各自原scope/live/phase、最终plan及preparing原件、原stage字节、独立SHA/原生模式匹配、实际Mac数据回放、原官方pure probe及首错、准确source和编译/测试结果/manifest。runtime secret/admin token/完整含密钥配置未留证；provider/model声明及所有文件内容均为本轮明确合成输入。

完整Darwin TCP表framing确认两轮原主端点缺席，自有test/helper/Node/Bun镜像进程为空；所有精确新N/K/history DB均取得/释放SQLite排他后删除。副本runtime/profile/测试binary/索引/暂存原件及核对后的重复回执目录全部清理；未修改日常配置、正式安装、官方源码，不发模型上游请求/WebDAV业务同步。未构建新.app/DMG或发布。

下一步仍须完整受控进程集合/监听及实际原件写者许可，才能接apply、成功关闭会话和重启放行；不能给MutationPermit补任意布尔构造器绕过。还需一般静态/动态/别名/derived渠道、config/Grok/Agent/全树、活跃更新、新用户目录、跨卷和原flags、显式中断恢复入口。Windows最新仍b2e89665d/0eef59a01；W0 HOME/Profile/W1/W2、NAME、System代理、官方发布验证及固定候选保持原计划，目标不关闭。
