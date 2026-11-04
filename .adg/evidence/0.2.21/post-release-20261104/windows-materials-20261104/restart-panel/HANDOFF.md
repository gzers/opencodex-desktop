# Windows 重启与面板贡献回交

采用 `docs/governance-main@6a3188d6c09a1f25bb0fdeb44b91b5455fd5216f` 的 IMP-30 / DMD-POST021，负责 S21-POST-09、S21-POST-10，分别绑定 AC-P21-09、AC-P21-10。代码贡献 HEAD：`4b0c357b12d331de8dd49f8209056394e93a728d`，分支 `codex/post021-windows-materials`。本目录随 `docs/post021-windows-materials` 回交，文档 SHA 以该分支提交记录为准。

本次不改共享根 store、入口、默认配置、渠道命名、模型发现或同步业务，不合并、不发布。原材质贡献 `684e85c29`、旧 TASK/RCP 和旧性能失败结论保持原样；本回交不替代原材质验收。

## 根因与实现

POST-09：真实 Windows 基线中缺失 `runtime-port.json` 时，Management 绑定经备份安全读文件助手抛出通用 Storage 错误，界面只显示 `manage backups / os error 2`。这不是“必须先打开面板”。同一最终流程中，在管理器从未创建面板子视图时恢复运行记录，仍能实际完成重启。

`e568356af` 分流缺失/损坏/不可读运行记录、不可达服务、失效绑定、缺失管理令牌和缺失客户端/保存文件；消息分别说明“配置已写入”和引用准备/重启尚未完成。失败或未确认重启时保留已保存卡片、具体错误、确认与重试入口，暂停倒计时。只有 `success && restarted` 才结束反馈。认证、运行证明、来源、路径及并发守卫保持。`c5e19db0a` 对齐既有集成测试的失败后重试合同。

POST-10：基线每次进入都重新读取偏好；原总等待计时在 Show 返回后才覆盖导航，不能约束偏好、状态、地址和原生创建等待。原生子视图复用/重建没有独立的权威就绪状态，晚返回及浮层隐藏存在竞态。本机 smoke 主窗口有显式隔离 profile，而基线子视图走默认 profile，另起浏览器环境放大首次创建成本。**不能把 smoke 特有的双 profile 直接归为所有正式配置的原因。**

`f03c09f72` 复用已加载偏好，记录偏好/状态/地址/原生队列/锁/环境/视图/显示/导航阶段；原生与界面均有 8 秒边界。不可取消的 blocking 创建通过代次失效，晚返回不发布；超时释放界面等待、显示明确失败并允许重试。子视图 profile/参数与主窗口一致。`47d9b7593` 保留早于 Show 回包的同代页面就绪事件；`cc62887fc` 将新子视图先建在父窗口裁剪区外，代次确认后再放入可见范围。`4b0c357b1` 补齐未运行/不可用的观测终态并清除遗留 loaded 标记。没有把 STA COM 对象跨线程传递，没有 unsafe Send 或依赖升级。

## 原生证据与矩阵

Windows Server 2025 build 26100，WebView2 156.0.4314.58，Intel UHD 770 / ANGLE D3D11；本次最终窗口 PID 744、DPI 192、显示缩放 200%，RDP 会话。OS/GPU 快照沿用同宿主本批材质采集，本次另查窗口 DPI/远程标记。仅声明该环境。

最终 Windows GNU EXE 启动日志为 `4b0c357b...`；源码、前端资源及 EXE 哈希见 `source-manifest.json`。GNU 本地驱动仅在隔离 EXE 副本增加 Common Controls v6 资源，前后哈希及原因见 `raw/candidate/manifest-adaptation.json`；产品源和 MSVC 发布配置未改。补充原生样本/单测来自 `cc62887fc` / `9a4f851e2`。最终 HEAD 仅再补 PanelRoute 的未运行/不可用终态与清除旧 loaded；Rust 生产源相同，差异路径在 source-manifest.json 明列。最终 HEAD 另外重建并复验主要流程。

