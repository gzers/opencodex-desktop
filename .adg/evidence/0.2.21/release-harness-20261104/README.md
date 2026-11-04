# 0.2.21 Windows 验收工具版本适配

2026-11-04，IMP-OPENCODEX-DESKTOP-29 的 G4/G5 准备。代码根为 `/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes`，分支 `codex/0.2.21-fixes`。这是工具适配与本机回归证据，Windows 原生安装、真实 OTA 和版本发布尚未执行。

修改限于以下五个已有文件：

- `test/smoke/windows-release-installers.ps1`：默认目标改为 0.2.21；明确接纳 0.2.17～0.2.21，继续拒绝未知未来版本、其它版本族和非规范版本文本。
- `test/smoke/windows-ota.ps1`：默认目标/基线改为 0.2.21/0.2.20；仅新增真实相邻方向 0.2.20→0.2.21，保留原四组相邻升级。未知未来、降级、同版及跨版跳跃仍被拒绝。
- `test/smoke/windows-model-ui.mjs`：0.2.20 与 0.2.21 均必须执行既有原生键盘/菜单断言；历史基线保留原行为。未知原生版本在任何 IPC 前拒绝。
- `test/test_windows_release_acceptance.py`：扩展离线白名单、显式 OTA 配对、默认值及副作用前拒绝的断言，覆盖 225 组目标/基线组合。
- `test/smoke/windows-model-ui.test.mjs`：验证本版键盘检查必须启用、旧基线行为和未知版本在 IPC 前拒绝，并验证 020/021 会进入 CDP 驱动。

测试先验证旧工具失败：9 项 Python 回归中，新增的 021 白名单、相邻升级和默认值三项失败。适配后 Python 9/9、Node 26/26 通过，0 跳过；原始失败及后续通过日志分别保留。Node 覆盖模型沙箱/真实临时文件与备份、包体字节身份、OTA 通道/基线以及 loopback CDP 边界。

可复现命令在代码根执行：

```bash
python3 -B -m unittest discover -s test -p 'test_windows_release_acceptance.py' -v
node --test test/smoke/windows-model-ui.test.mjs test/smoke/windows-bundle-identity.test.mjs test/smoke/windows-ota-channel.test.mjs test/smoke/windows-ota-fixture.test.mjs test/smoke/windows-cdp.test.mjs
git diff --check -- test/smoke/windows-release-installers.ps1 test/test_windows_release_acceptance.py test/smoke/windows-ota.ps1 test/smoke/windows-model-ui.mjs test/smoke/windows-model-ui.test.mjs
```

本机 Python 3.11.15、Node v26.8.1；没有 PowerShell。PowerShell 白名单和配对条件由离线测试按完整受限表达式核验，没有实际执行 PowerShell 或 Windows 安装器。Node 中的 CDP 准入测试使用哨兵调用，不构成真实 WebView2 或键盘操作通过证据。

版本参数和准入判断之外，两份 PowerShell 脚本的执行正文、原生键盘/菜单检查正文均与修改前相同。Mandatory SHA-256、唯一包体标记、完整安装 EXE 字节身份、基线资产摘要、原生进程/保存/备份/重启等门禁保持；摘要对比与范围核对见 `source-before.json` / `source-after.json`。

所有工作流、发布器及既有字节身份工具保持。`windows-release-acceptance.yml` 已支持字符串版本参数，无需修改；其网页默认值仍为 0.2.20/0.2.19，后续 021 执行必须显式传入 `version=0.2.21`、`from_version=0.2.20` 及新 Release 的 `built_sha256`。

`summary.json` 为本轮结果，`green-*.json` 记录真实命令与退出码，`*.log` 保留原始输出，`SHA256SUMS` 固定本目录证据。未执行 Git 暂存/提交/推送、Actions、发布、真实外部更新下载或日常配置访问；原工作区其它实现改动不在本轮写集。
