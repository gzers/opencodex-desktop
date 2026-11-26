# TEAM82 Windows 复验交接前置

2026-11-26。按 TEAM82／IMP228（治理提交 633f0d086438592aeacc2ce5ae8757b1c0fa3d9b），Mac 独占首次供应、关联界面和候选打包装配的共享改动。Windows 接收修复制品做真机复验并独立回交证据。复查时远端 main 仍为 a69ae55a3fe21b58f415da727b0d1bdff8f37568，已知缺包失败仍在该 SHA 的 Windows 候选上。accepted:false。

## 待证伪判断

完整包在断网真实桌面自动准备并显示本地解包、校验和进度。独立缺包夹具可暂缓和重开，固定载荷仅在用户确认后请求。环境就绪后的第一次 Status 自请求发出起使用 5 秒预算，无法确认时显示「运行状态未知」，提供重试和诊断并禁用 Start。明确空闲后才执行 Start → Status → Stop，分别用 30／5／10 秒。

## 复查结果

| 项目 | 结果 | 依据 |
| --- | --- | --- |
| 共享修复归属与远端输入 | pass | TEAM82 已明确分工。[远端与会话原始读回](preflight.json) |
| 原候选 MSI／NSIS 摘要 | pass，复用 | [固定制品复算](../team81-20261126-resume-r1/raw/digest-recheck.json) |
| 完整包断网自动准备 | blocked | 日常连接依赖本机 RDP。尚无可独立断开来宾网络并继续操作的 Windows 测试桌面。[环境前置及原错误](../team81-20261126-resume-r1/result.md) |
| 缺包暂缓、重开、确认门槛 | pass，复用 | [原实机回执](../team81-20261125-r1/result.md) |
| 确认前无下载、确认后下载 | blocked／fail，复用 | 确认前 WebView 的 443 连接用途未确定。确认后首个产品错误为 managed_dependency_missing，发生在下载请求之前。[观察记录](../team81-20261125-r1/raw/missing-proxy-observation.json) · [首错](../team81-20261125-r1/raw/runtime-install-guard.txt) |
| 首次 Status、未知交互及启停 | blocked | 新候选和隔离实机回执待交付。原 Windows 构建的后端阶段日志为空实现，普通管理器日志的启动时间不能替代首次请求发出时间。 |
| W2／真实双 SID | blocked | 同源冻结待交付。当前 Active 为 Administrator 控制台和日常账户 15119 的 RDP；既有普通账户 marenben 为 Disc。原 TEAM79 的普通用户 MSI 首错为 1625，安装环境须在下一候选前具备普通用户 per-user MSI 前提。[TEAM79 原状态表](../team79-20261124-r1/sweep/summary.json) |
| 本轮清理 | pass | 本轮查询 Git 与会话，未启动制品、创建运行夹具或改动网络及机器策略。历史原件保留，marenben 仍启用。 |

## 下次实机输入

Mac 在共享修复后回交 main 源码 SHA、Actions run、Windows artifact ID、MSI／NSIS SHA-256 与打包配置。Windows 从远端读回，复算包摘要，再用两个全新绝对隔离根验证。完整包用可独立断网且保持桌面可操作的环境；缺包网络观察区分应用载荷请求与 WebView 背景流量，记录确认动作和首次载荷请求时间。首次 Status 采集须在页面发出 get_status_snapshot 之前就绪，同一时钟记录请求与结果或未知界面，保留重试、诊断与 Start 状态的原始证据。单独调用 Status 可作诊断，不能作为启动页首次请求回执。失败时保全首错。已拒绝的自动化入口留存原回执。

两端定向通过后，以新的同源冻结账本为 W2 输入。到时重验空闲普通账户安装基线及同时 Active 的两个真实 TokenUser SID，再按最新合同执行四格安装、隔离启停、更新失败恢复、卸载及清理。
