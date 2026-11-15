# Windows Ready/PreStop原卷工作快照增量（生产仍未收口）

沿TEAM14/IMP140/windows-security-coverage-r4，fetch主线仍为main2ebdd3ae0/docs84ec482ee。代码be3de6dfd/57904c6bce37f4ba37338285409fa6f9551b40da已推，最终tree42a6e55ee39a9a95bec74407b73ad3bf7b800b79。只改允许的Windows originals/tree/v2/adapter及其测试，未写共享解析器/trait、Mac源码、官方包、安装器/Vue/日常配置；预存untracked schema未手动编辑/暂存。

新原生工作快照在Profile原卷custody的ready/pre-stop中create-only封存真实工作数据，文件按1MiB分块，保留既有单文件2MiB/树32MiB/节点512限制；私有新对象经sync、原对象重新封口读回、fileID/8投影/目录枚举/无ADS核对，持有原生记录和父pin。数据目录含原工作源ID/安全/内容摘要及实际字节，D/phase/current只存严格原生引用；没有第二个可变journal，也不把复制文件ID说成原件ID。原件恢复仍用原根同对象custody路径和原句柄。

新增必需非默认workingRecord（None或Snapshot）；Ready/PreparedRestore必须各自对应Ready/PreStop引用，并核session/binding/policy/custody卷及归档对象ID唯一性。旧缺字段/未知/跨卷/串代/错误根/错误phase引用拒绝，不迁移旧Full/partial/前期component-v2。纯解析始终PendingOnly，不能从JSON生成WorkingArchive、ReadySession或ClosedEligibility。原生归档不是官方writer-exclusion或内容变更许可。

实际Owner Ready/PreStop调用点已消费新归档方法。未知工作变化可读时先保全真实字节，再由原不变工作代次断言拒绝，canonical Captured/current不推进，原件/部分归档保留；没有把观察到的新内容自动改成准许基线。读取忙/未知仍拒绝。完整官方暂态/生成物语义与活跃两锁/NativeLease协调尚未实现，当前只是必要的原生历史数据能力，不是完整官方副作用支持。安全范围继续固定8投影，auditSACL及未来扩展excluded-unknown/fullSecurityProven=false。

| 执行 | 原退出码 | 结果 | 耗时ms |
|---|---|---|---|
| library01 | 0 | tree/v2 27过 | 208515 |
| library02 | 0 | 加卷/ID/ADS及真实current部分写入后28过 | 129620 |
| library03 | 0 | 加首分块封存中断后29过 | 117522 |
| product01 | 0 | clean最终MSVC custom-protocol debug全部bins，lib68warnings | 202967 |

首错无非预期失败。原故障/反例结果均保留：新真实2MiB文件两分块封存；未知源编辑保全但不签Ready；归档额外文件poison；Ready数据/phase10后实际current一字节write+sync故障保留损坏指针并拒绝后继；第一真实1MiB数据分块封存后注入故障，保留原件/partial/canonical seq9，拒绝补metadata/Ready/catch-up。注入点仅cfg(test)，无环境/IPC权限开关或生产默认成功。

三次实际执行EXE覆盖前已记录摘要；最终97份产品输入对工作字节一致、Gitblob行尾等价，library03的53份Rust执行输入同最终，见final-source.json。Desktop77319680/SHA256204a087ba13afe69693592f240e967e759a8440db3cd85715a68985fdb05b1fc；ocxd5398528/SHA256013e9827111d92eac3542e24e8ad3649ef27c8b059cdca49e68ac74bc2b79322；ocx306176/SHA2560d8b042156e97b4f8d0f043d4f159882b0b4fe383bda25c5ba01139325087f9b。debug不是release预算/冻结候选/发布。

本轮39个中断/未知保留场景共431文件31617713字节，均私有ZIP内容核验，原场景留；ZIP不克隆NTFS ID/安全。正例guard只清自身完成对象；没有新账户/Profile/Public/GUI/官方服务/force/日常PATH策略写。全部历史自动审批删除拒绝未重试/绕过，overallCleanupComplete=false。

共享最小请求见shared-minimum-request.json：完整独立声明路由解析declaration(bytes)及helpers只在cfgmacos私有macos_client_session.rs，LaunchPlan无声明路由，shared client_artifact_policy要求启动前独立allowlist且None必须拒绝。请求集成线把既有纯解析器/断言提至modules/runtime/client_route_declaration.rs（等价纯数据API亦可），Mac消费同一实现；Windows后续读原生pin的H/config.json并消费。保留静态/动态未知、选择/自定义/组合/禁用/别名及slash编码/冲突、strictJSON/边界，不接文件/网络或授native许可。不改四trait/HMAC/预算，不以Windows复制一份或从生成catalog反推填空。此项只阻完整内容判定，NativeLease/后继/GUI等Windows独立未完继续负责。

完整矩阵/限制归final-status.json，离线verify只验原始字节。B1/B2原证据复用，CAP/trust非空1314/5无新前提不重跑；普通MSI与历史清理未完。真实GUI准备的0c三个输入保持不可变，out01观察仍空，待marenben原桌面执行；它也不是官方Owner组合。原官方副作用完整判定、原生协调与后继历史释放、恢复Closed/真实GUI及S-D生产组合未完，public Start/后继拒绝保持，allAccepted/productionExecuted/actualGUI=false。原30/5/10/Restart30含Stop子10不变，无业务同步/模型请求/W2/双SID/发布。
