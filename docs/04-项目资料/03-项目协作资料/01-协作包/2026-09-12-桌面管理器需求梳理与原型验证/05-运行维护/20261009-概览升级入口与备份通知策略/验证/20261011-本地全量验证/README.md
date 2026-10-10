# 2026-10-11 本地全量验证

## 验证对象

- 代码分支：`feature/0.1.10-maintenance`
- 代码提交：`8f5a76f867272730ee22ba672934fdd82807d218`
- 文档分支：`docs/governance-main`
- 主题：共享生产树表组件的复选框、箭头占位、图标与文字垂直对齐，以及本地源码回归。

本次代码修复位于 `apps/desktop/ui/src/components/ui/UiTreeTable.vue`：去除原生复选框的基线外边距补偿，统一控件和文字的 flex 对齐与 22px 行高。它不改变树表选择、展开、备份范围和数据语义。

## 命令与结果

| 范围 | 命令 | 结果 |
| --- | --- | --- |
| Rust | `cargo metadata --locked --no-deps --format-version 1` | 通过 |
| Rust | `cargo fmt --all -- --check` | 通过 |
| Rust | `cargo check --locked --workspace --all-targets` | 通过 |
| Rust | `cargo clippy --locked --workspace --all-targets -- -D warnings` | 通过 |
| Rust | `cargo test --locked --workspace --features integration-test` | 690 通过 / 0 失败 / 4 忽略 |
| UI | `npm test -- --run` | 94 个测试文件 / 459 个测试通过 |
| UI | `npm run typecheck` | 通过 |
| UI | `npm run build` | 通过，170 modules transformed |

Vite 输出既有 `INEFFECTIVE_DYNAMIC_IMPORT` 警告，测试输出既有 jsdom HTMLCanvasElement.getContext() notice；均未导致命令失败。

## 证据与边界

结构化证据位于 `.adg/evidence/OCX-0110-20261011-LOCAL-VALIDATION/`，包含命令、结果、UI 构建摘要和范围边界。

本地源码通过不等于版本完成。以下门禁仍需真实设备、制品或长期运行证据：macOS 原生 UI / 安装回归；Windows 普通权限、安装位置与自定义路径、DPI、云母、原生标题栏、动效、透明下拉框；真实更新与重启；自动检查性能及缓存；stable 24h / beta 6h 周期；真实用户迁移；签名、发布资产、更新端点；全域事件完整性审阅。当前 Windows 候选包仍是 0.1.9，不能替代 0.1.10 打包与发布门禁。

本记录只作为协作包验证回执，不改变稳定核心事实，不关闭 IMP / TASK，不授权合并 `main`、晋升 stable 或公开发布。
