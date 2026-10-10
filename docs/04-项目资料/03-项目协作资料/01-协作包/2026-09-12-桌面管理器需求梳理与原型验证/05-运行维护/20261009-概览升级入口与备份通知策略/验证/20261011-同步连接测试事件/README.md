# 同步连接测试所有权与精确终态

源码：feature/0.1.10-maintenance@a3b84474d68b1dffc93f2999fa165abe44b87b63。版本仍 0.1.9，未合入 main。本轮只补生产已有连接测试的执行所有权与注册终态，不启用 0.2.0 文件 / WebDAV 比较或自动同步能力。

## 行为及边界

连接测试经 owned blocking worker 执行，storage writer 准入由工作线程持有至完成；IPC 观察者取消不释放任务 / 操作锁 / 存储准入。配置和钥匙串读取移出 IPC 主线程。进程内非排队 sync gate 覆盖连接测试、端点保存 / 删除和现有手动同步；忙状态立即拒绝、不请求、不登记网络失败。不是跨进程锁，也不宣称覆盖所有导入 / 配置写入路径。手动同步仍直接 await，取消后的任务所有权尚待补齐。

真实有界探测完成、当前端点和凭据回读一致后才投递 sync-connection-failed / succeeded；网络失败仍按既有 API 返回 Ok 的失败 DTO，终态依据网络结果而非 IPC Ok。配置变化 / 不可读无终态，状态结束为 cancelled / unconfigured。同根世代、动作和完整端点 / 凭据候选成功才解除原失败；端点、口令、远端目录、数据根变化或候选旋转不解除旧失败。持久通知与 scope 仅存摘要，不保存 URL / 口令。通知持久化失败不改变真实网络结果。

sync_probe 为 user-only execution job。连接成功仅解除旧失败、不另发成功中心通知；自动更新 query / progress 的通用恢复禁令保持不变。既有 planned sync job 保持 planned。

## 验证

- commands::sync::probe_tests 定向 7/7：无效配置零请求、回读不一致 / 失败不投递、有界超时、真实失败 / 重载 / 精确恢复、不同端点 / 凭据 / 根隔离、通知写失败不伪造网络失败、取消观察者后的工作线程所有权与冻结 / 忙拒绝。
- event_registry 31/31；Rust 全量 33 组 / 785 通过 / 0 失败 / 4 忽略。
- cargo clippy --locked --all-targets -- -D warnings、cargo fmt --all -- --check、git diff --check 通过。
- 生产计数：8 triggers / 33 jobs / 68 events（66 active / 2 planned）/ 21 policies / 74 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。

命令工作目录为代码 worktree 的 apps/desktop/tauri；定向与矩阵使用注入 transport / 临时目录，不调用真实 WebDAV 或钥匙串。全量已有测试范围见原日志，不据此推定真实平台验收。未改 UI，未重跑 UI / 原生。

## 未成功及无效轮次

保留全部原日志：initial 是脚本 SyntaxError 前未变更源码的 4 项旧测试，不作补丁验证；compile 为错误转换和枚举名编译失败；focused 为 6 通过 / 1 失败（夹具硬编码 scope 文件名）及未处理 inner Result 警告。修正后 accepted 7/7；全量、Clippy 与 fmt 均为最终源码。提交准备时另一次工具 JS SyntaxError 未执行任何 shell / 编辑，不算测试轮次。

## 未完成

手动同步任务所有权、端点保存 / 删除与其他领域终态 / 准入 / 恢复矩阵仍待补齐。没有 AppHandle 原生投递验收，Windows、安装器、真实更新 / 重启、配对性能和真实周期尚待验。TASK / IMP 保持 in_progress，不回写稳定核心，不颁发完成证书；未迁移真实用户数据、替换日常安装、公开发布或晋升 stable。
