# 原始失败记录及修正

- 初次编译暴露源类型导入、私有 rename 模块与 digest 名称歧义；修正本片类型和实际 pending-lock 入口，没有跨改 owner 或降低检查。
- 第一次 local 回归 35 过、1 旧断言失败：在模型 protect 后制造的真实 preferences 漂移，被过早 scope hook 改报 SyncBackupFailed；将 hook 移至原第二轮所有 participant verify 与 mapping guard 之后、首真实 apply 之前，保原 SyncLocalChanged、原 before 与零业务写。后继 36 以及 40 项回归通过，未改旧测试断言。
- 扩大 manager 组 fixture 时缺 ManagerSelectionV2.webdav_password_ids 字面量，显式补空 Vec；不使用全局 allow、不跳过测试。
- 首次严格 Clippy 仅指出一个可折叠嵌套 if；按其建议合并条件，最终严格 Clippy 通过，无 allow/ignore。
- readonly 边界按总控后继裁决收敛：模型依赖检查只看 saved 图，草稿存在检查移至真正 restore 模型写锁。新增真实未知草稿/正规编辑会话/正式 fullCAS discard 测试，原 draft 和实际业务文件在拒绝期间原字节不变，同旧 plan 可在明确 discard 后恢复。

保留所有成功/失败原日志原字节。11 新测试与先前 40 项 local 回归存在重叠，不相加为不同测试总数；40 项回归早于最后 readonly 边界小修，后者用 11 项定向集与最终 admission 单项核验。既有双证明 remaining 案例在此批通过，未重跑全部 Service。
