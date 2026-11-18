# r7 固定读回入口与保留写句柄的共享冲突

合同 TEAM66／IMP205，治理 bebdfa503f374d6115b9fa313b9c62b213009278。固定 Windows 源码 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7。待证伪判断为原 worker 持有 V2 附件写句柄时，固定 `ExportExact` 的只读打开会返回 raw32，独立读回因而无法完成。

- 固定启动器 `ExportExact` 以 Read access 和 Read sharing 打开记录。源码 `windows_official_runtime024/mod.rs` 的 `create_private_journal_file` 以 0xc0020000 access、1 share、1 disposition、0x200000 flags 创建记录。`before_resume_record_v2/storage.rs` 写入、`sync_all`、原句柄读回并把句柄存入 `self.files`。`recovery_worker.rs` 保留带附件存储的 WorkerLocal 后进入 `park`。`durable` 的成功追溯到上述路径与原 10 秒截止复核。
- `probe.ps1` 从固定启动器摘取原字节摘要验证后的函数，使用同一组原生文件打开参数和 34 字节合成记录，无产品进程、账户、ACL、旧现场或真实附件操作。首次控制端 PS5.1 子进程继承的模块路径使 Get-FileHash 缺失，入口 raw1，在创建夹具前退出。仅对子进程移除继承的 PSModulePath 后，PS5.1 返回 raw0，stderr 为空。
- 保留写句柄时，原 `ExportExact` 返回 raw32／HRESULT -2147024864，输出缺席。仅将合成提取版的只读打开 share 改为 ReadWrite 后逐字节匹配。同一期间第二个写者仍被原句柄拒绝，raw32。释放合成句柄后，原 `ExportExact` 通过且两个副本 SHA-256 一致。合成目录清理并回读确认缺席。

请 Mac 明确只读入口的最小固定配对变更。候选是在 `CheckAttachment` 的独立读取中仍请求 Read access，同时使用 ReadWrite share 以兼容已存在的保留句柄。保留现有绑定、只创建导出、完整语义核验、原 5 秒 QPC 和一次性门槛，并重新固定启动器和计时器中的绑定摘要。合成结论不计真实附件读回通过。当前 `range-r1/out01` 无回执，原暂存未变。Host 和原 worker 只读范围可独立采集，Explicit 及 DiagnosticAfter 依赖可用的后续计时读回入口。独立旧 r2 现场全部保全。
