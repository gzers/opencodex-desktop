# 端点变更核验与终态事件（2026-10-11）

实现：feature/0.1.10-maintenance@734583941b083a559b247bb58a1844b9f43536a6，已推送；未并入 main，版本仍 0.1.9。

保存先校验输入与最终配置，再存凭据、原子写配置，精确回读配置和口令均一致才成功。删除只接受明确 NoEntry 为不存在，删除后须核验确实不存在；锁定、平台错误、不可读或删除未生效均失败并保留配置引用供重试。保留凭据的删除不访问凭据后端。

两个用户触发终态对按完整请求摘要、动作和根世代隔离；仅精确成功解除对应失败，不新增成功提醒。通知只持久化候选 SHA256，不持久化口令。输入 / 准入拒绝不虚构执行终态。

凭据与配置不是跨存储原子事务，不做失败回滚：保存可能只提交凭据或遗留新引用，删除可能部分成功而配置保留。重试可核验并完成余下步骤。不得把失败解读为完全未变更。

最终 Rust 33 组 / 798 通过 / 0 失败 / 4 忽略；端点定向 9/9、凭据缺失分类 1/1、注册表 31/31，fmt、Clippy all-targets -D warnings 与 diff 检查通过。六份原始命令日志见本目录；命令、工作目录、退出码与源码身份见 .adg/evidence/OCX-0110-20261011-ENDPOINT-MUTATION/verification.json。

当前生产：8 triggers / 35 jobs / 72 events（70 active / 2 planned）/ 23 policies / 78 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。没有新增调度计时器或存储路径。原型表独立。

本轮未改 UI，未重跑 UI / 原生。注入凭据测试不代替平台 Keychain / Windows Credential Manager 或真实 AppHandle 投递。手动同步终态、CLI/IPC 所有权、全领域事件、损坏保护受控修复、原生更新 / 重启、Windows / 安装器、同机配对性能与真实周期继续待验。TASK / IMP 保持 in_progress；无稳定核心事实回写、真实用户迁移、日常安装替换或发布。
