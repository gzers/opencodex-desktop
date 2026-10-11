# 0.1.10 main@c1e3c09d 候选核验与剩余门禁

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。这里的“最终核验”仅指本轮候选证据收口，不是 0.1.10 整体验收。源码为 `main@c1e3c09d8ed33238543ce5943e439991b8099088`；代码已推送，本文和证据归属 `docs/governance-main`。IMP / TASK 保持 `in_progress`，`release_authorized: false`、`release_accepted: false`。未迁移真实用户数据、替换日常安装、创建版本标签或公开 Release。

## 本轮结论

| 检查 | 结果 | 证据边界 |
|---|---|---|
| 同 SHA 完整 CI | 五个 job 成功 | 不等于设备原生验收 |
| Windows 候选 | 单次触发成功；ZIP / MSI / NSIS 独立大小与摘要核验一致 | 安装后 EXE 摘要只由 runner 回执证明 |
| Windows runner 新装 / 首屏 / 卸载 | MSI、NSIS 均 pass | 沙箱覆盖数据根，脚本强杀；不能证明默认根、目标双权限、正常退出或物理 DPI |
| macOS 原件身份 / DMG | arm64、0.1.10、完整源码 SHA；DMG 校验成功 | 原件严格签名失败，公证未验证 |
| macOS 隔离副本生命周期 | 正常退出、释放锁、同根重启、重新持锁；备份摘要未变 | 改 identifier / ad hoc 签名的测试副本，不替代原件签名和真实升级 |
| macOS 原生局部画质 / 备份树表 | 准备态高 / 中档浅深色光层可见，低档无光层；根 / 年展开、月份收起；重启回读记录 | 截图观察，不是完整矩阵、连续动画、焦点删除或复选框验收 |
| 性能、真实更新 / 事件、正式签名 | 待验；原件严格签名明确失败 | 原预算与 TASK 所需检查均保留 |

## Windows 候选及安装器身份

