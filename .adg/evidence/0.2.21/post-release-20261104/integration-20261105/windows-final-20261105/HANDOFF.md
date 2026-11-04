# Windows 最终补验回交

本次补验采用 `codex/post021-sync-fixes@bd7c0fe8ccecfa39e0706c885dc233f3854dab74`、`docs/governance-main@59b319cf5cac0462985dd969d929f0f8fa8a6168`，另开代码分支 `codex/post021-windows-final`，文档分支 `docs/post021-windows-final`。最终代码补验提交为 `faf8dd9268bcee1f121129de5c7399dcee0fcd5f`；文档提交待本文件与回执提交后记录。

## 已通过的最终源 Windows 验证

- 最终源产品源码树为 `d55e6064c113a267b5f0d81c36e1c62476377acc`，EXE 源身份日志为 `bd7c0fe8c...`；前端全量 197 个文件、1289 项测试通过。
- GNU custom-protocol 构建成功。Rust 定向重启/管理/面板测试 38 项加最终全库显式 Node 环境回归 18 项通过；全库首次 981 通过、17 失败、4 忽略的原始日志保留，失败原因是测试要求 `OPENCODEX_TEST_NODE` 而环境未提供，随后以隔离 Node 重跑通过。
- 独立原生探针单测 44 项通过。改名实际探针使用真实 WebView2/Tauri IPC，验证普通备份、改名恢复目录、精确 config 前后镜像、旧文件保留、待处理记录清除及稳定 ID/相邻对象保护；旧预期遗漏 L2 已采用的 `provider.name` 更新，已在探针合同中修正并加入缺失/旧名反例。
- 隔离真实 WebView2 中，首次范围生成、真实 WebDAV 地址边界、配置同步范围读回和实际选中项通过。连接使用 `https://127.0.0.1:60441` 自有 TLS 只读夹具；没有真实 WebDAV、没有推理请求或远端业务写入。
- 已配置隔离连接启动时真实执行只读版本观察；夹具不可达时 UI 显示“读取失败”，观察来源为 `version-check`，没有伪造保存时间、同步成功或快照。缺运行记录重启显示具体 `runtime-port.json` 前置、无需先开面板、保存字节不变并保留重试。

证据见 `native/`、`checks/` 和 `source-manifest.json`。原始截图和失败日志未覆盖。

## 明确未完成项

### 物理桌面材质性能与 MSVC

当前 Windows 会话是 RDP，Win32 会话指标为远程会话，不能作为物理桌面性能通过。仍采集了最终源真实 WebView2 材质性能：Windows build 26100、WebView2 156.0.4314.58、Intel UHD 770 ANGLE D3D11、DPI 192、缩放 200%、Mica/高档、前台；结果只作为 RDP 观测，不能替代物理桌面门禁。原有 RDP 基线失败记录继续保留。

本机没有 `cl.exe`、`link.exe` 或 Windows SDK，只有 GNU Rust/LLVM 工具链，因此没有 MSVC 正式构建。GNU 构建成功不冒充 MSVC 通过。

### 正式 Codex 默认重启

未调用正式默认重启。未修改的官方 Windows adapter 只读发现到当前用户 MSIX：`OpenAI.Codex_26.1027.3384.0_x64__2p2nqsd0c76g0`，AUMID 为 `OpenAI.Codex_2p2nqsd0c76g0!App`。其进程树和 `shell:AppsFolder` relaunch 没有隔离 `CODEX_HOME`/数据根参数；当前实例属于日常同包范围，无法证明可安全只操作一次性实例，因此按门禁停止。OwnedClient 旧证据不代替正式 Codex 默认 adapter。

`default-adapter-observation.json`、`packaged-codex-processes.json` 保留了发现结果。没有调用退出、taskkill、AppsFolder 启动，也没有操作日常 Codex。

## 隔离与清理

所有最终源原生运行使用 `.toolchain/post021-windows-final/**` 独立数据根、合成凭据和 loopback 夹具。日常管理器 PID 24140 未操作；没有真实同步、真实 WebDAV、发布或合并 main。结束后已停止本轮隔离管理器、服务和客户端，并复核无对应监听端口。

最终任务保持 `in_progress`，本回交不生成最终 CERT，不关闭十项整版验收；G3/G4 仍需物理桌面、MSVC、正式 Codex 默认 adapter 及最终集成发布门。
