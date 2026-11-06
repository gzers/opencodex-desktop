# L3 G0/R0回交摘要

2026-11-06，来源为用户转入本对话的L3正式回交，source_thread_id=01a1b0c0-f5de-7c53-89a3-cad653af6685。本记录保留回交语义，不将其视为运行/视觉测试通过或正式R1放行；未声称总控已读取该对话的其它内容。

## 当前事实

L3只读核对TEAM、IMP-31/32/33、三DMD及相关核心/原型说明。参考代码0c57500f8f46ea50655c782c3f04334a39167abb、检出干净；文档读到HEAD 43c4068b5后的工作区字节，知悉本轮总控提交，不碰索引/共享树。没有创建工作树、编码、修改文件、运行产品/视觉测试或启动临时服务，0网络/日常副作用。以下为方案/源码检查，不是最终验收结论。

## 主要缺口及接口需求

- 完整数字版本/名称片段/位置/AND编辑器仍缺，真实编辑器在routes/ModelsRoute.vue；不能只改variants.suffix标签。
- NewChannelDialog、ChannelEditor、upstream和useNodeMeasurements等现有链依赖connection.protocol，必须全消费者接线；临时清单方式默认未选，不作为渠道/便携协议持久化。
- 现有读取调用models.load，需后端同源返回候选草稿、来源/人工覆盖/自动引用、匹配计数和读取基准，UI不复制匹配引擎；失败/空/晚回/冲突保护。
- 目录界面仍含migrate_data、旧默认runtime和自动发现恢复，须按L0新路径快照/有效性接线，不扫描旧目录。
- 安装身份恢复不等同偏好备份或JSON手工恢复；需独立诊断/明确来源/保护/服务前置/处理中取消边界/成功读回/失败保留状态。
- 同步预览需方向下拉、有效远端事实/可靠时间/代次基线和可执行理由；原操作只读核对不能调用新publish。必须保留同名MCP来源选择，切方向不丢选源。

L1经L0提供正式条件AST、精确revision、逐条件/字段来源和错误；L2经L0提供observe/prepare、远端有效性、提交/代次和原操作核对/恢复能力；L0提供唯一数据根/HOME/受管程序快照及恢复prepare/apply/status/cancel合同。正式字段/枚举未由L3擅定。

## 拟写集补充与总控处置

L3申请models/sync业务文件之外的routes/ModelsRoute.vue、SettingsRoute.vue、features/data-root/api.ts与store.ts、features/runtime/api.ts和RuntimeInstallDialog.vue；必要BackupFiles.vue依合同diff确认。src/stores/app.ts为共享门面，由L0单写或显式转交，不由L3扩写。

总控将这些现存消费者加入TEAM拟定分配描述；**不是生效机器写权**。R1仍需真实代码/文档SHA、合同和采用原型摘要、逐文件独占路径及必需检查。原型贡献在独立docs checkout，index/maintenance挂接须精确授权，不携带其它并行探索；核心/索引/IMP由L0写。

## 验证建议与来源时点

R1先补原型必要状态，登记采用后做真实组件；R2共享接口注册后接实际API。定向typecheck/Vitest、Chromium/WebKit亮暗和1180×760/900×650、项目缩放/13px字号、长ID/路径、焦点/底栏滚动；保存读回、取消零写、晚回/来源冲突/后续本机修改及MCP来源保留。Windows两SID/权限由L4验，不用页面模拟替代。

回交时model-config.js SHA256=7b9fa5555b330ced88b419d1d8324b4719b3394b8a0ee9de8dfd107262f3c1fa，候选JSON=aeeec2078ff1416f587019c376cc06b17259550e47db199f1e59a7fff8aae68b；index/sync包含未整体采用增量，不以最新整页摘要放行。TEAM/IMP摘要会随本次总控提交变化，L3须以本次推送真实文档SHA重新核对，旧HEAD不是冻结基线。
