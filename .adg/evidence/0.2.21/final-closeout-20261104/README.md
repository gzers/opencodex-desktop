Closed: 0.2.21 stable / Latest

Actual closure: formal-closeout-result.json. Three TASK/RCP/CERT sets, DMD and IMP are completed; REL24 is verified. Source 6e2197fd0b8a36dcb8d893b62693c5a2a77151cc, release 433749817, PR68 merged 7a8441f7e647e4b3540d5907e44ac4f186f90998. All8 AC /17 checks passed; public tag/Latest and original8assets verified. Source-and-raw archive preserves the prior preparation and original reports. The following section is the historical preparation procedure, not the current lifecycle.

# 0.2.21 正式关闭准备（未执行关闭）

最新6e原包/86默认npm实际链的接受输入见 BRIEF-PROGRESS.md、final-gates.progress.json、evidence-map.progress.json 和 source-brief.progress.json。旧43记录仅保留历史，不能替代产品已变后的86链。此处通用使用步骤保持，当前仍只计划不关闭。

唯一写集为本目录。真实代码根 ocx-0219-fixes，文档根 governance-main。仅适用于 DMD-UI-021 / IMP29 / REL24 和既存三个 UI021 TASK；不新增治理框架，不创建 TASK、不重绑、不提交推送。使用系统 Python3 标准库及实际 kernel，不依赖 PyYAML，不编辑正式 Markdown。

冻结来源在 frozen-task-contracts.json：MODALS 绑定 AC07 / 4 checks；SYNC 绑定 AC03 / 5 checks；INTEGRATION 绑定其余6 AC / 8 checks。合计8 AC、17 required_checks，逐项使用原始文字和原 baseline 文件摘要。

## 真实通过与待补

43c6253b582c3fe19ba67c8d1ccfeb4adabd7068 的真实 save → prepare → 新官方 Codex 默认/目录 → 实际新 provider / raw model / Responses 响应已通过，证据为 owned-client-routing-20261104/FINAL-43.md 和 production-prepare-43-result.json。模板引用这项并保留 global restart 未调用、真实 pending 保留、copy ack 是 mock 的边界，不写成真实桌面重启。26 actual native adapter / 21 parent 分层证据不能代表整版。父级告知 Windows Unix-only 测试符号边界已以1cf修正，CI41800125918回归中；本目录不代判 CI 结果。

最终产品源/包仍待绑定。Windows 实际官方默认 binding / 不改绑的原生准入、两平台最终原包原生、MSI/NSIS × 默认/自定义四格、Mac/MSI/NSIS实际0.2.20→0.2.21 OTA、最终CI、stable与匿名tag/Latest读回均不得提前填通过。旧debug包、编译、mock、跳过和局部通过不能替代。

## 父级补齐的输入

复制两个 .template.json 为本目录 final-gates.json、evidence-map.json 后填真实值。候选证据和历史失败保留；candidateEvidence/retainedFailures 不被当作接受证据。

- final-gates.json：最终 source commit/tree、release ID、原Windows EXE SHA256、带时区的真实完成时间/结论，以及公开stable后 verify-uploaded-assets.py 原样 asset-verification.json 路径/摘要。报告须实际显示8资产ID/字节/摘要、3签名、5平台及原Mac封包身份。
- 13个gate：按 requiredFacts 填实际 facts、tested_commit、证据路径/摘要和 claims JSON pointer。每项事实、来源、release ID都须由哈希JSON证据证明；provider不要求未来release ID。原生/安装/OTA另绑定指定上传资产摘要与原始EXE身份，不能换包。
- 43 provider链可以复用：工具实际比较43与最终源的既定产品/构建输入，完全相同且写明边界才接受。其它平台/CI要求最终source tree。
- evidence-map.json：8 AC和17 checks逐行填 passed、真实结论、非空evidence。证据必须是本版实际文件及SHA256，可附JSON断言；不接收旧证书作为关闭依据。

一项gate可由多份真实JSON共同证明。claims的键为 tested_commit、release_id、facts/具体字段，以及原生/安装/OTA的 executable_sha256；值为对应JSON pointer。字符串直接读值，也支持集合长度和精确状态比较，例如：

    "facts/uploadedAssets": {"pointer": "/assets", "length": true}
    "facts/macDeepStrictCodesign": {"pointer": "/macBundle/deepStrictSignature", "equals": "pass"}

未被指针证明的字段会拒绝。可使用父级汇总的真实最终回执，但必须保留原始日志/制品/运行身份来源，不得捏造完成。工具不联网重新验收，只验证输入字节、实际Git身份及确定性合同。

## 父级先回写源事实

仅在真实最终gates完成后进行。工具不改Markdown：

1. 把最终人类结论、标题、summary及关系先写DMD/IMP，暂保持in_progress，用实际 sync_source_bound_object + CAS put 同步两份投影。AC文字/关系及TASK绑定不改。
2. 再只改变DMD/IMP生命周期字段为最终值：state为completed；DMD非空resolution、next_action为null；IMP非空completion_summary。REL24写实际verified、原清单写最终逐项事实/限制。前言保持每行JSON值。runtime投影仍open，实际关闭待明确执行。
3. --prepare捕获最终文件after_digest、两个open投影与原基线到最终源的代码变化。审阅三TASK changed_paths：默认保守列出各自范围内候选，可收窄归属，但并集必须覆盖实际代码差异，每TASK须包含自己全部fact targets。其它对话脏文档不扫描、不纳入。

这是实际kernel的派生生命周期合同：最终关闭同一步不得再改summary/title/关系。输入变化后以新文件名重新prepare，保留旧计划，不重绑TASK。

## 最小命令

在本目录运行。prepare只捕获；普通命令只校验/plan，未满足返回2、满足返回0。只可新建本目录输出，不覆盖证据。

    python3 -B formal-closeout.py --gates final-gates.json --map evidence-map.json --prepare final-bindings.json
    python3 -B formal-closeout.py --gates final-gates.json --map evidence-map.json --bindings final-bindings.json --output final-plan.json

父级确认同输入plan全过后，唯一实际关闭命令（本worker未运行）：

    python3 -B formal-closeout.py --gates final-gates.json --map evidence-map.json --bindings final-bindings.json --commit --output formal-closeout-result.json

真实 RuntimeStore.commit_task_closure 逐TASK原子写入本版新RCP/CERT，前后检查源/范围/证据/CAS，再按IMP→DMD同步父级已写好的终态投影。三个TASK没有跨组原子事务；中断保留已写项并停止，不输出整版成功。保持原输入/时间再次执行时，只接受逐字节等于该计划的已存回执/证书；不复用旧证书、不替换输入。无发布、网络、日常配置或Git写操作。

## 准备验证

test_closeout.py只测试准备/拒绝路径，真实kernel只读核对TASK、criteria和baseline；未执行close_task、commit_task_closure或put。默认计划保存在pending-plan.json，runtime_written为false。13最终gate、8 AC、17 checks在该模板计划中仍待最终输入；43实际链的分层通过单列保留。

prepared-current-not-final.json仅验证prepare能够读取真实根/当前投影/范围并捕获摘要；其中仍为当前阶段事实和模板摘要，不能作为最终关闭输入。父级完成上述最终回写后，须重新生成final-bindings.json。
