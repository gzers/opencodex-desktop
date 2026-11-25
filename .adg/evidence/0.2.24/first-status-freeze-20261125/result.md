# 0.2.24 离线运行环境与首次状态候选

2026-11-25。源码来自已推送的 main `16990bb960d086525390fd068b6c13e474d406da`。候选构建的 head SHA 与该提交一致，Windows x64 和 macOS ARM64 构建成功。整版 accepted:false，Windows 真实隔离安装尚待回交。GitHub 未创建 v0.2.24 tag 或 Release。[源码](https://github.com/gzers/opencodex-desktop/commit/16990bb960d086525390fd068b6c13e474d406da) · [构建](https://github.com/gzers/opencodex-desktop/actions/runs/45616454620)

概览自动离线解包、校验随包环境，显示准备进度与「无需联网」。环境缺失可以稍后处理并重开入口，下载由用户主动触发。首次 Status 在 5 秒内无法确认时，显示「运行状态未知」，可重试或诊断，确认空闲前禁用 Start。面板查询另有独立 8 秒上限。

Vitest 911 项、界面构建、格式、严格 Clippy、Rust 定向 91 项及 13 项集成测试通过。Rust 全量首轮为 2,171 项通过、6 项失败、29 项忽略。六项后继隔离复跑通过，其中安装保护测试在 /private/tmp 仍被判定为 Target(SystemProtected)，改用工作树忽略的 target 下隔离 HOME 与 TMPDIR 后通过。首轮失败及复跑日志留存，测试夹具已清理。

| 制品 | SHA-256 |
| --- | --- |
| OpenCodeX Desktop_0.2.24_x64_en-US.msi | `1841428c34d878f650d255fbf7783c3a7bee964f32d1e642d7cbb3962e4fa8e4` |
| OpenCodeX Desktop_0.2.24_x64-setup.exe | `4853fa6a9043952c21b511ce1bf7f9cd9c782209436428fb0816cf58893774de` |
| OpenCodeX Desktop_0.2.24_aarch64.dmg | `c79bd918d82833e436aa57efe17fac049298a0cc1c864c36f28d028ce5040093` |
| OpenCodeX Desktop.app.tar.gz | `43decd815ebd7a947419ff66aaecaf20c6f32ae48540c6fe5d1296710e760491` |

独立复算四包及三个签名文件摘要，并以同提交配置的公钥验证三份 Minisign 更新签名。Windows 包内容检查报告与候选安装器字节吻合，MSI 和 NSIS 的 Authenticode 均为 NotSigned。Mac 单 .app 布局通过，ad hoc 代码签名有效，缺 Developer ID 和公证证据。本轮的构建平台证据为离线制品检查，尚无同候选的原生安装运行结果。[机器回执](verification.json) · [验签输出](verification.log)

[Windows 候选](https://github.com/gzers/opencodex-desktop/actions/runs/45616454620/artifacts/15022804957) · [Windows 安装器证据](https://github.com/gzers/opencodex-desktop/actions/runs/45616454620/artifacts/15024261290) · [Mac 候选](https://github.com/gzers/opencodex-desktop/actions/runs/45616454620/artifacts/15022743577) · [Mac 布局证据](https://github.com/gzers/opencodex-desktop/actions/runs/45616454620/artifacts/15022848264)

[TEAM80 单用户隔离验证](../../../../docs/03-开发实施/TEAM-0.2.24.md#team80-windows-单用户隔离验证)为当前交接。旧候选与篡改回归证据保留在 [前轮记录](../release-freeze-20261124/result.md)。W2、双 SID、真实业务同步及公开发布均未执行。
