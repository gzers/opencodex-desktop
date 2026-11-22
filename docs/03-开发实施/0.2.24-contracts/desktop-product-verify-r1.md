# 0.2.24 真实官方 2.99 产品路径验证

日期：2026-11-22。代码：main df4438a35。前序：958029879（范围收缩）、53580d8a1（Windows-only 修复）。

## 本轮要证伪的判断

1. 范围收缩后，产品路径已经没有任何写 OpenCodex 原生配置或接管真实用户客户端目录的入口。
2. OpenCodex 2.99 经产品编排启动时，使用的客户端目录在 Desktop 自管 HOME 内。
3. Windows 目标里不存在对已删除模块的残留引用。

## 结果

判断 1 成立：全部 Rust 源码搜索不到 client_recovery、ClientRecoveryState、CompletedLifecycleStop、StoppedObservation、with_client_recovery_root、manages_client_originals、native_stopped、recovery_state，也搜不到任何已删除的渠道、同步或导入模块名。命令层不再注册任何模型写入命令。

判断 2 成立。新增一条显式 root 的验证：

- 用仓库固定摘要 e0464990 的 2.99 运行包供应到一次性目录，程序镜像复验通过，版本 2.99.0。
- 通过产品编排的 Start 返回 Started，attested status 返回 running 且 ready。
- 运行中读取进程实际环境，CODEX_HOME 恰为 Desktop 自管 HOME 下的 codex-client，且不在真实用户客户端目录之下。
- Stop 返回 Stopped。
- 无渠道配置的 2.99 不会创建客户端目录，因此断言的是生效值，不断言目录必然存在。
- 真实用户客户端目录的 inode 与权限不变；mtime 不纳入断言，因为本机 Codex 会话自身会写历史。

判断 3 先被推翻、后修正：Windows 生命周期实现仍在调用已删除的 recovery_required，而该文件只在 Windows 目标编译。同批还删掉了只为保全这条链而存在的 manages_client_originals 钩子、孤立的状态测试文件，以及 Windows 测试里对客户端原件模块的引用。本机缺少 Windows C 工具链，无法真正交叉编译，这项靠源码审计而非构建证明。

## 顺带发现

GUI 直接启动时，若二进制不在 .app 包内，会停在安装位置提示，CLI 因此始终看到 instance offline。这是预期行为，不是缺陷；真实 GUI 验证需要在正式 .app 内进行。

## 验证

- cargo fmt --check、cargo clippy --locked --workspace --all-targets -- -D warnings 通过。
- 全量 lib 单测 2174 通过 / 5 失败 / 28 忽略；5 项均可在基线提交复现（macOS 监听夹具 3 项、私有临时路径布局 1 项、安装夹具系统保护 1 项）。
- vue-tsc、vitest、vite build 与门禁快照通过（本轮未改前端）。

## 仍未完成

Windows 真机原生启停与取消后的清理尚未在本轮执行，须由 Windows 侧在固定候选上补验。本机没有 Windows 环境，也没有执行真实业务同步或模型请求，未修改日常配置，未发布。
