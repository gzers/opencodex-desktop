# CLI 备份所有权与已保存策略验证

- 实现分支：feature/0.1.10-maintenance。源码：bc43e46023cde9781460962b054eb3d19ba027b5。
- 本轮只覆盖 Rust CLI 备份 / 列表，未修改 UI 或重跑原生。TASK / IMP 保持 in_progress。

CLI 手动备份复用 GUI 的源捕获、格式校验、手动分类、完整性摘要、已保存清理策略与真实终态。生产服务绑定运行中 GUI 的 AppHandle；无原生通知传输的隔离测试不作为原生事件验收。后台线程持有存储准入至结果及脱敏审计完成；观察者取消不取消执行。成功备份后的清理失败只作诊断，不能改写备份成功。

备份列表保留 snake-case DTO，包含手动及旧升级等有界记录，不做哈希校验或清理。合作锁等待 / 扫描移到后台线程；取消列表观察者后仍持有准入至扫描完成。查询取消不写变更终态审计。

兼容性：创建结果仍为 status / message；缺失、损坏偏好源与损坏策略统一返回 ExecutionFailed（旧 raw read 缺失源返回 TargetNotFound）。不增加不安全的存在性预检查。未确认返回 RequireConfirm；冻结创建 / 列表返回 TargetStateConflict，且零执行 / 审计 / 备份锁。

## 实际结果

| 检查 | 结果 | 原件 |
| --- | --- | --- |
| 全量 Rust | 33 组 / 815 通过 / 0 失败 / 4 忽略；exit 0 | rust-full.log |
| IPC 定向 | 13 通过 / 0 失败 / 0 忽略；exit 0 | ipc-focused.log |
| fmt | exit 0 | fmt.log |
| Clippy all-targets -D warnings | exit 0 | clippy.log |
| 代码 diff | exit 0 | diff.log |

6 项新增测试覆盖分类 / 摘要 / DTO / 完整列表、manual / automatic 策略及固定项 / 旧事务保护、坏源 / 坏策略无清理、成功 / 失败 / panic 取消所有权、冻结 / 未确认拒绝，以及真实文件锁等待时单线程异步执行器仍响应。最后一项为同进程锁测试，不能替代跨进程 W2 或原生性能预算。

首次 async 列表错误映射返回 ExecutionFailed，冻结断言失败：failed-focused.log，exit 101（11 通过 / 1 失败）。修正 storage binding 映射为 TargetStateConflict 后通过，未降低断言。两份较早成功轮次保留为 superseded，不作最终源码证据。原始日志 EOF 空行原样保留，暂存 diff 的日志空行提示不通过删改原件消除；文档 / JSON 另行检查。

注册表保持 8 triggers / 36 jobs / 74 events（72 active / 2 planned）/ 24 policies / 80 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup；没有新计时器或路径。UI 历史证据为 82c3479b 的 94 文件 / 459 项，不能作为本轮重跑。

其他 CLI 写入与全域事件、跨进程 W2、损坏保护受控修复、Windows 双权限 / 安装器 / 路径、macOS 隔离安装、真实更新 / 重启、同机 0.1.9 配对性能与真实检查周期继续待验。未关闭 TASK、颁发证书、回写稳定核心或发布。