[完整 CI 38134982244](https://github.com/gzers/opencodex-desktop/actions/runs/38134982244) 在 2026-10-11 19:43:13（UTC+8）完成最后构建 job，整体 `completed / success`。frontend、release-tools、backend、backend-windows、build 五个 job 均成功且 headSha 为本轮源码。

19:45:41 查重后只 dispatch 一次 [Windows 候选 38136743628](https://github.com/gzers/opencodex-desktop/actions/runs/38136743628)。锁定依赖回归 19:53:43、安装器构建 19:59:25、安装 / 首屏 / 卸载 20:00:43 完成；整体 20:00:57 `completed / success`。不重复打包，不混用旧 369a 包。

| 资产 | 大小 bytes | 本机重算 SHA256 |
|---|---:|---|
| installer ZIP，artifact 11702111331 | 7562414 | `c7b66cb7ebc7b2a38369b6ca3c35d7051fda5afa7d57f55c5293ed9806a2b16c` |
| runner evidence ZIP，artifact 11702016494 | 1703512 | `3add893cca8644b28772e1e5f32a8b622171661648270b98544498758ea53b1e` |
| MSI | 4538368 | `9b46cb725cdbef0a70e482dc027185f6f2d5d8998cd6ad06eb5adff513b02d92` |
| NSIS | 3231880 | `78abbd92eda79b6bf2ea91ed4e648afe1dfb63424479cd9726c6e9254d775f3e` |

ZIP 与 API 摘要 / 大小一致，CRC 与安全相对路径检查通过。两个 artifact 的 `installer-verification.json` 字节相同：1970 bytes，SHA256 `2df775be0bfc0b89fed06bddaf40811328ef5b46605a494f54e4118f28499cd2`。预计到期分别为 2026-10-25 20:00:47、20:00:50（UTC+8）；临时资产链接不是稳定发布链接。

runner 的安装后 EXE 摘要分别为 MSI `1e2d107dcf0cc847073bb4f9d3b60bff1622983d299649eae129aa0913620b20`、NSIS `a0c53c5bfd70d1eab06d0bfd0700d98b73bc6fbf650e962bc1aaa1b54ad54a57`，均与预期 bundle marker 处理后的哈希一致；marker 为 MSI / NSS，offset 7984469。它们不同于构建原 EXE `d00e9920249fa7c85a0b496f989e6ce074ce9cc5865f67e391380ec9efc66c56`。本地 artifact 不含安装后 EXE，不能声称独立重算已完成。`codeSigned / releaseAccepted` 为 false。

两种安装器新装 / 卸载 exit 0，最终 EXE 已移除。启动 30 秒后窗口响应、WebView 已挂载、关于显示 0.1.10 / Windows、Node/npm 发现、OpenCodex 未发现、CLI 关闭、沙箱初始化。环境为 Windows runner image `win25-vs2026 / 20260925.250.1`，CDP Edge `153.0.4234.48`，Rust / Cargo 1.99.0、Node 24.21.0、npm 11.19.0、Tauri 2.11.4。

`startup-diagnostics` 的权限字段是 PowerShell harness 的 token，不能当成目标应用 token。启动显式覆写沙箱根，不证明 `<安装位置>\data` 默认路径；脚本最终强杀不证明普通退出 / 锁释放。两张 CDP 图中透明侧栏 / 背景呈黑色，部分文字对比度低，原因未确认。后续须在目标设备同状态比较整窗截图与 CDP 图，记录 WebView2 / 主题 / 画质 / DPR / 实际进程权限；云母及原生外观门禁保持待验。

## macOS 原件与隔离副本

干净 main 工作树执行 `cargo tauri build --bundles app,dmg -- --locked`，exit 0，用时 129.91 秒。

| 原件 | 大小 bytes | SHA256 |
|---|---:|---|
| DMG | 4384913 | `821724953dedc7f8ef782d23834edc7f9c035a4bf8b65be0ac88a63989a0b541` |
| opencodex-desktop | 7897776 | `288f36c20354d377d55b29ce430c36f0383e2290e38c713a2167ce684e2dcca6` |
| ocxd | 386416 | `ee294c65ccc22f6f738e1dbcfce019ccf02ee848fee2ccbec17f00b5eaafc6c1` |

版本 / 构建号 0.1.10，arm64，完整源码 SHA 内嵌，`hdiutil verify` exit 0。原件 `codesign --verify --deep --strict` exit 1：资源封装缺失；签名为 linker ad hoc、无 TeamIdentifier，本机有效代码签名 identity 为 0。保留失败原件，正式签名 / 公证门禁未闭合。

只读挂载 DMG → ditto 到独立测试位置 → 卸载镜像均 exit 0。仅测试副本改 identifier 为 `com.gzers.opencodex.desktop.sandbox0110c1e3` 并 ad hoc 签名；副本 strict verify exit 0，EXE 7852000 bytes、SHA256 `b32c9a8d07955bcd5c0b3edaec34ba7ef738c9444bf0cfd7ac5af204077508f2`。这一摘要和签名结果不属于原件。

沙箱根为 `/private/tmp/ocx-0110-c1e3-candidate-hqjf10k6/test-data`，boundary 为其候选临时父目录。19:45:58 独立快照证明原生退出后进程消失、原锁文件非阻塞独占可取得且字节未变。19:46:12 显式同根重启，19:46:21 证明新 PID28269 / 三项沙箱环境 / 完整源码身份正确、锁重新持有且 metadata 对应 PID。20:06:19 复核仍只有一个测试实例；旧 369a 锁可取得。日常 `/Applications/OpenCodeX Desktop.app` 的 PID75764 / Oct 9 22:00:24 启动时间未变。

### 原生截图的有效范围与无效尝试

- `03/04/09/10/13/14`：准备态、高 / 中 / 低 × 浅 / 深局部观察；高 / 中档顶部和左侧有光层，低档无光层。不能以静帧证明动画 / 原生性能，也没有覆盖 ready、标准 / 紧凑的完整矩阵。
- `05/06/07` 是中档选择过程尝试，不能仅凭文件名记已切换；`08` 才是选择后回读。
- `16/17`：备份生成后根 / 年展开、月份 10 收起。`18-keyboard-to-payload` 文件名不准确，实际焦点在 `backup-manifest.json`；不计载荷聚焦成功。
- `19`：刷新后展开层级保留，尚不能证明焦点节点被移除后的回落或含复选框行对齐。
- `22` 用了半尺寸坐标，误点未切入备份；`23` AX 有记录但树表在可视区域外；`24` 滚动后才证明树表和备份记录可见。

重启回读 `bk_20261011114045_e3fa4410`（界面 19:40:45 / 1.3KB），根 / 年展开、月份收起，保留 10 份 / 30 天、固定与恢复入口可见。仅创建管理器偏好备份，未执行恢复、清理或固定。清单 schema 1、file_count 1、standalone、pinned false、related null；实际只有两个文件：

| 文件 | 大小 bytes | SHA256 |
|---|---:|---|
| backup-manifest.json | 562 | `90fb16b9acf4f8c2e1dd7cde2c3fcedb22e2ec85d7380311109e46249b9532a6` |
| preferences.json | 1369 | `1a5895d9c89e7144c3353cc3306ce6b0ee1028777c749bb6613adc1d7f1a86ad` |

载荷为 JSON 对象，清单摘要 / 大小匹配。创建后至退出、重启及后续回读的六次快照字节摘要一致，末次复核仍一致。它不包含面板完整配置、会话或登录状态，也未测试恢复。

## 证据保留与复核

`evidence/` 保留 153 份有来源的原件，以及 provenance 和方法；`SHA256SUMS` 的 155 项、全部来源大小 / 摘要及无损 roundtrip 已逐一核验，总计约 9.92 MB。见 [archive-verification.json](archive-verification.json)、[source-provenance.json](evidence/source-provenance.json) 和 [SHA256SUMS](evidence/SHA256SUMS)。

JSON / 脚本按原字节保存；日志和 AX 使用 mtime=0 的无损 gzip；原生 `.image` 实际是 JPEG，仅改扩展名为 `.jpg`。失败和误点截图也保留。10 份大文件仅保留摘要：安装器、ZIP、原 DMG / EXE、重复安装回执及大型 Cargo 元数据。不复制二进制、真实或测试用户载荷、WebView profile。源临时位置只是构建来源，不是长期证据唯一入口。

相关背景光修复测试证据仍引用 `.adg/evidence/OCX-0110-20261011-C1E3-MESH-CENTER/`，本轮不重复完整 UI 测试，也不把浏览器几何 / draws 检查转换成原生验收。

## 后续门禁与执行顺序

1. 受控性能评估：先固定 0.1.9 与 c1e3 的场景、画质、窗口 / 可见性、预热、顺序重复、采样方法及网络 / 主线程仪器，再执行配对。旧顺序十分钟观测只保留信号：58.34876% vs 54.67963% 单核、差 +3.66913 百分点，不能认定因果或通过。c1e3 几何定位修复不解释这项差值。
2. 调度门禁：CPU 增量 ≤0.5 百分点、RSS 增量 ≤10 MiB、启动 p95 增量 ≤100ms 且 ≤5%、无 >50ms 主线程长任务；冷 / 热各至少 20 次；100 次切页 / 10 次重启的缓存命中零请求 / 零探测进程；stable 24h / beta 6h 的真实周期、慢网 / 离线 / 恢复 / 单飞与退避证据。加速时间夹具不能代替真实周期。
3. 双平台同 SHA 原生验收：macOS 补完整准备 / 就绪 × 标准 / 紧凑 × 画质 / 主题、复选框 / 箭头 / 文字对齐和焦点；Windows 补实际目标普通 / 管理员 token、安装位置默认数据根、物理 DPI、整窗 / CDP 外观对照、IPC、正常退出 / 锁释放及安装升级。继续只保留一个原生测试实例。
4. 真实两类更新 / 事件：详情与网页入口、更新内容来源、进度、备份失败阻断 / 恢复前备份、重启回读、异常按实体精确解除和通道隔离。0.2.0 文件同步 / WebDAV 原型不构成本版本实现。
5. 正式资产：macOS 签名 / 公证、Windows Authenticode、更新源 / 签名 / 资产一致性审阅，以及 TASK 八项 required_checks 全部有相应证据。稳定核心事实晋升、真实用户迁移和公开发布仍需独立授权。

本轮仅产生局部核验回执，不创建证书、不关闭 TASK / IMP。
