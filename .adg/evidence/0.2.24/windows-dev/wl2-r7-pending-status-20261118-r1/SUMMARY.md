# r7 V2 保全回执的 Status 定向验证

本轮使用 main 6bafdd342e69ee83e842f4b35a20bafa1320645a、治理 6ef21eb5f144905318d94f5080b2acf408e258bf、TEAM50／IMP189／r7。代码为 96fb7829430f4c3fca0caeef384bd1a3e0bc14a6，三处改动均为测试。开发产品制品沿用上一源码 0d0c9d29ccd9075d59f92350c158c256c835924b，生产实现相同。

| 项目 | 结果 |
|---|---|
| MSVC | 首轮与最终测试编译 raw0。最终 185406 ms。受影响定向 2/2。 |
| Status 投影 | 使用实际原生附件写入结果及存活夹具线程。成功 durable=true。同名冲突 raw183，短写、flush raw1117、回读注入为 durable=false。首个 Start raw64、双记录摘要、附件原始错误和恢复 Required 保持。Running/LostWorkerUnknown 隐去附件。读取已过期拒绝。 |
| 首错 | 首轮定向夹具登记的线程槽为空，实际 Status 正确返回 LostWorkerUnknown，原测试期待 FailedRetained。保留原标准输出和错误。首轮包装器在解析计数前退出，未单独留存原测试退出码。 |
| 清理 | 最终五个夹具根缺席。首轮失败的两份原始记录核摘要归档后，清理已核定的自有文件及空目录。生成 schema 归档后缺席。 |
| 原包扫描 | 原 Source 存在且此前各步成功时，两次 V2 引用读取前后到达三次完整扫描，共用原截止，具体调用链见 source-call-chain.md。保留全部校验。 |
| 实际 Owner／普通桌面 | 保全附件私有 D pin 入口依赖旧 r2 I/D/H、原合成 Profile 和协调对象。真实回执与原 10 秒未验。独立的新任务必须从合同认可的真实普通桌面基线入场。 |
| 当前 r2 | 沿用全归档和十份回执，首错 command-lineage raw64，原 Job 24 项未知、raw50，executionReady=false。旧进程、三原件、锁、seq9 保留。 |

累计 267 项状态见 status.json。制品摘要、夹具原记录、包装器及原编译日志随附。公开 Start 保持拒绝。
