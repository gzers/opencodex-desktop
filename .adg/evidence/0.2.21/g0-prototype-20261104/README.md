# 0.2.21 G0 原型交付证据

范围：唯一活动总原型的 POST01、POST02、POST04、POST07、POST08；原型只使用内存 mock。软件实现、真实文件/凭据事务、真实客户端重启与原生验收由主线处理。本项不更新 UI规范、项目 README、DMD、IMP 或问题清单，没有 Git 暂存、提交、推送或发布。

## 来源与可重放增量

- baseline.json 记录修改前 docs/governance-main 的 HEAD 和 122 份原型文件摘要；preimage/ 是本次原始前像，保留当时其它对话已有改动。preexisting-prototype.diff 和 status-before.txt 保留原有工作区事实。
- source-manifest.json 是最新采用身份：20 个 JS/HTML/测试文件的前像、固定交付及活动工作区摘要。final/ 仅保存这 20 份固定证据副本。
- self.diff 相对本次原始前像生成，不等于整个 Git 脏工作区差分；新文件以 /dev/null 为前像。已在 delta-replay/ 实际重放，delta-replay.json 为 20/20 文件 SHA-256 一致；不是在 cf5b 或主线 HEAD 上重放的证明。
- 活动 index.html 后来引入 overview-platform-data.js 并改变 overview-journey.js 查询版本。本项保留这些并行改动，未补造缺失依赖；独立差分见 index.html.since-core.diff。该增量从 final/index.html 与 self.diff 的 G0 采用范围排除。
- 固定来源 QA 只读映射：本项修改取 final/，未修改的原型依赖取 preimage/，目录外既有素材从项目原位读取。snapshot-source.cjs、frozen-reader.cjs、serve-g0.cjs 仅为证据回放工具，没有回退活动原型。主线仍需在 cf5b/实际 HEAD 上重放本项增量后复测实际采用集，不整包采用脏工作区或证据副本。

## 行为与入口

| 验收 | 本项原型状态与核对方法 |
| --- | --- |
| AC-021-01 / POST01 | 保存成功不自动聚焦立即重启；终态 8 秒共用计时/进度。悬停、主动键盘、隐藏及重启确认暂停；确认 X/Escape/遮罩/取消仅结束确认并续计。到期不重启，稍后收起；旧通知/旧计时不关闭新保存。g0-prototype.test.cjs、既有通知回归及浏览器真实计时。 |
| AC-021-02 / POST02 | 渠道名称与运行标识联动直接改名；不合法名称可明确填合法标识，保留稳定 channel.id。就地合法、官方保留及当前/派生占用校验；详情和全页预览显示名称、运行标识与目标选择器，成功提示重启。节点、原始模型及认证引用保持。单渠道/全页、取消、失败/漂移、范围隔离有定向回归。真实配置引用/安全凭据事务不由原型证明。 |
| AC-021-03 / POST04 | 原同步预览显示可靠保存时间及来源；source 为 local/remote/unchanged/undetermined。内容相同无需更新；未知/坏时间、历史 mtime、同一 UTC 时刻异内容、绑定不明/时钟异常均说明原因、保留内容，确认更新可见但禁用。无新增选源弹窗；确认前重验保存版本、连接、所选范围。g0-sync-version.test.cjs 保留同范围时间、无基线对称来源及漂移反例。 |
| AC-021-05 / POST07 | 未配置、已配置未检测、正在检测、最近成功/失败区分。概览与同步同源；观察绑定保存端点 opaque revision，地址/目录/账号/凭据变化推进修订。未保存/候选检测不改当前观察；保存同一已测内容可沿用带时间证明；旧请求失效。“模拟重载”保留内存观察，不自动联网。线上版本未知不伪装空，历史版本注明“最近观察”及来源/时间。 |
| AC-021-06 / POST08 | 工具板有加载、成功空、读取错、非活动候选、缺密码；仅成功空隐藏入口。错误可重试，候选有数量及用途，确认启用才替换连接，缺密码阻止假成功。X/Escape/取消及关闭晚回不改表单/当前连接，本机范围保持。 |

入口仍为 docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html。新增运行依赖 g0-contracts.js，在模型及同步脚本前加载；场景复用右侧工具板，无新增产品页面或未来业务。

## 计时器遗留失败

baseline-notification-tests.log 原始复现：22 项中 21 通过、1 失败，保留 assert.ok(timer) 栈。用例只点“同步”停在预览，未确认执行，因此没有完成 Toast，timer 为 null；不是倒计时本体故障，也不是模型保存焦点问题。

修正 notification-countdown.test.cjs：先断言尚未执行、预览存在，再确认、等待完成并核对结果，然后继续原 timer/手动收起/铃铛实体/组件重置隔离断言。没有降低 timer 要求或绕过预览。core-tests-second.log 记录修正后通知及 G0 核心 41/41 通过。

## 检查记录与身份

- checkpoints/post04-core/：最早 POST04 核心，2/2 通过；其 null source 后来按主线对齐四种枚举，是历史记录。
- checkpoints/g0-core/：41/41 核心；browser-core106/ 为该阶段双引擎各 106 项，不覆盖之后“确认可见但禁用”等收口。
- all-harness-tests.log：活动脏工作区全量 511 项，374 通过、137 失败、0 跳过。129 项为并行 overview-platform-data.js ENOENT，另 8 项为旧改名夹具未明确合法目标。原始失败保留，不能称全量全绿。
- final-targeted-tests.log：固定来源扩展 171 项，168 通过、3 失败、0 跳过。两项为混合时间反例 fixture 变化，已恢复反例；一项为异步检测没有等待，已补等待，并恢复重试更新同一通知的行为。原始失败保留。
- final-sync-r2.log、final-sync-preview-r2.log 是收口期间补查，非最终统一来源结论。最终取 final-fixed-core-and-failures.log 及 verification-summary.json。
- chromium/result.json、webkit/result.json 是最新固定来源浏览器 QA；各含 6 个运行文件实际 HTTP 内容 SHA-256，与 source-manifest.json 对应。覆盖 1440×900、1180×760 暗色、900×650，各 16 张截图。旧记录保留在 browser-first/、browser-core106/、verification-r1/、verification-r2/。
- syntax-checks.json：19 个固定 JS/CJS 语法检查全过；delta-replay.json：本轮补丁在原始前像上重放 20/20 摘要一致。

最终固定来源：核心及此前失败路径 11/11、遗留 timer 定向复验 1/1，均 0 跳过；Chromium/WebKit 各 112/112，各有 16 张截图，6 个运行文件 HTTP 摘要均匹配最新 manifest。19 项语法检查、20 项前像补丁重放均通过。本项原型范围没有未完。

最终汇总和未覆盖项见 verification-summary.json。收尾检测到的后续并行 CSS/JS 查询版本变化保存在 concurrent-index-after-final.diff，活动文件继续保留，未计入 G0 固定来源。以上证据不等于生产隔离文件、Windows/macOS 原生、整版验收或发布；主线仍需在实际 Git 基线上重放并复测采用集。


主线提交归档说明：上述preimage/final/self.diff及原始失败日志以原相对路径和原始字节打包在raw-source-evidence.tar.gz，摘要见archive-manifest.json；旧工作区截图保留本机，不重复入库。实际Git重放的最终双引擎截图及结果归相邻g0-adoption-replay-20261104。
