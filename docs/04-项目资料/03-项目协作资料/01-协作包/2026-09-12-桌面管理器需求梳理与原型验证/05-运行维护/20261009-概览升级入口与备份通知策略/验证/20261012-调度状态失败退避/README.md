# 调度状态失败退避修复（局部验证）

代码：main@01a1d85504781f4cb189a68ed85d1d803a1409ff，父提交 fb34f5b91c3a6feb5b41f11f9527171c4bf742f5；已推送 origin/main。
对应 IMP-20 / TASK-182；没有更新整体验收状态。

原自动调度在读取状态失败后安排 30 分钟等待，但 finally 中的排队唤醒或后续前台 / 网络恢复会用 100 毫秒重新安排计时器。错误分支现在用单调时间保存最低等待截止，统一 arm 遵守该截止，最多一个计时器。隐藏时取消计时器，恢复仍保留截止；到期才重新评估。手动查询、45 秒启动宽限、原生对象级持久网络退避、文件路径与 schema 均不变。

## 验证与来源

运行目录为代码 worktree 的 apps/desktop/ui，Node v26.8.1 / npm 11.19.0 / Vitest 5.0.0。

| 日志 | 命令与结果 | 证明范围 |
| --- | --- | --- |
| red-test.log | npm test -- --run tests/update-scheduler.test.ts；exit 1，9 通过 / 2 失败 | 修复前计划读取被提前；失败轮原样保留 |
| green-test.log | 同一命令；exit 0，11 / 11 | 正负 7 天时钟变化、100 次在途唤醒、隐藏恢复、100 次后续唤醒保留 30min 下限；到期查询及 stop 清理 |
| related-tests.log | 日志首行完整 argv；exit 0，8 文件 / 38 项 | 更新查询 / 安装归属、共享详情、进度、设置重试等；包含调度 11 项，不另加计数 |
| build.log | npm run build；exit 0 | vue-tsc --noEmit 与 Vite 构建；保留 4 条 INEFFECTIVE_DYNAMIC_IMPORT 警告 |

run-results.json 记录实际命令退出结果、提交与两份代码文件摘要；代码保留在 main，不复制源码。manifest.json 列出五份原始来源及 README 摘要，archive-verification.json 为复制后独立重读核验。原始日志外部路径记录在清单，仓库副本无损。

## 验收限制与下一步

这些是模拟依赖与虚拟时间的本地测试，不证明真实 Windows / macOS 原生更新、安装、实际 24h / 6h 周期或性能。没有前台资源采样，没有替换旧隔离副本、日常软件或真实数据。此次错误路径是否与此前 CPU / RSS 增量有关尚未确定，不能据此解除既有失败。

review:scheduler-performance-0110 保持 failed；八项 required checks 未整体闭合，TASK / IMP in_progress，acceptance_results 为空，0.1.10 未获发布授权 / 验收。下一步按原预算补受控配对性能与真实周期，再结合真机验证，而不是降低阈值。
