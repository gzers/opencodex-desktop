# 第13项：新Desktop输出诊断后的user1单次验证

日期2026-11-09，Asia/Shanghai。用户已同意补脱敏诊断并交Windows做一次user1定向验证。accepted:false；本文件替代[上一交接](user1-start-stop-handoff-20261109.md)中固定旧Desktop产物的下一动作，四项回交和原判据保持。

## 已核查事实与本批边界

- Windows回交代码dc919da846260b0c11bebc04c56fd32963929d45、文档d4497d3ed及独立RDP对照已读取，原清单209条文件摘要核对一致。两个实际产品批次均只有1条失败启动回执，无运行记录；实际exit1分别8028ms、10325ms，后续30秒确认失败不覆盖首个退出事实。
- 最新宿主实际Caption为Windows Server 2025 Datacenter/build26100，以原host-environment.json为准；旧根README将其写成Windows11的描述不作为环境事实。RDP全程Active时仍失败，断线不是必要条件。
- 本次单用户账户/Profile/目录已清理；不据此关闭历史第18项，不先独立返修账户清理。正常Stop尚未到达，不计通过。
- 源码已确认存在诊断缺口：stdout/stderr共用尾部会影响分片标记，退出观察与输出drain异步；日志只导出原固定归属标记，标记之前的错误无结构化摘要。尚不能据此认定Windows实际exit1由哪一条错误触发。

## 代码及制品

- 新代码codex/0.2.23-integration@a4abf17aa7aa2b9f677c20aa3d30cda59deed5b4。8dfcc5281为产品诊断，a4abf17aa为单用户装配／来源采集。
- 产品只补输出流字节数、EOF／读取失败、固定错误kind／hint、允许列出的源码suffix及数字行列；不导出原始错误文本、符号、栈、用户路径或密钥。各流独立识别固定归属标记。非零退出后收尾最多100ms且不超过原预算剩余量，日志保留exit_observed_ms；没有延长启动／停止预算或降低就绪、ACL、Job、租约判据。未知信息仍写unobserved。
- 运行包仍为2.97.0-desktop023.lifecycle.2，15413257字节，SHA256 c02f4710496775d8f4dfe271bd90d6afdf7dbf960f6e696637271ab9f37bc70b。双平台policy及维护包源码未改；本轮不替换维护包。
- **必须先构建新源码Desktop，旧artifact13667461176不包含本批诊断，不能再拿它运行。** Mac未派任何Windows原生或core流程。

## Windows下一步：先构建，再仅运行一次

1. fetch代码和docs/governance-main，确认完整新代码SHA与本交接；保留自身未提交内容。只做新源码完整程序准备，可从该SHA派现有windows-candidate.yml的purpose=core-build（仅构建，不是core-sweep）。必须核实实际run.head_sha等于上述SHA；等待准备完整成功，下载名称windows023-core-program-a4abf17aa7aa2b9f677c20aa3d30cda59deed5b4的artifact，记录实际run/artifact ID、ZIP大小／SHA256，验证内含core-build.json全成员、appsTree、policy与运行包摘要。不得选择默认core-sweep或candidate。新制品身份未产生前保持未知，不沿用旧run或旧ZIP摘要。
2. 在已连接且可用的真实桌面上，只用一个隔离普通user1，按既有原生Profile、令牌与Desktop process_action启动路径执行一次。现有test/smoke/windows023-user1-start-stop.ps1增加-ExpectedSource与-ExpectedBuildRun，必须显式传上述新SHA和实际准备run；保持新鲜批次目录。不可直接运行双用户主脚本或需要peer的Child流程。
3. 启动确证成功才立即正常Stop。仍回交启动原回执／数量／唯一性／PID端口、启动trace、停止trace及真实退出码、账户/Profile清理和残留PID／镜像／SID四项；不以bootstrap exit0或tasklist推断产品通过。
4. 原app.log需保留startup_output=的stdout/stderr bytes、eof、read_failed、kind、hint、site和exit_observed_ms，同时保留原lifetime_admission字段（若有）。若读取未完成、错误类别未知或没有具体原生stage，明确证据不足，不自动再跑、不猜根因。kind/hint/site只是日志观察，不是归属证明。
5. 用采集器test/smoke/windows023-user1-evidence.py归档，并传--artifact-id、--artifact-zip、--document-ref（实际读到的文档完整SHA）；实际source/run取自本次manifest和回执，拒绝互相矛盾。没提供的制品字段保持null，不自动填旧身份。回交独立docs/分支，批次目录沿.adg/evidence/0.2.23/user1-start-stop-20261109/<新批次>/，原失败不覆盖。

本轮不跑core-sweep、双SID、安装器、全量或旧失败轮次，不改产品归属逻辑，不碰日常配置，不跑真实业务同步，不发布。构建失败保留日志，不为完成诊断去改产品语义。Mac收到四项和新日志后才决定第13项修复位置；第18项继续等待因果结论。

## 本机验证范围

17项进程／输出诊断测试、8项动作DTO测试、2项collector反例通过，strictClippy通过，PowerShell7 AST解析通过。仅本机隔离fixture／解析／编译检查，不替代Windows真实执行；初次新增模块借用检查失败及修正后日志分别保留，见[证据](../../../.adg/evidence/0.2.23/start-output-diagnostic-20261109/verification.json)。Skill参考development-knowledge的Windows进程状态与锁知识；没有执行其来源资料中的命令，也未将PowerShell7解析冒充WindowsPowerShell5.1实跑。
