# TEAM83 Windows 原生 Start 运行记录拒绝

待证伪判断（2026-11-27）：在无保留 Owner 的 Windows 原生 Start 入口，仅 `ocx.pid` 存在、`runtime-port.json` 缺席时，应当返回 `windows024_owner_unknown`，且保留 PID 记录。

源码 `c76116f8e5e2f299a90f1b75e0b54228c144cce5`，已推 `feature/024-windows-native-ci`。Start 复用 Stop 和 stopped Status 的记录检查。两份记录各自都必须返回 NotFound，现存记录或读取错误继续拒绝。冷启动空闲还需正向证明。沿用原 ActionWindow 与 30／5／10 秒预算。

| 检查 | 结果 | 原始证据 |
|---|---|---|
| 修复前，仅有 `ocx.pid` | fail，退出 101。首错为 `FileSystem { operation: "validate process executable", … }`，期望 `windows024_owner_unknown` | [红测](raw/red-msvc.log) |
| 修复后，原生生命周期定向库测试 | pass，退出 0，5 项通过、4 项实机用例忽略。PID 用例返回未知，原 PID 字节保留且未创建输出目录 | [绿测](raw/green-msvc.log) |
| 同源 MSI／NSIS SHA-256 与单用户桌面验收 | blocked，候选待集成线回交。完整包离线桌面、确认下载后资源可达、首次 Status 关联计时和冷启动空闲证明继续沿既有交接 | [实机基线和首错](../team81-20261127-7580b0d-r1/result.md) |
| 真实 Start → Status → Stop | blocked，尚无明确的冷启动空闲证明 | 无实机启停 |

rustfmt 和 Git 差异检查通过。修复后测试 EXE SHA-256 为 d396f8594c62e4504ee2bfd7a1212474aeaa99eb7ee45566838b8956e13738b4。测试仅使用故意缺失可执行文件的合成夹具和库断言，无真实运行时进程。已核对本轮夹具无新增残留，两份预存的 schema 文件逐字节恢复，SHA-256 均为 37ad285871f02aca13444391a6d2f14c4a7c6e8e464ff8cd5d32214d72387d49。原始日志见[摘要清单](raw/sha256.txt)。

Actions `45956736730`、`45956890861`、`45958601260` 的 Windows 后端在本次快照仍运行。前端、发布工具和独立 Windows 冒烟通过，当前制品为前端和进程令牌证据。实机候选待交接。历史失败现场保留，暂停 W2／双 SID，未发布，`accepted:false`。
