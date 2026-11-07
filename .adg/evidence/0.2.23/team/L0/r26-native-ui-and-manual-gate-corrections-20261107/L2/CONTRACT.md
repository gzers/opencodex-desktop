# 本批 producer 合同

源码 da4f0890b6916cc82ba137e28205ea9fe1e1338b。采用 Main 精确基线 47eb7cf7420ba9127f4d7047e4d83231f826035f。只改 file_transfer_v2.rs、sync_local_v2.rs 和各自 tests。

assetsReviewed 保留现有 v2 required bool wire。CaptureRequestV2 / ScopeRequestV2 / PreviewRequestV2 / ImportBindingsV2 调用方继续显式发送 false；不改 Option、不默认为 true。ImportBindingsV2::default() 为 false，前端 empty factories 也为 false。不需要一次额外人工复核动作，true/false 都不赋来源真实性或权限。

该字段以 deprecated manual-review declaration 文档语义保留。原 request/choices/scope hash 和冻结上下文字节不重写；原 O 的重试仍检查同一 scope/O/planDigest/source/CAS，不为了继续而替换 true。这里没有新增旧格式读取、迁移或兼容功能。

本批只删除人工门：FileOwner 对合法 narrow_received projection 一律计算真实 target_digest，不以 bool 决定是否核目标；不再返回 capture_assets_review_required。LocalApplication new/new_mapped/new_mapped_remaining 不再仅因资产存在且 false 拒绝。真实 issued mapping、全部来源/路径/摘要/冲突/凭据子项、范围/保护/原操作守卫完整保留。

Main/L3 的 UI 改动由其单写：删除额外复核 checkbox，不把所有 caller 值硬设 true；展示实际来源和差异，来源冲突保持未选，不代用户选择。includeGlobalTemplateRefs 仍维持当前明确选项的实际语义，本片不改此字段/模型语义，不恢复旧“全局采用及渠道组顺序”行。

本机 scopeprefs 的 validator 不判断 review bool，不需新增写集；Capture/Service DTO 的同义文档注释由 Main 后继处理，无需 wire 变化。
