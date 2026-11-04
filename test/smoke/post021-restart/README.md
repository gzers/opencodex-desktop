# Windows 保存后重启的自有客户端夹具

`OwnedClient.cs` 是实际 Windows GUI 进程，启动时读取隔离根的 `home/.codex/config.toml`，显示内容并写出带 PID 的读回文件。`owned-desktop.ts` 把它接入固定官方包的原始 `restartCodexDesktopApp` 实现，只枚举、退出和重新拉起这一受控 EXE；不调用全局 Codex 进程发现，不允许强杀或全局命令回退。

验证宿主必须设置独立 HOME/USERPROFILE/APPDATA/LOCALAPPDATA/CODEX_HOME/OPENCODEX_HOME，并使用包含 `post021-restart-panel` 的专属目录。将 C# 编译为该目录下的 `OwnedClient.exe`，向 `ownedDesktop(root, officialPackageRoot)` 传入本地固定 `@bitkyc08/opencodex@2.84.0` 包根，再将返回的 `restart` 作为官方 `performCodexRestart` 的测试依赖。包本身不修改；真实 app-server 管理和原生 Codex 当前选择读写仍由相应官方拥有者执行。

核对 `owned-restart-result.json`：旧 PID 被退出、无残留、新 PID 不同、重新启动结果为 started，且新进程 `client-readback-PID.txt` 与隔离配置字节一致。夹具的成功只证明自有 Windows 客户端重启链，不代替正式打包 Codex 桌面应用、安装升级或日常数据验证。
