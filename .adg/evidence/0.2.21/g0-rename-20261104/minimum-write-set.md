# 改名正式实施的最小写集

本表是 G0 实测后的实施接续，不是已修改的生产功能。无需再批准旧标识兼容或历史迁移；按已确认合同实施即可。探针的实际成功写集为 4 文件＋4 账户，缺失可选引用时为 2 文件＋账户。

## 业务落盘集合

| 对象 | 最小改动 | 保持与验证 |
| --- | --- | --- |
| 官方 home/config.json（必写） | 将已有 provider 原对象从 old 键搬到明确目标键；已有 name 回显跟随名称；精确替换 defaultProvider；modelPickerOrder / disabledModels 只换完整 provider 段，原 suffix 和位置保留；已识别私密字段中 provider-owned keychain 引用重绑 | 不重建模型 ID、模型规则键、alias、请求地址或未知字段；不全 JSON 字符串替换；默认 channel.id 未变时也必须改 defaultProvider |
| manager-state/model-channel-layout.json（必写） | 仅在同一 channel.id 上更新 label 和 connection.providerId | channel/node/template/采用关系及已有 UUID 凭据引用保持；不能让未改 layout 的 raw 键改名触发 deterministic stable_id 重建 |
| manager-state/preferences.json（条件写） | 已保存 modelTemplateAgent 引用 old 时更新 providerId 和 connectionDigest；稳定 channelId、modelId 及其它偏好保持 | 调用既有 preferences 事务 guard；用当前实际运行绑定及目标原始 provider 连接字段重新计算摘要，并与 execution_directory::bind 校验；不能遗留旧摘要 |
| manager-state/model-execution-source-ids.json（既存且引用 old 才写） | provider 键 old→target，保留原 UUID | 不为改名新建退休登记表；若原 UUID 与稳定 channel.id 冲突，阻断并重新核对 |
| provider-owned 安全账户（有引用才迁移） | service=opencodex.provider-api-key.v1；old 或 old/suffix→target 或 target/suffix；先创建读回再提交文件，后清理旧位 | 目标已存在不得覆盖；API Key/headers/proxy/MCP/executor 等已识别引用按归属处理；认证内容、池 slot.id/顺序/label/addedAt/活动选择不改；UUID-owned 凭据账户不迁移 |
| manager-state/saved-content-versions.json（正式保存须接入） | 在真实内容保存成功事务中登记新 scope/contentDigest 和可靠保存事实 | 与判新 G0 的正式合同接续；不能保留改名前摘要或用重启/打包时间充作保存时间。本次仅验证改名事务，不伪造已完成保存时间方案 |
| 事务 journal / 精确 before 保护文件（技术产物） | 受控根内持久化写集、摘要、账户迁移归属及提交决定；恢复可判定 rollback / committed cleanup | 0600 文件、0700 目录；日志不含凭据值；只恢复本事务仍拥有的当前字节，漂移明确返回待恢复 |

model-protocol-mappings.json 是读/占用/漂移校验输入，本次改名不改其 id、channelId、protocol、variant 或自身 providerId。协议分组的运行 provider 有独立稳定身份；正式规划须按同一 channel.id/协议找到已有分组，检查当前父引用，不能因父名称变化重建分组。跨归属共享安全引用须先阻断，不能取消 provider_auth 归属检查。

## 生产源码改动集合

| 位置（apps/desktop 下） | 直接实施方式 |
| --- | --- |
| tauri/src/modules/model_config/config_file.rs | scoped_policy 按稳定 ID 合并目标渠道，允许经过验证的 provider 改名，仍阻止换管理身份、占用冲突及越界凭据编辑；scope_picker 同时识别 old/new，不能遗留 old 或重排其它渠道。将现有 raw/layout 回滚扩为上述完整事务，所有入口共用，不只松开 XOR 条件。seed_imports 的旧 receipt 阻断保留，必要时重新准备材料。 |
| tauri/src/modules/model_config/config_file_mapping.rs | 增加纯 RenamePlan/预处理：验证后搬原 raw 对象和精确引用，并形成 provider 已归一到新键的 mapping_before；普通模型差量继续复用现有 mapper。保留 identity/保护检查，只允许已验证 rename 映射；不要删除两处旧断言后直接放行。默认引用更新不能只依赖 default_channel_id 变化；picker 改名保持原位置和完整原始模型后缀。 |
| tauri/src/modules/model_config/新事务辅助模块（或 config_file 内部模块） | 提取探针的 plan / execute / journal recovery 职责，复用 atomic_write、TargetFileLock、backup::manager preferences guard、safety 和可注入 credential store；正式锁序在全部参与者中一致。目标文件及凭据摘要需在确认时重验；提交失败和清理待补偿回执区分。 |
| tauri/src/modules/model_config/execution_directory.rs、agent.rs | 将已有纯 connection_binding helper 暴露给同模块事务复用，保持一个摘要算法；Agent 偏好直接纳入事务，不在文件提交后另起不受保护的 save_settings。探针临时镜像 helper 已与真实 bound_directory/bind 验证相等，生产不保留第二份公式。 |
| tauri/src/modules/model_config/provider_auth.rs（按复用需要） | 可复用/提取当前 owner 检查与 Key 池身份规则；使用可注入账户读写边界完成 create/readback/delete，禁止新引用字符串重新 hash 池 ID。保留跨 provider 安全阻断。无需改 SafeStore 的保护 envelope/历史备份格式。 |
| tauri/src/modules/model_config/content_versions.rs 及真实保存调用点 | 与同步判新工作合并一个参与事务的保存版本计划，复用同一内容摘要事实；不在改名模块自造时间/第二套版本记录。 |
| ui/src/features/models/ChannelEditor.vue 及现有 models 保存消费者 | 按冻结原型接入名称＋明确目标标识预览/校验；保持草稿以 channel.id 定位，单渠道只提交本渠道，取消无写入；单渠道/全页共用后端事务，成功提示重启。不能只去掉 readonly 或只改 label。 |
| 对应 tests / config_file_scoped_tests | 保留本次旧行为证据，将旧 provider 固定断言改成新合法改名与非法换身份的成对测试；引入本探针成功、真实文件错误、凭据错误、冲突/保护、取消、漂移和恢复用例。Key 池身份基线继续保留。 |

既有协议分组/便携传输模块无须为改名扩大格式：本次实际导出仍传稳定 channel.id，不在 connection wire 增加源 providerId 或 UUID 凭据引用；不承诺跨设备强制改接收端运行标识。原始模型 ID、节点、模版、认证内容和历史会话目录都不进入改名重写集合。

正式落地时以同一事务同时组合普通编辑差量与改名计划。本探针验证的是改名事务及全页批量改名，并未把生产 save 入口替换成功，也未声称任意其它表单编辑组合已经通过。原生 Keychain/Windows 凭据、非合作客户端读写与重启后的真实目录/路由按 AC-021-02/08 补验证；G0 不运行日常 Codex。
