# 原始失败与修复依据

- auth-none-before-owner-fix-failed.log：原真实接收把新来源 authMode:none 丢失为 None，第二次 compose 返回 model_receive_credentials_direction。修复来自实际接收 owner 的精确提交 202e449aa54249a5983f17740e9bec20ee5182d7；合入后 auth-none-after-exact-owner-fix.log 通过。测试来源仍为 none，未改成 key。
- same-id-first-guard-failed.log / same-id-unsaved-draft-diagnostic.log：实际失败为 model_receive_unsaved_draft。隔离环境 begin_draft 仅为建立稳定 UUID，却留下初始化草稿。对本次明确无用户编辑场景，仅移除该初始化草稿，保留原 raw config、UUID registry、auth mode；生产未保存草稿守卫未放宽。same-id-noop-and-confirmed-download.log 是最终通过记录，临时诊断输出已移除。
- 真实重启案例在实际 model receipt 和 adopted mapping 证明之后、manager/assets 之前用 test-only fault 中断。销毁 Service 和 preview 后破坏当前 H/config 为无效 JSON，新 Service 仅完成原操作剩余参与者，网络计数不变、原模型回执与当前坏配置字节不变。改写 context 的 mapping-after 并重算其校验也被真实双证明拒绝，恢复原 context 后同 O 可重复取得回执。
- 首轮严格 Clippy 对 cloud.is_none 后 expect 报告不必要解包；最终改为 if let Some(cloud)，未屏蔽 lint。最终普通 crate Clippy 记录见 normal-crate-adopted-service-clippy.log。

保留失败日志原字节。以上通过仅对应所述目标案例，不代表整版验收、真实服务器或发布完成。
