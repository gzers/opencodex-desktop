# 0.2.24 首次运行环境供应修复与 Windows 候选

Settings 首次安装误走替换流程并提前调用 `selected_program()`。生产修复提交为 `fcddce49bd916244cc037378e6a1705c6755cdf4`。新路径使用首次供应流程，依次核验随包清单与载荷。完整包可本地准备，缺包只有当前尝试明确确认后才请求。延期、旧确认失效、描述符无效和本地归档损坏均不触发下载。供应完成后刷新环境，并进入首次 Status。以下定向测试在包含该修复的历史主线 `7580b0d79289864cb105e6d56d668b608b93495b` 执行。整版 `accepted:false`。

| 前置验证 | 结果 | 证据 |
| --- | --- | --- |
| 严格全工作区 Clippy | 同源 macOS 本机通过。托管 CI 状态见下方回执 | [本机输出](strict-clippy.log) |
| 实际完整包离线供应 | 1 项通过，官方 2.99.0 归档由实际 worker 准备，耗时 66.20 秒。配置保持、安装租约释放、服务未启动 | [输出](complete-offline.log) |
| 受控缺包供应 | 7 项通过，覆盖确认与延期、失效确认、校验拒绝、下载后供应与临时文件清理 | [输出](confirmed-download.log) |
| 上一源码的相关界面和构建工具 | UI 33 项、发布校验 36 项、updater 10 项通过。最终同源 CI 见下方回执 | [历史 UI 输出](ui.log) · [历史发布工具输出](release-validation.log) |

三项定向检查用于前置定位。候选构建与复用要求推送后的同一 main SHA 的 backend 和 backend-windows 均通过。受控下载使用测试载荷和传输夹具。正式公开下载尚未供应，Windows 原生桌面须单独复验首次供应与 Status。

