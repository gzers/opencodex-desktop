输入 main 6bafdd342e69ee83e842f4b35a20bafa1320645a、governance 7cc64ed4ffd0f8b67f642e676290fbcfae7eac84、TEAM48／IMP187／r7。Windows 源码 6dbffae54c528750bfb6d5ad9b9e08dabaf78495 已推送并独立回读。阶段计量只在指定合成测试开启。完整观察次序、保留句柄寿命、字节比较、typed Current、成员枚举、poison、拒绝和截止均保留。

12 次追加及最终独立复查共 25 次完整 Journal 读回，合计 97869 μs。逐记录 fresh 核验 51906 μs，保留记录核验和字节读取 7529 μs，合占 60.73%。后者包含保留 pin、私有安全、对象身份及长度检查。Current 单独字节读取 395 μs，纯 typed Current 核验 1512 μs，合占 1.95%。其余阶段及未归属耗时见 measurement.json。每组顺序和预期字节均核对。最后读回结束时原 10 秒尚余 9572923 μs。单个 MSVC debug 合成前缀只说明本样本成本，不签真实恢复预算，也不构成移除任何核验的等价证明。

MSVC 定向测试编译 raw0，5 项通过，MSVC 开发版三个产品编译 raw0。测试 SHA-256 21fe1b26d660ffe69ca890aa4d03cc2a665c79bde575f28c80107c9dcdb07bc3。14 个报告夹具根核实缺席，生成 Windows schema 经归档核摘要后清理。当前 r2 继续使用已经核验的十份只读回执，首错 command-lineage raw64，原 Job 24 项未知、raw50、executionReady=false。普通桌面私有 V2 及严格 V2 Pending 首次耐久边界待验证。公开 Start 保持拒绝，未运行原包、W2、双 SID 或发布。
