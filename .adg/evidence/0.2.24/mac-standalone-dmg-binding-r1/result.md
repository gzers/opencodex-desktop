# Mac 0.2.24 单应用 DMG 验证

使用 IMP223 实屏验证过的 ARM64 Release 应用包，源码 `57073c4e5df13294e9c067ee86bf4535ded7f9b8`。包内主进程 SHA256 为 `666060f66d015fd5b82449ec5947bf31d4188942a79953ee04430e23a254660e`。

- DMG 为 278,417,530 字节，SHA256 为 `b086a75841f8ed835d81938f791877f37943803ef6c3053c781a87f6b2c86762`。DMG 留存本机。
- 只读挂载含单个应用包、指向 `/Applications` 的链接和安装说明。挂载前后严格深度 ad hoc 验签通过。挂载内应用树与原包摘要一致，内置官方 OpenCodex 2.99.0 资源离线校验通过。
- 原包未变化。挂载和打包临时目录已清理。本轮未启动应用或安装。`hdiutil` 的命令弃用提示已保存在原始回执，相关命令退出码均为 0。

现有构建日志缺少完整 Cargo JSON 回执，三项可执行文件的发行构建绑定、基于仓库更新公钥的 Tauri 制品签名绑定仍未验证。项目现行公开测试分发采用 ad hoc 签名。全版 `accepted:false`。

[打包原始回执](package-evidence.json) · [版本预检](version-preflight.json) · [运行前假设](hypothesis.md)
