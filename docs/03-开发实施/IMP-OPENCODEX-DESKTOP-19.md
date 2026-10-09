---
id: IMP-OPENCODEX-DESKTOP-19
object_kind: implementation.change
state: in_progress
demand_ids: ["DMD-OPENCODEX-DESKTOP-MANAGER"]
title: 0.1.9 用户画质与 Windows 窗口外观和菜单适配
summary: Windows 原生适配及固定689a86e6的CI、候选MSI／NSIS门禁通过；用户试用确认后已合入main，0.1.9公开并晋升稳定Latest，完整main CI与发布验签通过。发布结果以REL-09为准；Snap及跨平台／显示器／生命周期等专项仍保留in_progress，保留日常安装。
---

# 0.1.9 用户画质与 Windows 窗口外观和菜单适配

2026-10-08，Asia/Shanghai。用户要求接续文档提交 `bc6561acac5075bce76babfc367ff18067b17efe`，画质由高／中／低统一决定，不跟随系统减少动态、不增加动态开关；先采集下拉穿透原因，修复后覆盖动画、静态 WebGL、光场软边、前后台恢复及双权限/DPI，开发、测试后提交，**不推送**。

代码分支为 **`feature/0.1.9-windows-appearance`**，基线是已安装候选的 `origin/feature/0.1.8-windows-package@cffca3afc29b8b251f3f14535f997b5d08560cb2`。0.1.8 尚未并入 main，因此沿此基线保留已有 Windows 适配，不把文档分支当作源码基线。文档和机器证据仅回写 `docs/governance-main`。CHANGELOG 仍由后续 main 集成维护；本轮不合并、发布或替换日常安装。

## 版本范围补正

用户随后明确指出 [Windows 窗口外观与菜单适配专题](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261008-Windows窗口外观与菜单适配/README.md) 也属于 **0.1.9**。此前未建立该独立包，`8a8397a` 未实施原生标题栏／窗口外观和菜单栏适配；该遗漏现纳入本实施范围。Windows 补建入口时尚未取得原专题正文；本次由 Mac 补齐原文、来源与截图，接续设计见末节。原生实现及验收仍未完成。

“玻璃与画质改动已提交”不等于 0.1.9 全部开发完成。整体完成须同时闭合本专题及下面既有修复的门禁；保留 `state: in_progress`。

## 已定位原因与修复范围

- 下拉菜单在设置行与卡片的 `backdrop-filter` 祖先中，模糊采样被祖先 backdrop root 限定；原 `blur(8px)` 声明存在，但下一行在祖先范围外，文字仍清晰。相同 EXE、菜单填充/位置/blur 下，临时将菜单移动到主壳，或仅移除祖先 blur，穿透文字变为模糊，证明是嵌套采样范围，未发现 GPU 不支持。实现采用共享菜单浮层移到无滤镜主壳，保留缩放、选择、外部点击、Escape 与方向键。
- 画质唯一来源为用户三档；删除 JS/CSS 及管理器面板浮层中的系统减少动态覆盖。中低档静态，高档前台连续，页面/原生后台仍暂停且不改偏好。能力失败的 CSS 回退保留。
- WebGL 光场的“可以显示”与“可以连续动画”分开；中档静态首帧、状态颜色/主题/尺寸改变及返回前台按需重绘。
- CSS 光团改为在边界之前衰减至透明，容器横纵软遮罩防止弧形/矩形裁切；不直接恢复逐层 blur。WebGL 边缘另补横向渐隐，保持既有质量和 DPR 上限。
- 软件候选清单更新为 0.1.9。针对独立 sandbox/CDP 增加有界 100%/150%/200% 设备渲染比例验证入口，普通启动不接受该调试缩放。
- 原生最小化恢复发现旧 Tauri 键盘焦点不等于 Windows 应用激活：前台 HWND 已匹配且窗口显示，但 `AppActivity` 仍 false。Windows 改以根 HWND 的前台／显隐／最小化状态及 WinEvent hooks 更新唯一前台信号，保留无轮询调度与 macOS 接线。

## 本轮门禁与证据

所需检查：菜单原因与像素模糊、前端类型/单测/构建、后端格式/Clippy/回归、真实 WebView2 画质与媒体开关、静态首帧/尺寸重绘、前后台/最小化/隐藏恢复、普通/管理员 token 与三种实际 DPR、页面下拉菜单键盘/选择/缩放定位、软边截图与短时帧预算、日常安装前后身份。原生标题栏／窗口外观、应用菜单栏与托盘的专项门禁由新增维护专题接续；页面下拉用例不能作为其通过证据。

