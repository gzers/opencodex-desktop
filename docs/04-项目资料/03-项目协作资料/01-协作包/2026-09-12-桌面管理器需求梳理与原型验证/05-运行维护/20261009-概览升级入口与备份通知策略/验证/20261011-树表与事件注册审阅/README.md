# 20261011 树表与事件注册审阅

本轮对应 0.1.10 目标执行，审阅对象是代码分支 feature/0.1.10-maintenance 的提交 c654287c06da3e621091c1051ebc19ce1c0b1077。文档与证据归属 docs/governance-main。本轮只做静态源码盘点、统一组件合同核对、事件边界确认和定向回归，不构成合并、完成证书、发布或 stable 授权。

## 树表盘点

生产层级树表只有共享实现 apps/desktop/ui/src/components/ui/UiTreeTable.vue 与 apps/desktop/ui/src/components/ui/treeTable.ts，生产入口为 apps/desktop/ui/src/features/backup/BackupFiles.vue。SettingsRoute.vue:1158 和 SettingsRoute.vue:1195 的两个 data-root-table 是数据根路径的平面清单，不是层级树，不计为遗漏的树表组件。同步页的 WebDAV 与配置文件导入两个 tab 仍保留。

共享合同已经固定：复选框 16px、箭头或占位 22px、图标 18px、文字行高 22px，名称行 align-items: center；支持展开 / 折叠、键盘导航、刷新后焦点保留、加载 / 空态 / 错误 / 重试；备份树初始展开深度为 2，默认展开到月份层级。后续所有生产层级树表继续复用该组件，0.2.0 原型只继承组件合同，不提前改变业务范围。

## 事件边界

当前注册表统计为 8 triggers / 39 jobs / 81 events / 79 active / 2 planned / 27 notification policies / 87 emission sites / 8 scheduling sites / 8 UI feedback sites / 12 paths / 4 cleanup policies。完整事件矩阵见对应 .adg/evidence/OCX-0110-20261011-TREE-EVENT-AUDIT/event-matrix.tsv。

生产直接 app.emit / app.emit_to 的边界集中在 apps/desktop/tauri/src/commands/event_delivery.rs；其余生产发出点通过 event_delivery::emit_signal、emit_signal_to、prepare、publish 或 publish_checked。runtime.rs::EventProgressSink::emit 与 lib.rs::TauriStatusEmitter::emit 的瞬态状态也已纳入校验和统一投递边界。

以下 4 项已登记但明确属于进程内部信号：

- process-cancel-signal
- process-observation-signal
- process-exit-signal
- runtime-stdout-signal

它们的注册定义均为 process_signals / runtime / observe / observation / local / signal / active，发出点标记为 internal_only。它们不进入通知持久化、不提供前端订阅、不属于用户可见通知，因此不计为统一通知投递缺口。

## 验证结果

定向命令：

    cargo test --locked --manifest-path apps/desktop/tauri/Cargo.toml --features integration-test --test event_registry -- --nocapture

结果：33 passed; 0 failed; 0 ignored。

此前绑定同一源码基线的本地全量源码证据保持有效：Rust 694 passed / 0 failed / 4 ignored，UI 94 个测试文件、459 项通过，类型检查和生产构建通过，fmt 与 Clippy -D warnings 通过。警告仅为已有 Vite INEFFECTIVE_DYNAMIC_IMPORT 与 jsdom canvas notice，未阻断结果。

## 未闭合门禁

macOS 原生 UI / 安装回归，Windows 普通权限、DPI、云母、原生标题栏、动效、透明下拉框与安装路径，自动检查性能，真实 stable 24 小时 / beta 6 小时周期，真实更新 / 重启，签名制品、更新端点、真实用户迁移和发布资产仍需真实证据。当前 releaseAccepted: false；不替换日常 0.1.7、不迁移真实用户数据、不公开发布、不晋升 stable。
