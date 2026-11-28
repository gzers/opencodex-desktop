# TEAM88 Windows 共享接口交接

共同特性分支为 feature/024-team86-shared，Windows 消费 d4d5c07a56b7610e0de0ecde907d2f09a2680114 的 Doctor 和下载修复。合并诊断已在 4a899f874d43022cf194e0b9adb4c66984819aa9 回交，无需重跑。Status 首拒绝为 Lookup(NoMatchingSlot)，Doctor 在子进程创建后等待超时。

请 Mac 继续独占以下共享写集，并在同一特性分支交付最小接口。

1. infrastructure/status_source.rs 的 Windows 2.99.0 Status 分派须把已绑定的 Desktop data root 和当前 mutation gate 传给原生生命周期。保留传入的同一 5 秒 deadline。建议过渡入口为 windows_official_lifecycle024::status_with_context(runtime, home, data_root: Option<&Path>, mutation_in_progress: impl Fn() -> bool, deadline)。Windows 将实现该入口。未绑定／mutation 冲突只影响空闲确认，原保留 Owner 路径保留原语义。请复核 GUI 和 CLI 的两个 Status 构造都有实际绑定上下文。
2. modules/runtime/port.rs 暴露端口设置锁和锁内准备接口。保存、已保存端口快路径和自动选端口均使用同一 TargetFileLock。原生 Start 在同一 guard 下复验和选端口，持锁到创建边界，锁等待受原截止及已有锁上限约束。若可行，先在锁内算出候选端口，原生复验成功后才提交原子写入。避免递归取锁和先释放再创建。GUI、CLI 共用同一原生入口，显式端口也须参与协调。

Windows 独立实现原生身份、程序来源、记录、进程和监听读回，接口缺席时只暂停依赖项。PID-only、锁竞争、未知监听、访问拒绝及超时保持拒绝。预算 30／5／10 秒，旧包已作废，新候选待同源双平台 CI。accepted:false。

## Windows 接续时核对的接口缺口

在 d4d5c07a56b7610e0de0ecde907d2f09a2680114，fetch_inner_at 仅传 runtime、HOME 和 deadline。请将当前绑定的 Desktop 根及可重复查询的 mutation gate 传入 Windows 原生 Status。未绑定或 gate 查询不确定时，空闲证明须拒绝。

prepare_start 目前在写入时自行取锁，复用端口的快路径不取锁。请提供绑定到同一设置目标的 guard，锁内可只读选端口、成功复验后再复用或写入，并沿用原 3 秒锁上限及同一 action deadline。显式端口也需经过同一 guard。

释放 guard 时需已有跨宿主可读的正向启动事实，或者继续持有至该事实可见。请覆盖 CreateProcess 返回挂起进程到官方记录可见之间的空窗，包含创建成功后取消或报错的路径。Windows 将以独立 GUI／CLI 宿主验证最多只到达一次创建边界。此接口到位前，仅登记定向证据和维持未获证明的启动拒绝，暂不报告并发验收通过。
