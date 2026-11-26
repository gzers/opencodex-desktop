# Windows W2 安装环境只读预检

2026-11-26，按 TEAM83／IMP229。复核时 main 为 `a69ae55a3fe21b58f415da727b0d1bdff8f37568`，Windows 原生提交为 `a288fc41457ffa682edb3b3df2e00500a61a5649`。尚无新安装器，也未进入 W2。此预检只确认现有账户是否具备独立安装的登记基线。

待证伪判断：当前控制台与现有普通账户可以提供可核验且不占用日常安装或历史失败现场的产品登记。若只看得到键、无法核实内容，或登记属于既有安装，该账户暂不能作为干净安装的证明。

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| Administrator、marenben 产品键 | 两者均可见。只证明存在性。 | [PowerShell Registry provider 原始回执](raw/user-product-key-visibility.json) |
| 既有产品和卸载登记的内容 | 未知。当前令牌读取两个用户的 32／64 位视图均被拒绝。首错为 `Requested registry access is not allowed.` | [逐项原始回执](raw/registry-readback.json) |
| 普通用户 per-user MSI 准入 | 待实测。当前 32／64 位机器视图均未见 Installer 策略键。历史 TEAM79 的实际 MSI 曾返回 1625，客户端和服务报告有效 `DisableMsi=1`。 | [当前策略读回](raw/registry-readback.json) · [历史 MSI 首错](../team79-20261124-r1/sweep/03-msi-default.json) |
| 同时 Active 的真实桌面 | 本轮未齐。Administrator 控制台与 15119 RDP 为 Active，marenben 为 Disc。 | [会话原始回执](raw/sessions.txt) |

TEAM79 曾记录 Administrator 的日常 0.2.17 NSIS 登记，以及保留在 marenben 下的自定义 NSIS 失败现场。现有产品键仍可见，但本次无法判定各键当前关联的版本和安装位置。当前 NSIS 模板用固定的产品键和卸载键写入安装路径，因此自定义文件目录本身无法证明登记独立。冻结后，应从各测试用户自身会话核对产品及卸载登记；若与日常安装或失败原件共用，应在干净的隔离 Windows 环境验证，并先实测普通用户 MSI 准入。 [历史双用户安装回执](../team79-20261124-r1/sweep/08-dual-user-private-install.json) · [NSIS 源码](https://github.com/gzers/opencodex-desktop/blob/a69ae55a3fe21b58f415da727b0d1bdff8f37568/apps/desktop/tauri/windows/nsis023.nsi#L733)

本轮未启动候选，未变更账户、策略、PATH 或安装登记，无本轮测试残留。历史失败原件和 marenben 的账户及 Profile 保留。TEAM81 仍待新候选及可独立断网的桌面，W2 待同源冻结。`accepted:false`。
