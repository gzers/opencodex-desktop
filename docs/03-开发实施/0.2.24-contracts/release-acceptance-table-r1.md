# 0.2.24 发布前验收表

日期：2026-11-22。代码：main 06ddcc7d1（另有 bfa0d4d3b 的 CHANGELOG 与前端测试修正，产品行为不变）。文档：docs/024-extension-boundary、docs/024-windows-boundary。

整版结论：**accepted:false**。Mac 侧除界面实屏观察外均已取得证据；Windows 侧尚未按新边界重建，真机启停与清理仍缺。

## Mac 侧

| 项 | 判定 | 证据 |
|---|---|---|
| 所有权收缩（渠道/模型归 OpenCodex） | 通过 | main 958029879；合同 desktop-owner-scope-correction-r1 |
| 真实 `~/.codex` 只读 | 通过 | 生产代码无任何 `home/.codex` 写点，仅测试夹具；config.toml mtime 仍为 2026-11-21 20:43，SHA256 4b7102ff43… |
| 扩展写入落在自管根 | 通过 | main 8b450a3fe；真机交叉核对（ocx doctor 报自管根、codex mcp list 列出写入项）；回归 desktop_mcp_write_lands_in_the_codex_home_the_service_reads |
| 单客户端写入不留痕迹 | 通过 | main e84d5e28f；回归 writing_one_client_leaves_no_trace_in_the_others |
| 共享 %TEMP% 下不绑定自管根 | 通过 | main 8b450a3fe；lib 用例 data_root_directly_under_the_shared_temp_parent_never_binds_a_managed_codex_root |
| Rust 静态检查 | 通过 | cargo fmt --check、cargo clippy --locked --all-targets -D warnings |
| Rust lib 测试 | 2177 通过 / 5 失败 | 5 项与基线 6784f89fe 相同（macOS TCP listener fixture 3、data_root `/private`、受系统保护的临时路径） |
| Rust 集成测试 | 通过 | 38 个 test 二进制无失败（--no-fail-fast） |
| Windows 目标类型检查 | 通过 | cargo xwin check --target x86_64-pc-windows-msvc --all-targets：0 错误（仅既有警告） |
| Windows 测试目标链接 | 通过（编译期） | cargo xwin build --all-targets：0 错误，生成 54 个 PE32+ x86-64 可执行文件 |
| 前端类型检查与单元测试 | 通过 | vue-tsc 干净；vitest 142 文件 / 892 用例全过 |
| 前端发布门禁 | 通过 | qa/gate-release.mjs：3 通过 / 0 失败 / 0 跳过 |
| 官方 2.99 产品路径启停 | 通过 | 隔离根 Start → attested Status running/ready → Stop；实读进程 CODEX_HOME 为自管根 |
| 候选包首次供给（无头） | 通过 | 程序根建出、current.json=2.99.0、node v26.8.1、ocx 2.99.0、doctor 报自管根、配置字节未变、无残留 |
| DMG 打包与只读挂载核验 | 通过 | SHA256 b6d4f6a97f…，273,933,528 字节 |
| 发布输入预检 | 通过 | verify_release_inputs_024.py --tag v0.2.24 |
| 原型与软件一致 | 通过（按路径） | 原型 Codex 落点改为 `<OPENCODEX_HOME>/codex-client/...`；原型自测改动前后同为 383/201，未新增失败 |
| **界面实屏观察（首次供给后）** | **未完成** | 该机持续锁屏，系统拒绝 AX 观察与截屏 |

## Windows 侧

| 项 | 判定 | 说明 |
|---|---|---|
| 按新边界重建分支 | 未完成 | codex/024-windows-native 仍为 6532cbb6e（撤回范围） |
| Windows 代码可编译可链接 | 已证明 | 类型检查与全部测试目标链接均通过；仍不等于真机运行 |
| 单用户真实启停耗时与终态 | 未完成 | 需重建后执行 |
| MCP 写入自管根、真实 profile 未变 | 未完成 | 需重建后执行 |
| 双账户实例自管根互不覆盖 | 未完成 | 需重建后执行 |
| 卸载/账户清理 | 未完成 | 需重建后执行 |
| 安装器与正式候选验收 | 未完成 | 冻结候选后执行 |

交接与四项回交要求已写入 Windows 实际消费的 `docs/024-windows-boundary`（`windows-dev-handoff.md` 末段，附两份合同）。

## 未纳入本版

- 旧版配置自动迁移、兼容路径：按决定不开发。
- 其余五个客户端重写用户级配置时保留注释与键顺序：既有行为，本版未改，写入前有备份。
- `installed` 恒为 true：需先定产品口径，本版未改，已在证据中单列。
