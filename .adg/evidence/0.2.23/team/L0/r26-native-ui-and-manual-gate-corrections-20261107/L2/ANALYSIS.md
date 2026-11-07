# 本版人工复核门回归：原因、合同与修正边界

先只读核查于代码 47eb7cf7420ba9127f4d7047e4d83231f826035f、文档 fc3e3b672c583ab2e4d0292b04e6851c4df4c65f。Main 已有原生截图 sync-file.png 的底部两个技术 checkbox，原字节归档 native-sync-file-before.png。随后总控明确授权修正后端重引入的人工 review 门；本片仍不修改 UI/docs/schema/全局引用语义。

## 既有要求优先

- DMD-OPENCODEX-UI-020.md AC-020-01（frontmatter 第9行）明确：同时移除旧 UI 必经步骤、按钮门槛和后端人工确认契约；保留路径/软链/文件限额/凭据范围/解析校验，不得仅隐藏复选框或伪造“已人工确认”。第23行再次写明移除逐次人工复核。
- IMP-28:20 明确旧字段可读但不作为写入门槛，不伪造人工确认。0.2.20-bugs.md:35 的 S20-01 是原问题及决定，188 行是前后端旧门已移除的历史回交。本版不应以重新增加一个 bool 门撤销已确认交互。
- 当前原型 sync-config.js:278 的 reviewed 是空字符串，284 的注释明确“普通同步不要求逐次复核”。本版 L2 文件协议合同要求冻结 MCP 客户端/定义摘要和凭据/范围/CAS，没有要求增加一个人工勾选步骤。后台实现有新 bool 不等于用户批准新步骤。

## 实际代码：声明与真实校验的区别

| 链路 | 修正前 false 的实际效果 | 真正来源/内容证据 |
| --- | --- | --- |
| CaptureRequest/本机目录/保存范围/普通已保存文件导出 | validate_request 不检查 review；资产导出能用 false。不是每个导出都由它阻断 | 明确 Selection、MCP 客户端、definitionDigest、实际安全读源、凭据子项 |
| portable_assets::snapshot_skill:319 / split_mcp:370 | 参数名 _reviewed，未作为真实性依据；实际目录、软链、文件类型/限额、秘密过滤和解析仍执行 | 安全路径与实际文件/定义/秘密范围 |
| FileOwner import preview:930,970 | 有 Skill/MCP 时 false 跳过真实 target_digest 分支并返回 capture_assets_review_required | narrow_received、actualtarget images、selected client allowlist、摘要、目标/库存 CAS、同 O confirm |
| LocalApplication new:1049 / new_mapped:1099 / new_mapped_remaining:981 | 有 Skill/MCP 且 false 时报告 SyncScopeChanged，导致初始接收或原操作续做被人工声明阻断 | 原 owner-issued mapping、精确选择、root/source/target/scope/请求摘要和真实日志/双证明 |
| 原操作记录 | bool 参与 Capture scope_digest:502 及 request/context/choices 冻结 | 它能区分冻结意图的字节，但 true 本身不能证明任何人已审读、任何来源合法 |

因此“选定对象/已知准确来源 → actual preview → 确认原 O 与 planDigest”能够承接原有选择授权，不必增加单独 review 勾选。UI 应表达实际来源/冲突/内容，而非谎称人工已复核。仅把 caller false 填成 true 与正式要求冲突，也无法替代实际来源检查。

## 最小生产合同与修正

1. v2 wire 不改可选性：CaptureRequestV2、ScopeRequestV2、PreviewRequestV2 与 ImportBindingsV2 仍保留 required bool assetsReviewed。ImportBindings 的 Rust Default、UI empty factory 为 false，调用方继续显式发送 false；不改成 Option，不自动填 true。
2. ImportBindings 字段注释 deprecated manual-review declaration；true/false 都不给源权限。删除 FileOwner 的 bool 门，并对合法已投影资产范围一律计算原有真实 target_digest。删除 LocalApplication 三处资产存在且 false 拒绝的分支。
3. bool 仍作为原 DTO/冻结 context 中的既有字段保存；不重写旧 O 的 request/choices、不从恢复中补 true、不更改 scope hash 规则、不重编码原冻结载荷。这里只是本版 deprecated 字段，不增加旧文件格式迁移或兼容入口。
4. 本机 scopeprefs 只调用 validate_request，后者不判断 review，无额外门需开放写集。UI 与 producer DTO 的其它位置由 Main/L3按相同语义消费，本线不写。

## 必须保留的校验和 UI 前置

- 已选择对象/凭据身份、精确 revision/digest 及依赖闭包不变；未选对象/凭据不传、不接收，不拿“review done”扩围。
- 导出/同步 actual capture 多 MCP 来源必须明确选择。没有选择且不唯一，capture_mcp_source_required 保持；所选来源不存在/变化、definitionDigest 不匹配保持拒绝。不能自动选第一个或把 unresolved 视为 reviewed。
- 文件包本身的所选 MCP 定义已由 payload/digest 固定。接收客户端 choice 的已有 allowlist/目标路径保护保留；不能把 bool 当已选择客户端，也不在移除 review 的同时虚构新的接收客户端默认值或新增必经确认。
- 源/范围/连接/保护或输入改变，使预览失效并重算真实计划；界面确认必须提交当前同一 O/planDigest。取消不采用，未确认计划不写业务对象，partial 仅续原 O。
- 安全路径 no-follow、限额、凭据子集、读取错误/冲突、source/target/scope CAS、backup barrier、真实 participant receipt 和幂等都不修改。

## “全局”控件的独立语义核查

所述 sideRef / “全局应用及渠道组顺序”未出现在当前唯一总原型 sync-config.js，也未出现在 IMP-31 采用的 cde9cc52a 文件。0.2.20-bugs.md:37 的 S20-03 反而把旧右栏“全局采用及渠道组顺序”列为确认差异；190/216 记载范围由可见对象、依赖与顺序承载，所选渠道源全局引用折入渠道精确依赖，未选渠道与主版本保持。不能从旧侧栏字段名推定本版采用。

当前 v2 includeGlobalTemplateRefs 的真实含义也超出一个标签：

- saved capture:598 / portable_v2:195 将 Policy.template_refs 放入 portable global refs，channel_order/defaultChannel 的导出另有规则。
- objects_overlay_v2:361 合并全局引用；368 对所选渠道排序已有独立行为，375 仅无原 baseline 时采用默认渠道。
- receive_models_v2:728 同一开关还重排 source_to_target 映射的渠道，并采用包内可映射 defaultChannel；需要全部相关精确依赖。
- template_refs、adopt_template_defaults 与 include_global_template_refs 是三个独立授权概念；不应合并标签或强开此开关来冒用户同意全局修改。

本批按总控裁决保持该显式字段现有语义，不恢复旧侧栏行、不强制 true、不借 review 门修复顺带调整全局。Main/L3下一批需要按当前采用对象树冻结如何表达 global refs/顺序及未选影响，再决定表现；技术 checkbox 不是自动获批的产品控件。

## 验证口径

改前两条真实失败、后继受影响模块与严格 Clippy 的精确日志和源码提交见 receipt.json。没有用删除/ignore 旧失败断言收口：原“必须复核”的测试改为 false 可继续，同时继续验证真实 owner 发出范围和未授权扩围拒绝。新定向用真实 FileTransferOwner/ReceiveFileHost/LocalApplication 和隔离 D/H，不访问真实 WebDAV或日常数据；其中原 LocalApplication fake 模型夹具只作为 source/payload 的测试准备，新增实际应用使用真实 ReceiveFileHost 回执。