屏幕像素检查比较菜单相同填充下有/无 blur 的条纹对照，不能只凭计算样式判定玻璃。动画用实际 SVG 改变与 drawArrays 验证，中档应静止且保留光场。DPI 通过独立 WebView2 的设备渲染比例模拟并核对实际 `devicePixelRatio`；不修改桌面全局 DPI或声称已覆盖不同显示器/Windows 10/11。

运行测试使用候选 EXE和独立管理器/WebView根。临时状态投影、媒体模拟和条纹只用于隔离诊断，结束后清理；不生成真实业务状态、不启停日常代理、不安装升级。原始截图、GPU/浏览器/祖先样式、矩阵 JSON及身份摘要保存在 [证据目录](../../.adg/evidence/WIN-019-APPEARANCE-20261008/manifest.json)。本轮已提交的是玻璃／画质／光场／激活部分；原生窗口外观与菜单栏尚未开发，双权限矩阵另有当时的会话受阻记录，实施状态继续保留 `in_progress`，不宣称整个版本开发或验收完成。

## 已完成的源码与检查

代码本地提交：`8a8397a61eee676d6b5166bd67a43930eb5e8d44`（`feature/0.1.9-windows-appearance`）。随后按该提交重建 release EXE，未制作安装器；EXE 0.1.9，7,620,096 bytes，SHA-256=`24ea68223d9e4eecc64218ad6f5cf1b025c80bcdbbccbe83c49449015a504657`。最终原生检查的启动日志必须匹配此完整提交。

前端 81 文件 / 376 项通过，类型检查与生产构建通过；Windows 后端回归 337 项通过、4 项既有忽略，fmt 与 Clippy `-D warnings` 通过；loopback CDP 协议回归 7 项通过。便携 LLVM/MSVC SDK 链接时缺少外部调试 PDB 的 LNK4099 警告保留；release 链接成功。初次高并行调试链接的工具异常不作为应用启动崩溃，后续 `-j 2` 检查正常。

文档分支 README 中、英文此前落后于 main 的已实现网络与自更新说明；本轮按共享文件规则机械恢复为 main 的同一份内容，0.1.9 分支继承的 README 原已与 main 一致。AGENTS、LICENSE、.gitignore 保持一致；没有进行旧分支的全局改写。

## 最终 EXE 的原生结果与会话阻断

| 实际权限 / DPR | 首屏 / 画质 / 菜单 | 30 秒性能 | 最小化、隐藏恢复 / 重载持久化 |
|---|---|---|---|
| 普通 / 1 | 通过 | WebGL/CSS 通过 | 高／中档恢复、三档重载通过 |
| 普通 / 1.5 | 通过 | WebGL/CSS 通过 | 高／中档恢复、三档重载通过 |
| 普通 / 2 | 通过 | WebGL/CSS 通过 | 高／中档恢复、三档重载通过 |
| 管理员 / 1 | 通过 | WebGL/CSS 通过 | 高／中档恢复、三档重载通过 |
| 管理员 / 1.5 | 首屏、画质、菜单通过 | **失败：rAF 会话中断，稳定样本不足** | 未执行 |
| 管理员 / 2 | 未执行 | 未执行 | 未执行 |

普通实际 tokenElevated=false / elevationType=3；管理员 true / 2。所有已运行首屏的日志均匹配 `8a8397a…`，同一 EXE SHA-256。通过的完整四组，WebGL/CSS 高档 30 秒 rAF p95 为 17.3–18.8ms，绘制回调 p95 为 0.3–0.6ms，均满足 60Hz 的 2B／B/2，未见 >6B 长帧。系统 reduced-motion 两种模拟均不改三档；高档 SVG 5 个不同采样／连续 WebGL 绘制，中档 SVG 静态、尺寸恢复通常 2 次按需绘制，低档无 canvas。六种菜单主题／档位、键盘／选择／关闭和应用 75%／150%／200% zoom 均保留真实断言。

