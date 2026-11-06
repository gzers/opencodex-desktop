# L1 对实际消费门的有界复核与修复回交

2026-11-06，Asia/Shanghai。只读复核L0 integration工作区config_file_v2.rs，不修改该文件、gate tests、命令或transaction。范围为接口、source/人工/ledger配对、读取幂等与anchor正常流，不是整版审计或独立全体验收。

发现并已向L0具体回报：

1. 观察时config_file_v2.rs:431～440：旧anchor组消失时直接将protocol/wire改为既有nonanchor组键，旧nonanchor mapping仍保留，随后validate必报重复映射，导致正常“2组→只剩原nonanchor”无法保存。L0已接受并决定保留持久anchor identity，由L1编译器支持其停用与恢复。
2. 观察时config_file_v2.rs:226～246：再次Read只恢复部分draft字段，stageConnection中未保存的连接/节点/认证引用/选择/顺序/新增对象被observed取代。L0已接受并负责保留完整非source人工draft意图。这里引用观察时行号，不声称L0并行修改后的当前行号未变。

ledger正常接线核对：gate用state.policy及其semantic摘要读取/恢复source，unsaved draft意图在其上合并；编译输出compiled.policy用于新ledger和Store prepare，方向一致。没有将draft source当作可信已保存ledger基准的确定错配。原件已在第一次字节解析使用parse_strict_json_v2，重复键边界已正确接入。此结论是静态接口核对，不声称真实gate/transaction/命令测试已执行。

本线额外确认compiler缺陷：已有派生provider保留旧apiKey/headers，不随逻辑连接轮换。新回归在修复前失败，输出保留gate-review-credential-before.log。此项与anchor编译修改按L0要求同笔回交。

提交：7abd03040768405b36ce6ad74ab7646b47154f88（codex/0.2.23-model-matching），父3e32bb1e437168dc248177c40b49ebcfc08432ee。实际写集仅compiler_v2.rs、provider_import.rs、consumer_v2_tests.rs，工作树干净。

- 失活anchor/非anchor都disabled并清空已管型号；anchor无marker要求，不删除认证、原note或未知字段。mapping身份不变。
- 默认运行provider跟随实际defaultModel所在活动组，否则稳定首个active provider；零active组时保留停用anchor并移除该channel的失效defaultModel，不伪造型号。
- reader在anchor空停用而其它组活动时，按实际承载型号组的enabled恢复逻辑渠道，不把anchor停用误认整个渠道停用。
- 已有非anchor组从同逻辑anchor的实际连接材料同步apiKey/apiKeyPool/headers/proxy/authMode/apiKeyTransport/apiKeyPoolStrategy/keyOptional，轮换/删除均生效，仍由实际auth允许集前置验证。

验证：正常应用crate源码快照15消费者测试通过，0失败/忽略；此前12项回验，加“2组→仅原nonanchor→回anchor”“凭据轮换及清除”“零active组不造型号”3项。验证mapping不变、重开启用状态、重复compile稳定、原ID/选择/原note/认证/未知字段保持。Clippy正常crate --lib --tests --no-deps -- -D warnings通过，rustfmt及diff检查通过。

快照是前批受控完整crate基线，只覆盖本线实际源码；只额外注册本线新模块。未导入L0未提交gate或transaction，不冒充实际L0统一门通过。证据：gate-review-credential-before.log、gate-review-final-consumers.log、gate-review-clippy.log。同目录保留。没有网络/上游/日常配置/发布，测试temp-root自动清理，无临时服务。代码及结论已直接回报L0。

