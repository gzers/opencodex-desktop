# Windows 单实例局部门禁与共享树表对齐

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182。原始 TASK-182 仍未同步，本轮没有构造正式 RCP 或改变 required checks。证据归 `.adg/evidence/OCX-0110-20261012-WIN-CHILD-APPDATA/`；隔离二进制与数据根在代码 worktree 的忽略目录，不将它们提交到文档分支。

## 代码与候选身份

`0a83761a99bdcb721d929f33e4c60c2e95eb6224` 修正显式沙箱的子进程 AppData：Known Folder 只用于日常模式，沙箱使用自己的 HOME / AppData。真实 Windows 子进程断言同时核验 HOME、USERPROFILE、APPDATA、LOCALAPPDATA 以及未透传宿主 NVM 变量。该提交连同前两笔隔离修复经特性分支回归后快进并推送 main。

- 精确 feature CI `38202731337` success，条件跳过的 macOS bundle build 不计通过；Windows fmt / Clippy / 后端回归 / release EXE / 最终首屏通过。
- 精确 Windows 候选 `38203025770` success；main 集成 CI `38204104373` 后续也 completed / success。
- ZIP `11726204462`（候选）SHA-256 为 `c56aadac6d8caf35421eb9161705651900a3d2fa0408b2ea03af6c1f579e60eb`；ZIP `11726297619`（证据）为 `24f057c171dd3f6a0679dc526377ae822681b341baef215f7826297db60916df`，均与 GitHub digest 一致。
- MSI 摘要 `c916aff2d4ec79358fa7dd259e308b9658416643d7fbb52435359520f49c2b12`；只读提取的主 EXE 为 0.1.10，摘要 `fa89ba3f75d44b331cfb54df3ce75a74af0ce45cbf5ce907e50667b7945aaa2f`，与 CI 安装后 EXE 一致。本机没有执行 MSI / NSIS 安装器。

此前 `6b9abc3` 的普通 CI `38201999141` Windows 库测试为 494 passed / 3 failed / 4 ignored，失败都是既有 IPC 所有权断言的 2 秒等待超时。下载原测试 EXE 并核对 digest 后，三项在本机定向通过；默认并行的完整库测试为 496 passed / 1 failed / 4 ignored，唯一失败是可执行文件内嵌 CI 源码路径的 LICENSE 在本机不存在。该本机结果不作为完整回归通过，也不能解释 CI 超时因果。原 CI 失败、原断言和原超时均保留，未延长等待或减少测试；`0a83761` 的绿色结果不覆盖旧失败。`6b9abc3` 的候选构建 `38202038747` success 另按 SHA 保留，不当作最终补丁证据。

## 单实例与普通权限首屏

使用唯一容器 `test/runtime/windows0110/`，活动根为其 `data/`，沙箱 HOME 为 `data/home/`，WebView 数据为 `data/EBWebView/`。专用 supervisor 持有应用进程句柄和独占会话文件锁，拒绝同时运行另一测试实例。跨重启复用该容器，不开启不同测试根绕过保护。

首次普通权限启动 PID43792 已观测 `elevated=false / type=3`，但应用初始化退出，stderr 为 Tauri `resolve app data directory; unknown path`；失败现场保留。准备脚本补建测试 HOME 中尚不存在的 Roaming / Local AppData 目录后，同一根重试成功，PID41528，source startup log 匹配 `0a83761`，实际令牌 `elevated=false / type=3`。首次失败不计首屏通过，没有将 harness 令牌冒充应用令牌。

Windows Computer Use 选到唯一候选 HWND394058，实际原生截图和 AX 见 `native/ordinary-overview-*`；只读窗口查询显示 DPI192、WebView DPR2、窗口可见、非最大化、非最小化。前台时 app_foreground=true，DWM backdropAttribute=2 / result=0，应用回报 mica / nativeCaption=true，截图有原生标题栏及系统按钮。失去前台后另存 backdropAttribute=1 / inactive 观察。只覆盖本机 Windows Server 2025 build26100 的实际 200%；没有把 CSS 或 WebView 模拟缩放计为原生 100% / 125% / 150%。

初次发现状态缺少 Node / npm / ocx，证明该沙箱未沿用宿主工具。后续仅把程序 Node24.19.0、npm10.9.0 与固定包装器复制到测试 HOME `.local/bin`；没有复制用户配置或凭据，没有安装或启动面板。

## 真实备份、恢复与事件

下列均在同一个实际运行的候选中，调用真实 Tauri 命令或原生 UI，没有注入业务成功结果：