**阻断证据**：管理员 DPR 1.5 的 WebGL 样本仅 638 个 rAF，最大间隔 8049.1ms，p95=17.3ms、绘制回调 p95=0.3ms，无 LongTask；该段按失败保留，没有只看 p95 判通过。2026-10-08 17:39:09，Terminal Services 的 21／22／41／42 事件记录另一会话的登录／桌面启动，时间与采样中断重合。这是环境干扰证据，不能单凭它把所有长帧都定为外部原因。

随后相同 EXE 的管理员完整重跑在初始前台断言就失败。17:46:08 的独立核查确认工具运行于 session 2，状态 Disc；候选 visible=true、minimized=false，但 `GetForegroundWindow=0`，连续三次 `SetForegroundWindow` 均 false。此时另一次隔离首屏仍能通过 Vue/Tauri 命令和来源检查，区分启动可用与交互桌面不可用。所有本轮候选进程均退出，未切换／注销其他用户会话，也未用 CDP 假焦点或取消原生前台断言绕过。

**接续条件**：重新连接本机 `15119` 的 Windows 会话，确认交互桌面可用，再用同一提交／EXE、独立输出目录从管理员完整三档 DPR 矩阵重跑；保留旧失败记录，成功后再关闭本实施门禁。文档细节见 [维护分析 12](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261006-Windows启动崩溃与图标适配/分析/12-0.1.9玻璃与画质修复及Windows复验.md)。

物理显示器 DPI／迁移、Windows 10/11、macOS 原生以及安装器升级／卸载与发布仍属后续设备覆盖。本轮真实首屏为环境准备态，九态是装饰输入投影，未以此宣称真实代理九态生命周期验收。软边截图表明原椭圆硬裁剪已去除，不表示所有原型入口逐像素相等。

## 会话恢复后的接续复验

2026-10-08 18:53，本机 `15119` 的 session 2 已由外部重新连接；本轮没有操作用户会话。使用原 `8a8397a`、同一 EXE SHA-256 和未修改的 PowerShell 7 门禁，独立目录接续管理员矩阵。首轮 v3 的首屏／画质／菜单通过，但 CSS 30 秒样本出现 3 次 >6B 长帧（最大 204.2ms，无 LongTask、无前台重获），按失败保留；没有证据确定其根因，未将其静默归为会话问题。

随后串行 v4 **完整 DPR 1／1.5／2 均通过**，实际 tokenElevated=true / elevationType=2、PowerShell 7.6.5；启动日志均匹配源码提交。高档 WebGL 30 秒 p95 为 16.2–16.3ms；CSS 为 16.2–31.7ms，均满足既定 60Hz 的 2B=33.333ms；脚本回调 p95 为 0.3–0.5ms，未见 >6B 长帧。三档与媒体模拟、页面菜单像素／键盘／应用 zoom、高／中档最小化和隐藏恢复、偏好重载及首屏断言全部保留。日常 EXE／偏好摘要及 PID／启动时间未变，候选及 CDP 端口已清理。

接续证据入口：[result-summary.json](../../.adg/evidence/WIN-019-NATIVE-SCOPE-20261008/result-summary.json)／[manifest.json](../../.adg/evidence/WIN-019-NATIVE-SCOPE-20261008/manifest.json)。原 `.adg/evidence/WIN-019-APPEARANCE-20261008/` 保留当时的失败与受阻原件，不改历史通过口径。**既有画质矩阵的会话阻断已接续完成；原生窗口外观与菜单栏仍未实施，0.1.9 整体不关闭。** 本次接续没有改源码，也没有增加该原生专题的实现或验收声明。

## Mac 原专题同步（2026-10-08）

接续 docs/governance-main@fee8f4c8 后，原 [设计与实现交接](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261008-Windows窗口外观与菜单适配/分析/01-设计与实现交接.md)、来源截图与浏览器材料已补齐，Windows 右侧专题入口已接入跟踪的共享原型。此前“未取得原文”是 Windows 当时的信息缺口，现已解除；代码仍为 8a8397a，本次未改应用代码。

后续实施单行标题栏、Win11 原生 Mica／实色回退、移除 Windows 原生菜单行及无损功能入口、关闭偏好与 macOS 保留路径。按专题完整门禁采集新提交／制品的窗口、Snap、系统菜单、物理 DPI、多屏、材质、双权限和关闭生命周期证据，再接续 macOS 原生与安装器回归。既有 DPR 矩阵及浏览器 mock 不能替代上述验收。**仅移除材料同步阻塞，IMP-19 继续 in_progress，0.1.9 整体未完成。**

