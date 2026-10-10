# 更新偏好保护源码验证

代码分支 feature/0.1.10-maintenance，提交 7ce357b2。macOS 27.0.1 arm64；本记录为源码自动化，不是原生更新 / 安装器验收。版本仍为 0.1.9，未改日常安装。

- `cargo test --manifest-path apps/desktop/tauri/Cargo.toml --all-targets --no-fail-fast`：full-v2.log 为 32 组 / 734 通过、0 失败、4 忽略。忽略不计通过；在最后同候选防覆盖补丁之前运行。
- `cargo test --manifest-path apps/desktop/tauri/Cargo.toml modules::update::handoff --lib`：handoff-v3.log 为最后补丁的 7/7。覆盖不同候选、错误关联、独立用户固定、回执写入后轮换重试及同候选新旧尝试冲突。
- `npm test -- --run`（apps/desktop/ui）：ui-v2.log 为 93 文件 / 432 项通过。
- `npm run build`（apps/desktop/ui）：ui-build-v2.log 为 vue-tsc 与 Vite 生产构建成功；已有动态导入分包警告保留。
- `cargo clippy --manifest-path apps/desktop/tauri/Cargo.toml --all-targets -- -D warnings`：clippy-v2.log 通过，在最后防覆盖补丁之前。fmt 通过。

full.log 的旧失败为 create_restore_backup_stays_pinned 仍断言强制用户固定，修正为独立 active 保护后重跑；ui.log 旧两项失败为初始错误值 null / 空字符串断言，修正后重跑。保留失败原件，不用重跑覆盖。

临时保护在失败 / 中断时保留，不依赖 guard 析构解除。面板真实安装成功后解除；管理器精确重启核验后先持久验证回执，再解除关联备份。解除失败保留保护；面板提供安装成功但保护待核对提示，尚无持久自动重试关联。系统安装器、真实更新、重启与故障后人工核对仍待验，不据此关闭 TASK-182 的 required_checks。

日志原始输出来自对应 /tmp 文件，保留其中既有警告；SHA256SUMS 用于校验本次留存。
