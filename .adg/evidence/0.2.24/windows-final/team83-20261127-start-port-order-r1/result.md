# TEAM83 Windows Start 拒绝时保留端口设置

待证伪判断（2026-11-27）：桌面 Start 在仅有 `ocx.pid`、无保留 Owner 时会返回 `windows024_owner_unknown`，且不会写入数据根或改动原记录。

Windows 源码 `b490f29e38654457bd0466072f526369fc3ccdd2` 已推 `feature/024-windows-native-ci`。本轮把桌面入口准备 Auto／固定端口的操作放到原生所有者锁内，先处理已有 Owner，再检查两份原记录，最后才为新启动读取或保存端口。沿用原 ActionWindow，保留未知拒绝和 30／5／10 秒期限。

| 检查 | 结果 | 证据 |
|---|---|---|
| 修复前，桌面 Start 遇到只有 `ocx.pid` 的夹具 | fail，退出 101。返回 owner unknown 并保留 PID 字节后，首次失败 `Start wrote to the data root`。 | [红测](raw/red.log) |
| 修复后，Windows 原生生命周期定向测试 | pass，5 项通过、4 项实机用例忽略。返回 owner unknown，PID 字节保持，数据目录为空。 | [绿测](raw/green.log) |
| Auto／固定端口与损坏设置保留 | pass，3 项通过。 | [端口设置](raw/port-settings.log) |
| 同源 MSI／NSIS 与实机复验 | blocked。主线 `a75d3b0af3e131e10b7ee8fcc2a7687378c7482a` 只有 CI `45960532202`，最新候选运行 `45863614604` 对应旧主线 `7580b0d79`。 | [此前实机首错与离线前提](../team81-20261127-7580b0d-r1/result.md) |
| 真实 Start → Status → Stop | blocked，尚无正向冷启动空闲证明。 | 本轮仅合成夹具 |

本机通过 MSVC、rustfmt 与 Git 差异检查。无真实运行时进程。测试临时目录未见本轮夹具残留，两份预存 schema 文件的 SHA-256 仍均为 `37ad285871f02aca13444391a6d2f14c4a7c6e8e464ff8cd5d32214d72387d49`。原始日志见[摘要清单](raw/sha256.txt)。

同源制品、独立断网的交互桌面、确认下载后的固定载荷可达、首次 GUI Status 计时及冷启动空闲证明沿现有交接。W2／双 SID 暂停，未发布，`accepted:false`。
