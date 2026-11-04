# Windows 最终补验回交（复核更正）

采用代码 bd7c0fe8ccecfa39e0706c885dc233f3854dab74 和文档 59b319cf5cac0462985dd969d929f0f8fa8a6168。代码贡献分支 codex/post021-windows-final，当前 b2cb82cdee172c16bd597a6b2017b54d40d11d5f；文档贡献分支 docs/post021-windows-final。只改验收驱动，apps 树与指定集成源完全一致。产品运行身份以原生日志 bd7c0fe8c、EXE 哈希 76dc1049…54b778a 为准，新增测试提交没有冒充重建产品。

本文件纠正 68c03436 中不充分或错误的通过声明。旧提交和失败仍可追溯；本批总体结果为未通过，不生成 CERT，不关闭集成验收。

## 实际验证

- 前端 197 文件 / 1289 测试、类型及 Vite 构建通过。GNU debug custom-protocol 构建通过，仅隔离 EXE 副本补 Common Controls v6 资源，不是 MSVC 正式制品。
- Rust 首次全库 981 通过 / 17 失败 / 4 按原条件忽略。17 个失败均缺显式 OPENCODEX_TEST_NODE；设置 Node24 后对应 Windows 组 18 项通过。定向 cache_refresh25、management3 通过，面板11项在原全库日志中通过。没有执行“显式 Node 后全库再跑”，不把分组复验拼成一次完整绿测。原始及复验日志见 checks/、reviewed/backend/。
- 最新原生探针单测45项通过。reviewed/model-regression.log 的完整 windows-webview.mjs 已实际 PASS；此前多个失败各自保留。完整 model-ui、native021-behavior、保存通知倒计时、焦点、层叠 X/Escape、取消/确认、模型排序、导入、相邻对象及 Key 保护见 reviewed/model-regression/。
- 改名备份以真实 WebView2/Tauri 保存产生的文件执行断言：普通保存恰一份写入前备份；改名额外恰一个 rename-UUID 恢复目录，配置前后镜像匹配，旧备份保留。实际备份字节、最终配置和返回稳定 ID 见 reviewed/model-regression/backups/ 和 config-readback.json。L2 已采用 provider.name 更新，旧预期遗漏该字段；新反例仍拒绝旧名/丢名、未知字段或相邻对象变化。
- reviewed/native-first-scope-reviewed/sync-setup.json 在全新数据根核对全部本地/远端可用渠道和模版 ID 与首次所选 ID 精确相等，并读取真实复选框。此场景远端夹具仅含偏好，remoteObjects=0；证明一条本地合成渠道及17个本地模版首次选中，不能外推为新设备远端独有对象场景。
- reviewed/native-readonly-reviewed/ 实际通过时间未知、已知版本、无快照、读取失败四态，真实 TLS 夹具仅服务 127.0.0.1:60441。reviewed/readonly-reviewed-summary.json 证明配置、连接、范围、保存时间凭据四个文件的前后字节摘要一致；https-requests.json 保留实际只读 GET。
- reviewed/native-channels-reviewed/ 中，中文名称、自动生成 channel-* 内部键、真实 loopback 获取 vendor/native-model、保存后改为 openai 显示名并完全重开、节点检测及配置不变均有证据。第一次创建脚本在错误的 aria-label 选择器处停下，先前已通过的获取/保存断言保留；补充 channels-reopen 用真实编辑器字段完成余程。models-reviewed-requests.jsonl 记录两次认证清单 GET、零 POST；未改产品私网准入逻辑。
- native/native-restart/ 缺 runtime-port.json 时具体前置、无需先开面板、重试入口和配置字节不变通过；只是缺前置失败恢复，不是正式 Codex 默认重启成功证据。该场景先用合成配置种子，保存通知通过实际生产宿主显式打开，未声称自然保存入口的完整正式桌面重启通过。

## 面板新增失败及恢复

reviewed/native-panel-reviewed/ 中真实官方 GUI 2.84.0 的常规重开与浮层恢复通过，使用未修改包的 startServer；测试管理依赖拒绝任何桌面重启。原先固定 Bun1.4.2 二进制已不可用，本次使用独立 tools 副本 Bun1.4.4，只运行隔离根。

