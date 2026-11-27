# TEAM84 Windows 准备状态与 Doctor 报告定向复核

候选 e663369226337c9a9cad594ccd3cafd1719c3711，Actions 45973489918。待证伪判断为 Windows 构建的准备状态会接受过期尝试，或 Doctor 报告在正常交给解析器后无法生成只读摘要。复算现有 MSVC 测试二进制 SHA-256，与[原生状态探针](native-provisioned-status-probe-r1.md)一致。指定运行 1 项准备状态测试和 4 项 Doctor 解析、限制与序列化测试，两个测试命令退出码均为 0。[原始输出](raw/preparation-doctor-component-tests-r1.txt)

准备状态测试验证双重尝试、过期重试和旧工作者的拒绝。完整的启动准备状态集按 macOS arm64 编译，未进入该 Windows 二进制。本地五项测试不执行官方 Doctor、网络下载或桌面操作。截图中的 Doctor 失败仍成立，原始退出码、超时与报告是否生成仍未知，沿用[共享交接](doctor-source-readback-r1.md)。

复核远端 main 与 Actions，尚无更新同源候选。现有令牌无法枚举 Hyper-V，当前路径也未找到 Windows Sandbox 入口，独立离线桌面可用性未建立。未触碰 RDP、网络策略或历史现场。本次测试没有创建新的测试 HOME。首次桌面 Status 及可信空闲仍待候选复验，Start 保持拒绝。accepted:false.