## fb4fe172 后的原生实施

用户授权接续后，在原 `feature/0.1.9-windows-appearance` 实施 Windows 专属原生适配。实现 `d4d7164`、原生测试 `965c314 / 94f3f29`、独立托盘重载补充 `2ec3812`、候选出包门禁 `15219bf` 均为本地提交，最终源码 **`15219bf08ceaaef86ae471772d27e04fdbf5ff1d`**。先前“原生尚未实施”保留材料交接时的历史口径，当前实施／测试／未通过项以 [原生回执 03](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261008-Windows窗口外观与菜单适配/分析/03-原生实施与本机门禁回执.md) 为准。

采用系统 caption 和按钮，保留 User32 拖拽／缩放／系统菜单／Snap 命中及 CloseRequested，只移除 Windows 原生应用菜单行；非 Windows 原菜单不变。Mica 用 DWM 真实属性与回读，主题／透明可用性／非活动触发原生更新和实色回退，不改用户三档、不读取系统减少动态。移除菜单后的独立原生重载入口迁到 Windows 托盘，继续不依赖页面。精确原生 DPI 与 WebView DPR 分开复验，新增 125% WebView 模拟入口。

按新源码重建的 EXE 0.1.9，7,625,728 bytes，SHA-256=`b48c30667116b59aab5cefdb2c9617456d073427cde54ac2ba425e9109252992`。前端 378、Windows 后端 integration-test 338（5 项既有忽略）、CDP／制品身份 12 项通过，类型／构建／fmt／Clippy 通过。发布工具全套在本机有 Linux／minisign 环境错误，未判通过；远端 CI、Win10/11 物理 DPI／多屏、macOS 与新制品安装往返仍需适用环境接续，本轮不推送或发布。

固定新 EXE 的普通／管理员四档 WebView DPR 共 8 组画质矩阵全部通过，p95 最高31.9ms／回调p95最高0.5ms；管理员首轮125.8ms长帧的失败原件与未确定原因保留。原生真实DPI=192／DPR=2，各权限14项中13项通过，Snap独立自动识别失败；最终悬停截图可见系统布局，但Win+Z图未显示，不判完整Snap通过。MSAA证据不等于人工读屏，shell PID保留／API重开不等于真实代理／托盘入口生命周期通过。

最终证据见 [result-summary.json](../../.adg/evidence/WIN-019-NATIVE-IMPLEMENTATION-20261008/result-summary.json)／[gate-matrix.json](../../.adg/evidence/WIN-019-NATIVE-IMPLEMENTATION-20261008/gate-matrix.json)／[manifest.json](../../.adg/evidence/WIN-019-NATIVE-IMPLEMENTATION-20261008/manifest.json)。完整设计门禁在回执03逐项保留，未完成项包括Snap、目标系统／物理多档DPI／多屏／人工读屏／真实代理与托盘往返／macOS／完整CI／新安装往返；**实施继续in_progress，不能进入全部验收完成或发布完成状态**。候选清理、日常EXE与偏好哈希／原进程身份／系统透明恢复已核对。

## 候选安装版本门禁接续（2026-10-08）