服务端口从60444变为60446后，快速离开/重开在15秒观察内停于“官方面板加载失败”，服务健康正常。reviewed/native-panel-address-reviewed/panel.json、manager.log 保留失败，日志出现多次队列/锁与Show/Hide交错；根因未确认，不声明是空URL修复本身的问题。reviewed/native-panel-recovery-reviewed/ 证明点击真实“重试加载”恢复，之后两次常规重开及通知浮层恢复通过，配置字节不变。native-official-panel-0.jpg 为 Windows 原生窗口稳定帧。

快速路由动作使用真实生产路由 store，间隔10/30ms，是明确隔离的竞态驱动，不是物理鼠标节奏。原生旧视图 close 失败注入未执行；其归属/重试只保留源码及11项面板单测证据。新切源失败仍需集成线复现定位。同步方向与有界ETag在实际 Rust 测试中覆盖，本次没有另跑 Windows 原生发布确认流程，不能把只读 GET 当成原生发布验收。

## 环境阻断

用户确认“没有，只有rdp”。当前为 Windows Server2025 build26100、WebView2 156.0.4314.58、Intel UHD770 ANGLE D3D11、DPR2/200%缩放。RDP采样 native/material-performance/performance.json 只作该环境观察；沿用原 B=16.667ms、p95≤2B 和长帧检查，不能替代物理桌面性能门。此前基线/候选RDP失败不改写。

reviewed/tooling-reviewed.json 复查 cl.exe/link.exe/vswhere 和标准安装/SDK目录均不可用，MSVC正式构建未执行。GNU成功不代替正式MSVC包。

正式 Codex 默认 adapter 只读发现 MSIX/AUMID；listProcesses 返回 null，表示枚举未能完成，不表示无进程。旁证显示日常同包 ChatGPT.exe 正在运行；默认发现/AppsFolder 重启不按 CODEX_HOME 限定实例。未调用正式默认退出/重启/读回，没有用 OwnedClient 代替。当前无可证明独立的一次性用户/实例，保持缺验。

## 隔离越界事件与清理

前一阶段固定PID清理 Stop-Process 包含30196，它的实际路径是日常安装目录 D:/Software/Develop/OpenCodeX Desktop/data/runtime/opencodex/node_modules/bun/bin/bun.exe。这越过用户隔离约束，不能再宣称日常进程完全未动或隔离检查通过。随后观察到服务新PID66908（2026-11-05 01:45:17+08），管理器PID24140仍为2026-11-04 19:58:15+08启动；没有对新服务做补偿重启或更改日常配置。

未取得越界前后日常配置字节证据，不能认证该事件零副作用。详见 isolation-incident.json。复核阶段清理按隔离EXE/脚本路径、PID和创建时间匹配；cleanup-reviewed-final.json 证明隔离管理器、服务、Python夹具和指定监听端口已清理，当前日常管理器/服务未再操作。

总体 required_checks 中 physical-material-performance、msvc-build、packaged-codex-default-restart、native-regression（切源失败）、isolation 均未通过；回执须相符。所有任务保持未完成，旧冻结任务/证书不重绑。未合并main、未发布、未跑真实WebDAV或推理。后续需要可用物理桌面/MSVC工具链、可证明归属的正式桌面实例，以及共享面板切源失败的定位；由本机统一集成和发行。

## 指定源 CI 补充证据（不代替本地缺验）

CI run 41868797161 关联 head bd7c0fe8c，实际PR测试合并提交7dff65891af38d47db6a7ce0b4f1c93f754a265f；已fetch并核对两者 apps 树均为0eec77398bbd1cbe49f88dd3bd5e8046266e14c1。Windows运行器win25-vs2026的后端回归、Clippy及 release custom-protocol 构建步骤成功，最终EXE原生冒烟失败，不能写CI全绿。

制品13346024660的20,453,585字节ZIP SHA-256为559f2a33e70be75bb29585bd36ca932e0acb833c2a7ef0e6ad6224ca71128113，与GitHub返回摘要相同。归档仅有库测试EXE及诊断，没有release应用EXE。startup.json证明实际release应用启动、源码身份匹配，但普通用户冒烟未完成；native021-behavior.json的失败是改名后provider.name新值与旧探针预期不同，与本地已修探针相同。原始CI报告、归档索引与任务步骤见ci-supplement/。

本机构建身份已复查为普通权限（非管理员），MSVC/SDK仍缺。远端构建证据只能补充同产品源信息，不替代本地MSVC构建/候选EXE复验，也不替代物理桌面性能。正式Codex默认重启仍无可安全限定的独立实例，未调用。总体目标未达成。