| 原失败步骤 | 首条错误 | 责任与后继 |
| --- | --- | --- |
| Windows 隔离缺包，Settings 确认安装 | `managed_dependency_missing` | Mac 修共享首次供应和 UI。Windows 用新候选定向复验。[原回执](https://github.com/gzers/opencodex-desktop/blob/02be801131e028df45f3c4a2034e92ac41d22c27/.adg/evidence/0.2.24/windows-final/team81-20261125-r1/result.md) |
| 旧 CI 的 macOS、Windows 严格 Clippy | `use of deprecated method std::sync::atomic::Atomic::<u64>::fetch_update: renamed to try_update for consistency` | Mac 修共享测试代码。同源 Windows CI 和 macOS 本机严格 Clippy 复核通过。[旧 CI](https://github.com/gzers/opencodex-desktop/actions/runs/45774644978) |
| 旧 CI 的发布工具 tar 夹具 | `tarfile.ReadError: empty header` | Mac 修校验流程，后继 CI 通过。[旧 CI](https://github.com/gzers/opencodex-desktop/actions/runs/45774644978) |
| 旧 CI 的生产 ESM 启动测试 | `AssertionError [ERR_ASSERTION]: Expected values to be strictly deep-equal:` | Mac 更新共享启动断言，同源 macOS 本机生产启动验证通过。[旧 CI](https://github.com/gzers/opencodex-desktop/actions/runs/45774644978) |

上一源码的后台回归在同一批排查中确认 15 项测试夹具失败。Mac／共享测试维护负责，集中修正在 `7580b0d79289864cb105e6d56d668b608b93495b`。生产断言、超时预算和未知拒绝保持原样。

| 平台与步骤 | 项数 | 首条错误或断言差异 |
| --- | ---: | --- |
| Windows 普通账户后台回归 · 运行归属 | 4 | `assertion failed: matches!(f.query(), ServiceOwnerProof::Owned { pid: 4242, .. })` |
| Windows 普通账户后台回归 · 同 SID 子进程互斥 | 1 | `left: (0, 0)`，预期 `(1, 1)` |
| Windows 普通账户后台回归 · 路径夹具 | 1 | 结果含 verbatim 路径前缀，夹具仍比较未规范化路径 |
| Windows 普通账户后台回归 · 扩展路径夹具 | 3 | Codex 写入路径的相等断言失败，夹具使用 Windows 上的非绝对路径 |
| macOS 后台回归 · 改名可执行程序扫描 | 1 | 进程扫描未发现夹具 PID 和路径 |
| macOS 后台回归 · TCP 子进程夹具 | 3 | `owned fixture response missing` |
| macOS 后台回归 · 路径夹具 | 1 | 结果含 `/private/var`，夹具预期 `/var` |
| macOS 后台回归 · 备份位置夹具 | 1 | `Target(SystemProtected)` |

旧失败保留在 [macOS 原任务](https://github.com/gzers/opencodex-desktop/actions/runs/45854374009/job/140028783078)与 [Windows 普通账户原证据](https://github.com/gzers/opencodex-desktop/actions/runs/45854374009/artifacts/15138510777)。旧源码 `6864ce65e40ee0f204b58e707b7182f2c6d7431d` 的 [诊断构建](https://github.com/gzers/opencodex-desktop/actions/runs/45858594804)独立保留。

## 旧候选失败与 Windows 回执

| CI | 步骤与结果 | 责任与后继 |
| --- | --- | --- |
| [45862219344](https://github.com/gzers/opencodex-desktop/actions/runs/45862219344) | Windows 最终 EXE 冒烟首错 `Sandbox startup log missing; see sandbox-file-inventory.json`。对应 macOS 作业被取消，原因未证实。 | Mac／共享测试维护修正 0.2.24 沙箱日志根。macOS 同源本机补验保留，后继主线仍须双后端 CI。 |
| [45917490865](https://github.com/gzers/opencodex-desktop/actions/runs/45917490865)、[45942760452](https://github.com/gzers/opencodex-desktop/actions/runs/45942760452) | Windows 安装恢复步骤首错 `Unsupported native model UI acceptance version`。后一轮已通过普通账户回归、TokenUser、原生凭据、受管升级与离线重装、MSVC EXE 构建和最终 EXE 冒烟。 | Mac／共享测试维护修正裸 EXE 驱动，单列缺固定依赖拒绝和基线保全。Windows 在真实桌面完成已供应候选的失败恢复复验。[Windows 回执与原始首错](https://github.com/gzers/opencodex-desktop/blob/070cb2dc62809faa316692322fc5ac7e467dd5df/.adg/evidence/0.2.24/windows-final/team81-20261127-ci-45942760452-r1/result.md#windows-作业终态) |
| [45946693215](https://github.com/gzers/opencodex-desktop/actions/runs/45946693215) | Windows 普通账户后端、TokenUser、原生凭据、受管升级和离线重装、MSVC EXE 及冒烟通过。旧驱动仍报 `Unsupported native model UI acceptance version`。 | Mac／共享测试维护于 `6aa38bde9` 更新驱动。[Windows 回执](https://github.com/gzers/opencodex-desktop/commit/e547a3487a3eeb4cdd19da972dc3d99ed532674c) |
| [45949909762](https://github.com/gzers/opencodex-desktop/actions/runs/45949909762) | Windows 步骤 10「Windows backend regression」失败。首错 `windows_cli024.rs:563 assertion failed: f.key().read().is_err()`，50／51 个普通账户测试二进制通过。严格 Clippy、TokenUser、原生凭据及证据上传通过。受管升级和离线重装、MSVC EXE 和后续冒烟跳过。 | Windows 独立探针证实 Win32 写入未终止字符串时可能按尾部缓冲区补齐，旧日志未记录具体输入。Mac 于 `37a141fbee8e357db3d42f2aa830c6be2c8be195` 改用原生接口向私有测试键写入精确畸形字节，增加用例编号和拒绝后逐字节比对。[Windows 原始回执](https://github.com/gzers/opencodex-desktop/blob/e7d1c79d37ef8149ac000c0026945966a035e42c/.adg/evidence/0.2.24/windows-final/team81-20261127-6aa38bde-r1/result.md#主线-windows-作业终态) · [原始 Actions 制品](https://github.com/gzers/opencodex-desktop/actions/runs/45949909762/artifacts/15178672325) |
| [45956890861](https://github.com/gzers/opencodex-desktop/actions/runs/45956890861) | Windows 功能分支普通账户回归首错 `credentials.rs:600:47` 的 `store a: NotConfigured`。库测试 1801 通过、1 失败、19 忽略，单独凭据步骤与清理通过。 | 共享测试维护负责归因。同样凭据源码的主线 [45956736730](https://github.com/gzers/opencodex-desktop/actions/runs/45956736730) 已通过普通账户回归。共享根竞态尚未证实，若复现按目录、DPAPI 和原子写入阶段定位。[原始回执](https://github.com/gzers/opencodex-desktop/blob/a815b338bf2862dc3a3fb7c85fa7e90c96a2d16e/.adg/evidence/0.2.24/windows-final/team83-20261127-ci-45956890861-r1/result.md) |

Windows 的 Status 与 Stop 截止时间修正 `7889ecc35`、`97f100b32` 均已进入以上后继主线。Windows [回执](https://github.com/gzers/opencodex-desktop/blob/e7d1c79d37ef8149ac000c0026945966a035e42c/.adg/evidence/0.2.24/windows-final/team81-20261127-6aa38bde-r1/result.md)等待同源安装器。真实普通账户 Stop 与已供应固定 2.99 的安装失败恢复尚未验收。断网桌面按用户决定跳过，原回执保留。

Windows 已独立复算本轮冒烟制品 `15176338274` 的 ZIP 摘要，11,484 字节，SHA-256 为 `049ca349066a13b509d6c922e2f38b300d141bf8900e4182edf96c584e49c2f9`。冒烟证据覆盖代际不符拒绝、子进程退出、临时普通账户与 Profile 清理。该轮未启动产品。见 [Actions 冒烟制品](https://github.com/gzers/opencodex-desktop/actions/runs/45949909762/artifacts/15176338274)。

Windows 本机用 MSVC 验证主线夹具修复：8 项定向测试通过，测试键已清理。Windows 的并行修复单独保留，主线采用已集成的精确字节夹具。[Windows 回执与定向日志](https://github.com/gzers/opencodex-desktop/blob/f36f838943825eefc9cd59ae21f00ed204d13e8c/.adg/evidence/0.2.24/windows-final/team81-20261127-6aa38bde-r1/result.md)

## 接收 Windows 原生启动修复

| 失效步骤 | 首错或已确认现象 | 责任与核验 |
| --- | --- | --- |
| Start 第二次完整镜像扫描、创建与恢复挂起进程 | 源码审查发现第二次完整扫描未传原 Start 截止时间，创建和 Resume 前缺少余量检查。没有真实生产超时采样。 | Windows 修复，Mac 复核并集成至 `c42361f28943d907211b5cc996d5732d91975727`。沿用 30 秒预算，Resume 前超时保留已登记 Owner 与 Job。Windows MSVC 组件证据为 4 项生命周期与 2 项镜像时限通过。[原始回执](https://github.com/gzers/opencodex-desktop/blob/1165748bff71d0530034eb28482a091e0bb16d61/.adg/evidence/0.2.24/windows-final/team83-20261127-start-deadline-r1/result.md) |
| 无保留 Owner，隔离 HOME 只有 `ocx.pid` | 旧定向夹具进入后续计划，首错 `FileSystem { operation: "validate process executable", … }`。 | Windows 修复，Mac 复核并集成至 `a75d3b0af3e131e10b7ee8fcc2a7687378c7482a`。任一运行记录存在或读取结果不明确均拒绝。MSVC 定向 5 通过、4 个真实系统场景忽略，原 PID 记录字节保留，未创建输出目录。记录都缺失不足以证明空闲。[原始回执](https://github.com/gzers/opencodex-desktop/blob/7216f3ebb76a63d40f2637bd82d1484a9078b2eb/.adg/evidence/0.2.24/windows-final/team83-20261127-start-records-r1/result.md) |
| 桌面 Start 入口在 Owner 未确认时写入自动端口 | 仅有 `ocx.pid` 时服务已拒绝，回归首错 `Start wrote to the data root`。 | Windows 修复，Mac 复核并集成至 `3a844aef364830604da49268d3c470a318ecb3fe`。在 Owner 锁内先检查运行记录，再读写端口设置。MSVC 归属定向 5 项通过、4 项实机用例忽略，端口定向 3 项通过。[原始回执](https://github.com/gzers/opencodex-desktop/blob/db13544d8a5b1b700b768d85a26419dff80c4711/.adg/evidence/0.2.24/windows-final/team83-20261127-start-port-order-r1/result.md) |

| Start 端口写锁争用超过剩余预算 | 源码审查确认端口写锁使用独立的最长 3 秒等待，可能超过启动剩余预算。 | Mac 修共享代码至 `e663369226337c9a9cad594ccd3cafd1719c3711`。macOS 本机格式、严格 Clippy、端口 4 项、原子写 3 项、文件锁 3 项通过。Windows 原探针因 `blocked by policy` 未创建夹具。候选定向验锁争用拒绝、原记录保持及无新服务启动。[Windows 原始审查](https://github.com/gzers/opencodex-desktop/blob/57f62feaddaa1cf8f282800450ebcdbd037b2573/.adg/evidence/0.2.24/windows-final/team83-20261127-port-deadline-review-r1/result.md) |

Windows 另独立复算旧主线 `3a844aef3` 的 CI 冒烟 ZIP，制品 15181926704，11,481 字节。进程令牌和身份、重启夹具、同一临时 SID 的账户和 Profile（记录、注册表及目录）清理均通过。该轮未启动产品。[冒烟制品](https://github.com/gzers/opencodex-desktop/actions/runs/45963586935/artifacts/15181926704) · [原始回执](https://github.com/gzers/opencodex-desktop/blob/57f62feaddaa1cf8f282800450ebcdbd037b2573/.adg/evidence/0.2.24/windows-final/team83-20261127-ci-45963586935-token-r1/result.md)

Windows 补交的 CI 原件各自绑定相应源码。

- 较早主线普通账户回归已通过同名凭据用例，旧首错归因仍未确认。 [Windows 原件](https://github.com/gzers/opencodex-desktop/blob/77e103afdb2f3d2347ae3ed3532ad9d5a9fef55a/.adg/evidence/0.2.24/windows-final/team83-20261127-ci-45956736730-r1/result.md)
- 较早主线的受管升级及离线重装覆盖官方 2.90.0／2.91.0。固定 2.99.0 安装失败恢复仍待实机验证。 [Windows 原件](https://github.com/gzers/opencodex-desktop/blob/77e103afdb2f3d2347ae3ed3532ad9d5a9fef55a/.adg/evidence/0.2.24/windows-final/team83-20261127-ci-45960532202-r1/result.md)
- 当前源码 e66336922 的独立冒烟通过令牌、进程身份、重启交接和临时普通账户及 Profile 清理。 [Windows 原件](https://github.com/gzers/opencodex-desktop/blob/77e103afdb2f3d2347ae3ed3532ad9d5a9fef55a/.adg/evidence/0.2.24/windows-final/team83-20261127-ci-45965743318-token-r1/result.md)
- 最终源码的 Windows CI 作业 140393871165 通过两项端口写锁截止时间夹具。750 毫秒夹具返回 Timeout，未发布设置，HOME 为空且无 Owner。250 毫秒夹具保全原字节，释放锁后可重试。普通账户库回归 1806 通过、19 忽略，旧凭据用例通过，旧首错归因仍未确认。裸 EXE 冒烟只证明窗口响应与清理，环境门禁仍为 `integrated_runtime_not_ready`，准备状态为 `idle`，首次 Status 与真实桌面启停仍待复验。受管升级及离线重装使用 2.90.0／2.91.0，固定 2.99.0 的真实桌面安装失败恢复仍待验证。[Windows 原件](https://github.com/gzers/opencodex-desktop/blob/e6d0617bf882c457fb960770bd26d7e661841ea0/.adg/evidence/0.2.24/windows-final/team83-20261127-ci-45965743318-r1/result.md)

候选绑定最终主线。旧源码 CI 45960064096、45960532202、45963586935 保留原运行及状态，已由后继主线替代。组件证据支持代码接收，真实普通账户桌面复验仍待回交。

## 同源候选

源码 `main@e663369226337c9a9cad594ccd3cafd1719c3711`。同源 `backend` 与 `backend-windows` 均通过。[CI 45965743318](https://github.com/gzers/opencodex-desktop/actions/runs/45965743318)

定向复验使用同源候选的安装器。已核对 Actions ZIP 摘要、安装器字节摘要、更新签名和实际内嵌的官方 OpenCodex 2.99.0 运行归档。[构建 45973489918](https://github.com/gzers/opencodex-desktop/actions/runs/45973489918) · [Windows x64 安装器 15187611477](https://github.com/gzers/opencodex-desktop/actions/runs/45973489918/artifacts/15187611477) · [Windows 运行归档 15187765798](https://github.com/gzers/opencodex-desktop/actions/runs/45973489918/artifacts/15187765798) · [Windows 内嵌内容核验 15187547131](https://github.com/gzers/opencodex-desktop/actions/runs/45973489918/artifacts/15187547131)

| 安装器 | 文件 | SHA-256 | 更新签名（Minisign） | Authenticode |
| --- | --- | --- | --- | --- |
| MSI | `OpenCodeX Desktop_0.2.24_x64_en-US.msi` | `bbe3880bb1779ce90641cdba40b756c9516c777733b458c4ccf2c2b55f099bab` | 已通过 | `NotSigned` |
| NSIS | `OpenCodeX Desktop_0.2.24_x64-setup.exe` | `45cb6d27d20c1a60f1167b9e4641ed5e862bc39398f0c90a7cf9fa830c0467aa` | 已通过 | `NotSigned` |

独立运行归档 `runtime-bundle-0.2.24-win32-x64.tar.gz` 的 SHA-256 为 `14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7`，与两个安装器内嵌归档一致。

本机独立核对 macOS 候选文件摘要、DMG 布局及更新签名。临时代码签名通过完整性校验。候选缺少 Developer ID 签名，公证装订未核验。[macOS arm64 候选 15186409765](https://github.com/gzers/opencodex-desktop/actions/runs/45973489918/artifacts/15186409765)

| 制品用途 | 文件 | SHA-256 |
| --- | --- | --- |
| dmg | `OpenCodeX Desktop_0.2.24_aarch64.dmg` | `4c0c845d5b7efb5980a7235557198c2d5d4a32651042844abb57a05b24e7c032` |
| updater | `OpenCodeX Desktop.app.tar.gz` | `d4697295a67aea93ea484183d7b5c4a219a780303cc6fae16d6535dfb66a856e` |

[同源 CI 回执](ci.latest.json) · [构建回执](run.json) · [Actions 制品索引](artifact-index.latest.json) · [Windows 核验](verification.json) · [macOS 核验](macos-verification.json)

Windows 真实单用户原生复验仍待回交，完整验收保持 `accepted:false`。
