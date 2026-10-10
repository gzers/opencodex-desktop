# 树表刷新与重试焦点局部验证

源码 feature/0.1.10-maintenance@27e425bf2aaa9c889cd5212c4fab2d3d8b3166d4，已推送；未合入 main，开发版本仍为 0.1.9。

## 问题与修复

d3e4df32 的 macOS 隔离原生键盘刷新丢失焦点，落到 HTML content，原 AX / 截图保存在 before-fix。共享 UiTreeTable 的刷新在加载中保留可聚焦性，以 aria-disabled / aria-busy 表达状态，调用端加载守卫防重复请求；重试失败恢复到重试，成功恢复到树行。树行移除只在该行实际持有焦点时转移，不抢走页面其他控件焦点。

## 自动验证

使用跟踪的 package-lock 执行 npm ci。UI 全量 94 文件 / 454 项通过；npm run build 的 vue-tsc 与生产构建通过；git diff --check 通过；cargo build --features custom-protocol 通过。原件在 logs。本补丁仅改 UI / UI 测试，未重跑本提交 Rust 全量，不能沿用旧结果声称当前全量通过。jsdom canvas 警告不作为图形或原生验收证据。

## 隔离原生操作与结果

macOS 27.0.1（26A434）arm64；临时 custom-protocol debug bundle，经 ad-hoc 签名；binary SHA-256、显式隔离环境见 manifest.json。只复制先前隔离夹具的偏好和两份备份，关闭自动检查，未复制损坏保护回执，未替换日常安装。

1. 数据与备份：默认根 / 年份展开，月份可见但下级折叠，见 native/default-ax.txt 与截图。
2. 键盘 Right 展开月份、子目录；Left 从子节点返回父节点并折叠，焦点保留，见 keyboard-month-expanded-ax.txt / keyboard-collapse-parent-ax.txt。
3. 用 Tab / Shift+Tab 聚焦刷新，再 Return 刷新。after-keyboard-refresh-ax.txt 仍以 button 刷新为焦点，月份展开保留。
4. 仅在隔离 backups 下新建 __tree_focus_reject_link 链接，触发有界浏览拒绝；刷新焦点保留，Tab 到重试后 Return 再失败，retry-failure-ax.txt 仍聚焦 button 重试。
5. 终端只移除新注入链接，Return 重试成功；retry-success-ax.txt 聚焦根行 button 收起 backups，月份展开仍在。
6. 复核两份备份的四个文件摘要全部不变，见 backups-before.json / backups-after.json。未操作恢复、清理、打开文件、安装或真实用户数据迁移。

## 验收边界

通过的是 macOS 隔离原生树表键盘刷新 / 失败重试局部检查。删除行不抢外部焦点只由自动化覆盖；全主题、三档、缩放、复选框像素对齐、Windows 与安装器没有本轮证据。保护损坏受控修复、实际更新 / 重启、全域事件、配对性能及真实周期仍待验。TASK-182 / IMP-20 保持 in_progress，required_checks 不因本局部结果全部通过，无完成证书或发布授权。

机器摘要与证据文件 SHA-256 位于 .adg/evidence/OCX-0110-20261010-TREE-FOCUS。
