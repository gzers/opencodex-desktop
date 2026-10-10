# 设置保存与默认恢复事件验证

日期：2026-10-10。源码：feature/0.1.10-maintenance@d4f4dc29eaef36700bb132f5a215f83102eb27e4。代码与文档分支各自提交；版本仍为 0.1.9，未合入 main。

## 实际范围

save_preferences / restore_default_preferences 在已准入的工作线程、偏好文件锁及更新状态锁内记录实际写入终态。保存与恢复默认使用不同 action；完整 Preferences 序列化内容经 SHA-256 生成候选，包含更新通道。同根世代、同动作、同候选成功才能解除旧异常。候选轮换后返回旧配置不会恢复旧 UUID，因此不推定解除历史旧异常。只保存摘要与固定安全文案，不把路径或偏好内容写入通知。

校验失败、通道切换忙 / 待重启、Windows CLI 规则及写者冻结等准入前拒绝只即时反馈，不强写冻结根。通知失败不更改已提交设置结果；成功只解除对应失败，不新增成功中心提醒。没有增加计时器、网络查询或新文件路径。注册表现有 8 triggers / 31 jobs / 62 events（60 active / 2 planned）/ 18 policies / 68 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup；生产 planned_closed 两项继续关闭，原型表保持独立。

## 验证

在 apps/desktop/tauri 执行：

- cargo test --locked commands::preferences::tests：首次 6 通过 / 1 失败；初始化合法创建了 manager-state 文件，原测试误断言目录为空。修正为比较初始化后的文件名与内容快照，重跑 7/7。初次失败日志保留。
- cargo test --locked --test event_registry：31/31。
- cargo test --locked：33 组，769 通过、0 失败、4 忽略；包含共享取消后工作线程准入与写者冻结回归。
- cargo clippy --locked --all-targets -- -D warnings、cargo fmt --all -- --check、git diff --check：通过。

真实隔离文件写入失败 → 持久异常 → 加载历史 → 同配置成功解除；保存 / 恢复默认相互隔离；stable 失败与 beta 成功相互隔离；校验 / 忙 / 待重启拒绝不生成事件文件；通知持久化失败后设置真实落盘而异常保守保留。通用终态矩阵继续核对迟到 / 未核验 / 候选 / 根世代与删除历史边界。

本补丁未改 UI，未重跑 UI（上一 UI 基线 27e425bf 为 94 文件 / 454 项）。测试调用相同工作线程保存内核与通知持久化适配器，没有使用 AppHandle 原生发出点，不声称原生通知已验收。Windows、原生设置操作、全域事件与性能 / 安装器 / 发布门禁仍未通过。TASK / IMP 保持 in_progress，不形成整体验收证书，不迁移真实用户数据，不替换日常安装，不公开发布或晋升 stable。