| 场景 | 结果与证据 |
| --- | --- |
| 基线缺运行记录 | `raw/baseline/missing-record-repeat.json` 精确复现 storage/manage backups/os error 2；`before/restart-missing-record-native-0.jpg` |
| 修复后缺记录 | `runtime_record_missing`，明确指出 runtime-port.json，说明无需先开面板；原生错误和重试见 `after/missing-runtime-record-native-0.jpg` |
| 损坏记录 / 过期监听 / 管理令牌缺失 | `corrupt-record-final.json`、`stale-listener-final.json`、`missing-token-final.json`，错误码分别为 runtime_record_invalid / runtime_unreachable / management_auth_unavailable |
| 服务实际停止而记录仍在 | `service-stopped-final.json` 为 runtime_unreachable；不冒充文件缺失；最终 HEAD 的 delivery-stopped-terminal.json 为 not-running、loaded=false，原生图 after/delivery-service-stopped-native-0.jpg |
| 待处理改名时 config.json 缺失 | `missing-saved-config-final.json` 为 restart_saved_config_missing，未重启、待处理保留；各故障注入均在 finally 恢复隔离文件 |
| 普通保存，服务运行且从未开面板 | `no-panel-presence.json`、`save-no-panel-valid.json`、`no-panel-valid-owned-restart.json`、`no-panel-valid-readback.json`；补充 cc628 的 92812→59060 读回一致。最终 HEAD 另有 delivery-ordinary-*：43480→48044、contextWindow=65539，noNativeViewSinceStartup=true；启动后没有创建子视图日志 |
| 普通保存，已打开过面板 | 最终 HEAD 的 `delivery-after-panel-save.json`、`delivery-after-panel-restart.json`、`delivery-after-panel-owned.json`、`delivery-after-panel-readback.json`；48044→92572，contextWindow=65540，读回一致 |
| 改名保存、当前选择及实际重启 | 补充 9a4f851e2 的 `rename-local-fixture-final.json` / `rename-local-stable-final.json` / `rename-local-readback.json`；Fixture→FixtureRenamed，74220→95772，实际 Codex 0.166 原生运行体将选择改为 FixtureRenamed/raw-model，待处理清除；`after/renamed-client-readback-native-0.jpg` |
| 前台首次建子视图 | 基线 `panel-cold-foreground.json`；最终候选 `delivery-panel-cold.json`；前后原生图见 before/panel-ready-foreground-native-0.jpg、after/delivery-recovered-panel-native-0.jpg |
| 再次打开 / 暖服务 | `panel-warm-final.json` 和 `delivery-reopened.json`；均完成显示，复用时不等待不会再次出现的页面事件 |
| 服务新进程 / 地址变化后首次打开 | 两端 `cold-service-completed.json` 和 manager.log；candidate 新服务 PID 8620、60434，321.3ms 完成当前 trace，旧子视图重建 |
| 有浮层 / 关闭恢复 | `delivery-overlay-open.json` / close；`after/delivery-overlay-native-0.jpg` 中原生页隐藏，通知可见；关闭后恢复真实官方页 |
| Show 前等待超时 / 晚结果 / 重试 | 最终 HEAD `delivery-timeout.json`：隔离注入地址依赖 pending，8010.9ms 转失败；恢复后旧结果没有覆盖失败，`delivery-timeout-recovery.json` 96.7ms load-ready；原生失败图 `after/delivery-timeout-native-0.jpg` |
| Show 本身 pending、晚回与浮层竞态 | post10-panel 单测覆盖；原生正常 Show/导航/浮层另测。没有把地址依赖注入宣称为实际 Windows OS 创建线程卡死 |

上述重启通过未修改的固定官方 `startServer`、管理接口、`performCodexRestart`、`restartCodexDesktopApp` 梯形实现执行。测试依赖只把桌面发现/退出/拉起限定到 `OwnedClient.exe`；该进程是真实 Windows GUI，启动即读隔离 config.toml，并输出带 PID 的读回文件。**实际退出/重启的是自有验收壳，未重启日常 Codex，也没有验证正式打包 Codex 桌面壳的默认发现/启动 adapter。** 当前选择写入由真实已安装 Codex 0.166 原生运行体执行；不拿自有壳冒充官方打包桌面壳。

所有凭据、配置、服务及 GUI 数据均在独立根。改名初始 fixture.invalid 被官方 DNS/目的地策略拒绝，随后改用自有 127.0.0.1:60433 上游，**仅在隔离 fixture 配置设置现有 allowPrivateNetwork=true**，未改产品准入判断。模型服务只返回合成清单、不转发推理，无真实 WebDAV。官方包安装未执行生命周期脚本。

## 耗时口径

同宿主、前台、中档、同官方 GUI 2.84.0、主窗口已启动、进程内首次建子视图：基线从路由动作至 load-ready 轮询确认 **2419.2ms**；最终候选 **302.8ms**。最终候选自身 trace 在 **275.3ms** 达到导航/显示就绪，100ms 轮询晚一帧确认。原生阶段相对本次请求：队列 0、锁 8、地址 17、浏览器环境 25、视图创建 144、显示 153ms。基线偏好额外读取约 5.8ms，首次导航的主要延迟在原生浏览器环境/创建至导航开始之间。

热复用约 101ms 的轮询确认不应误称冷启动；发生地址变化时更不能只看遗留 loaded 标记。本目录冷服务首轮旧采样 `cold-service-first.json` 曾过早停在 loaded=true/“正在嵌入”，**不作为 ready 耗时证据**；以完整 trace、后续原生画面和 manager.log 为准。新驱动同时要求当前可见提示为已嵌入。采样属于 load-ready/原生显示阶段，不是逐像素首帧光学计时，不推广为所有正式安装的固定性能提升比例。

## 检查、失败与后续

- 前端全量 196 文件 / 1283 测试通过；Vue 类型检查及 Vite 构建通过（保留原有 chunk/动态导入提示）。
- Rust 定向 25 cache_refresh + 3 management + 10 panel = 38 测试通过；两个零匹配过滤器未计入。未声称全库 Rust 994 项已运行。
- cargo fmt --all --check、cargo clippy --locked --all-targets --features integration-test -- -D warnings、最终 custom-protocol Windows 构建通过。Clippy 组件曾缺失，安装现有工具链组件后实际通过，未升级工具链。
- 初次前端全量 3 项旧断言失败：旧测试要求失败也关卡片/发 toast；按新产品行为改为保留卡片和 modal 错误及重试，全量重跑通过。第一次面板断言、不可发送 COM 尝试的编译失败均保留在 failures/，不当最终通过。
- 早期 no-panel 样本实际启动时自动开了面板，已移为失败分类并重跑；早期 invoke 包装失败和 cached-loaded 采样错误不用于性能结论。截图首次抓帧可能滞后于激活，交付图已按目标及稳定帧核对；误抓的非目标窗口帧已删除，未交付。
- 改名初次身份尚未探测完成、目的地拒绝和一次 saved_runtime_drift 保留；后续稳定重试完整通过，没有放宽并发/归属保护。

贡献实现和本机隔离验证已回交，TASK/RCP 保留各自冻结合同，不写共享 DMD/IMP/问题清单，不生成整版发布证书。L 统一集成后仍须在同一最终提交补 Mac 受影响重启/面板、Windows 其它贡献回归，以及正式打包 Codex 桌面壳默认 adapter 的授权隔离验证、原包安装/升级/签名/更新与回退。原材质远程桌面性能预算失败也仍待集成者处置。