| 局部检查 | 证据与结果 | 边界 |
|---|---|---|
| UI 生成偏好备份 | `bk_20261012044452_a921f503`，根 / 年默认展开，月份收起 | 首份备份与默认展开，不代表全部树表交互 |
| 固定与不同值恢复 | 先在 UI 将 theme 改为 light，再固定原备份并调用恢复；theme 恢复为 system，偏好 SHA-256 回到 `ada85b4be000ec965a47f9bd2077ee736b6b06233fc2270c8323c356c4726200` | 恢复最终命令通过，UI 确认路径因输入故障待验 |
| 临时保护解除 | 恢复前保护 `bk_20261012045017_0609193d`，恢复后 protected=false / reconciliationPending=false；原用户 pin 仍 true | 不替代升级 / 重启保护或轮换全矩阵 |
| 精确事件恢复 | 测试根写入非法 backup policy schema99；真实备份失败投递 registered danger，管理器更新查询未解除；恢复原策略后同偏好内容成功备份才解除捕获 failureId | 仅覆盖该真实故障 / 重试及对象隔离；全部领域待验 |
| 面板缓存与通道隔离 | 真实 npm 元数据查询返回 2.83.0 与 integrity；切 beta 再 stable 后 panel-latest 缓存 SHA、next_due / failures / last_success 完全不变 | 查询完成后的缓存 / 截止隔离，不覆盖在途事务 |

面板查询持久状态 last_success=1791781246、next_due=1791867646，间隔 86400s；这是实际成功查询写出的未来截止，不是已完成真实24h到期观察。管理器也有真实手动检查截止；自动检查仍关闭，未将等待时间或后台进程存活冒充真实周期通过。

UI 在备份页面后续操作时出现 `failed to activate captured window`；按 Skill 刷新唯一窗口并重试激活后为 `GetCursorPos failed: access denied 0x80070005`。停止 native 输入，不推断应用崩溃或锁屏原因，不用旧截图坐标继续操作。后续备份恢复和事件证据明确来自真实命令，界面验收仍待继续。

## 树表失败与修复

原生 200% 备份树表观察到操作按钮中心比名称 / 箭头低 6 个 CSS 像素。`ordinary-tree-geometry.json` 记录名称控件22px、操作按钮34px却同顶边，导致两者中心偏差；失败截图和 AX 保留。

修复 `cdfd7061adc9dbf81b6b5eac616ff785d27f1e44` 已提交推送 main：共享组件以34px首行节奏对齐名称、复选框、箭头、图标、用途与操作；操作列通过 `kind=action` 标识，按钮保持34px命中高度，多行文字按第一行对齐。没有在备份页面另写偏移补丁。

浏览器 QA 渲染真实共享 Vue 组件，覆盖选中 / mixed / disabled / 无选择框、目录 / 叶节点、多行名称、两主题和 CSS 1 / 1.25 / 1.5 / 2 缩放。最初探针失败原件保留：Windows ESM 路径格式错误；第二版未检测旧组件的操作按钮，出现假阴性，修正定位器后才形成可用红绿。第三版旧组件实际失败（6px），修复后8组通过，控件误差≤1px且按钮命中高度保留。Chromium CSS zoom 不等于真机 DPI 或 WebView验收。

UI 全量95文件 / 476项通过，typecheck与生产构建通过；保留 jsdom canvas 与动态导入分块警告。几何夹具一度占用原 `qa/tree-table.html`，`4f7f33435ec1a7032e468363f191ddce50330fd8` 恢复其原字节，将新几何测试改用独立入口；独立入口8组再次通过。原QA及测试范围保留。当前 main=4f7f334，CI `38205914227`、Windows候选 `38206176747` 在运行；实际本机实例仍是0a83761，不混记为修复后的原生通过。

## 仍未闭合

原始TASK、管理员真实令牌、其余物理DPI、标题栏操作 / Snap、实际安装 / 升级 / 卸载保留、完整备份轮换 / 损坏 / 逃逸、更新详情与真实进度 / 重启、树表修复后的原生复验、全领域事件、在途通道隔离、真实30min状态错误退避、同机官方0.1.9配对性能和真实stable24h / beta6h均未完成。

原启动p95、CPU / RSS、主线程、100次切页 / 10次重启零查询预算不变；既有性能失败保留，IMP继续in_progress，不关闭或发布。日常EXE摘要保持，日常进程退出原因仍未知；本轮没有操作其进程、替换日常安装或迁移真实用户数据。
