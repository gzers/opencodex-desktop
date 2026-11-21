# Mac ARM64 Release 编译与实屏应用包绑定

固定源码 57073c4e5df13294e9c067ee86bf4535ded7f9b8，使用与 IMP223 一致的本地运行包和构建配置执行锁定依赖的 ARM64 Release 构建，退出码 0。完整 Cargo JSON 回执有一个成功的构建终态，以及三个唯一的可执行产物。

已封装并通过 IMP223 实屏验证的应用内，opencodex-desktop、ocxd 和 ocx 均与 Cargo 产物匹配。签名归一化只在临时副本上执行，比对涵盖完整文件字节。原件摘要复核、原包严格深度验签、主进程及 IMP224 DMG 摘要复核通过。临时副本已清理，本轮未启动或安装应用。

证据覆盖本机编译到实屏应用的绑定。正式发布流水线和仓库更新公钥对应的 Tauri 签名仍待验，整版 accepted:false。

[绑定结果](build-binding.json) · [原始日志摘要](cargo-receipt-index.json) · [运行前假设](hypothesis.md)