原生实现及上述本机回执写入后，代码 `15219bf0` 已推送。对应 [CI #41](https://github.com/gzers/opencodex-desktop/actions/runs/37782268133) 全部适用检查通过；特性分支的 main 专属 build job 按规则跳过。[Windows candidate installer #5](https://github.com/gzers/opencodex-desktop/actions/runs/37782267972) 的 NSIS／MSI 构建成功，但安装验收脚本仍写死 `^0.1.8`，在已安装 MSI 的 `ProductVersion=0.1.9` 检查处报 `Unexpected version 0.1.9`。候选安装包上传被跳过，不能将构建成功记为安装验收通过。上文“不推送／远端未执行”仅保留对应轮次的历史口径。

用户同意接续修复后，在 `feature/0.1.9-windows-appearance` 提交并推送 `bc46da14be245364fe436a8b845f03a1da133d50`：从现有 `tauri.conf.json` 读取期望版本，版本缺失失败，安装后的 EXE `ProductVersion` 必须与期望值完全一致。制品逐字节身份检查、隔离 runner 限制、源码提交一致性、首屏与卸载检查均保留。

随后该提交的 CI 前端步骤发现 CDP HTTP 503 回归的时序依赖：最后一次请求在发现截止时间前超时，最终错误是 timeout，而测试固定要求最终错误为 503。`70711ced11e64d5ced3323e2a1815f31d9263121` 单独修正测试：发现流程仍必须失败，诊断历史中必须有实际 503 状态和 HTTP 503 错误；该错误语义测试使用独立请求预算。生产发现超时与独立 stalled-response 时限断言未变，不忽略失败、不放宽安装验收。相关 Node 协议／制品身份 12 项通过，CDP 七项另连续三轮通过。

`70711ced` 的 [候选安装器](https://github.com/gzers/opencodex-desktop/actions/runs/37787195944) 已通过：MSI／NSIS 均为 0.1.9，安装后的 EXE 包标记及逐字节身份一致，首屏通过，卸载返回 0 且 EXE 已移除；制品与证据已上传。其 [CI](https://github.com/gzers/opencodex-desktop/actions/runs/37787195941) 前端、发布工具和 Windows 后端通过，macOS 后端因 TLS 行为测试失败，整体未通过。

macOS 的 TLS 行为测试使用非阻塞 listener，accepted stream 继承该模式；握手首字节未到达时 read_exact 返回 WouldBlock，原代码将其转为 None。已用本机延迟 100ms 写入 0x16 的独立 loopback 对照复现：未重置时 WouldBlock，显式恢复阻塞后成功读到 0x16。`0f4d0d5e3d5401b8d7a3d9859fb19fe2925d52d8` 在测试端点接受连接后设置阻塞读取，保留原 1 秒读取超时、3 秒接受截止及 0x16 断言；本地 TLS 两项、fmt 与对应 Clippy 通过。该提交的远端 macOS 后端已通过，但 [CI](https://github.com/gzers/opencodex-desktop/actions/runs/37795778611) 又在 CDP JSON 语义测试失败：150ms 发现预算耗尽前仅捕获 version 请求超时，没有进入 list 解析，不能算取得解析错误证据。

`689a86e6261c9f48ea6c4f89f0991c6267735540` 将协议语义测试与短时限测试的预算分开；语义用例仍必须失败且保留真实 HTTP 503、200＋JSON 解析错误或 ECONNREFUSED，成功发现与重试仍严格断言。stalled-response 的 150ms／30ms 预算及总时限断言、生产发现参数均不变。Node CDP 七项与制品身份五项合计 12 项通过。

固定最新提交的 [CI](https://github.com/gzers/opencodex-desktop/actions/runs/37796440853) 与 [候选安装器](https://github.com/gzers/opencodex-desktop/actions/runs/37796440954) **均已通过**。前端类型／单测／构建、CDP、发布工具、macOS 与 Windows 后端格式／Clippy／回归、Windows 最终 EXE 首屏全部通过；main 专属 build job 按特性分支规则跳过。MSI／NSIS 候选安装包及证据已上传，仍为未签名、未接受正式发布的测试候选。旧失败、复现、最终 run 身份与安装后的制品校验保留在 [.adg 证据目录](../../.adg/evidence/WIN-019-INSTALLER-VERSION-20261008/manifest.json)。

本次代码仅调整候选安装验收、CDP协议回归与TLS行为测试，未合并 main、正式发布或替换日常安装。安装器 runner 的首次安装／首屏／卸载不能替代用户升级、Snap、物理 DPI／多屏、真实代理／托盘生命周期及 macOS 原生／安装验收，IMP-19 继续 `in_progress`。

## 用户试用确认与发布接续（2026-10-09）

用户明确表示“我看了下0.1.9基本没有问题了。可以发布。”，据此快进合入 main，补写 main 专属 CHANGELOG，固定 `v0.1.9@de6862d40389d527527b9db8ef52e186a58b625f` 并执行既有双平台发布和稳定通道校验。发布身份、验签、公开状态和回退结论统一记录于 [REL-09](REL-OPENCODEX-DESKTOP-09.md)。此前“不发布”的描述是各历史执行轮次范围，本次发布由新授权接续。

用户试用确认与发布完成不将未执行专项变为通过：Snap、物理 DPI／多屏、目标系统、人工读屏、真实代理／托盘往返、macOS 原生 UI／安装和旧客户端升级专项继续保留原证据与待补测状态。IMP-19 不据此改成全部验收完成；本次未替换本机日常 0.1.7。
