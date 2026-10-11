# 0.1.10 调度时钟边界修复与回归

关联 IMP-OPENCODEX-DESKTOP-20 / TASK-OPENCODEX-DESKTOP-182。实现归属 main，本文与证据归属 docs/governance-main。局部回归通过，版本验收未完成；不晋升核心事实。

## 源码与修复

代码 [deb740c09857b5f0bef7c9b4090fe5e8587bb4b3](https://github.com/gzers/opencodex-desktop/commit/deb740c09857b5f0bef7c9b4090fe5e8587bb4b3) 已提交推送，父提交为 3ed00487958185a520e61ed2ba2d706f95e9a384。四个变更文件及字节摘要见 [源码身份](evidence/source-identity.json)，未改缓存 schema、存储路径或检查周期。

- UI 的启动 45 秒宽限改用 performance.now()，系统时间前跳或回拨不改变已经经过的时间。
- 后端规划前只修正当前管理器通道和面板超出既有周期 / 退避上限的未来截止，并先持久化一次。失败次数和 last_success 历史保留，非活动通道不改；普通后续唤醒不会不断延后截止。
- 失败退避仍为 5 分钟、30 分钟、2 小时，随后回到该目标既有周期；修正时不立即查询。时间前跳后的过期目标沿用既有预留逻辑防止重复查询。
- 写者准入先于调度互斥锁；自动检查关闭、忙事务或数据根冻结时，不修正截止、不生成查询目标。负数历史等非法状态拒绝读取，原始文件不静默覆盖。

这是时间与存储边界修复，不解释旧十分钟探索性对照的 CPU 差值。stable 24 小时、beta 6 小时及面板周期的真实跨时验收仍需实施。

## 回归结果

| 检查 | 结果 | 原件 |
|---|---|---|
| 有效修复前 UI 回归 | 7 pass / 2 fail | [日志](evidence/ui-red.log.gz) |
| 有效修复前 Rust 回归 | 3 fail | [日志](evidence/rust-red.log.gz) |
| 修复后相关 UI | 4 文件 / 24 pass | [日志](evidence/ui-green.log.gz) |
| 修复后 Rust 命令边界 | 4 pass | [日志](evidence/rust-green-command.log.gz) |
| 完整 UI | 95 文件 / 474 pass | [日志](evidence/ui-full.log.gz) |
| 完整 Rust workspace | 34 suites / 866 pass / 0 fail / 6 ignored | [日志](evidence/rust-full.log.gz) |
| fmt / strict clippy / UI 类型与构建 / diff | exit 0 | [结果汇总](evidence/test-results.json)及同名命令清单 |

六个 ignored 项涉及 live / keychain / npm / WebDAV / helper / native tray 等环境相关路径，不计通过。早期命令清单只有 argv / exit，没有起止时间与持续时间，不补造性能数字。外部 code-before 快照含修复前测试编辑，不是干净产品基线；本文没有复制源码补丁。

## macOS 同 SHA 候选

[构建与独立核验](evidence/candidate-verification.json)记录 arm64、版本及 build 0.1.10、EXE 内完整源码身份，hdiutil verify exit 0。构建 exit 0，耗时 122.561674916 秒（构建耗时，不是启动性能）。

| 原件 | 大小 | SHA256 |
|---|---:|---|
| OpenCodeX Desktop_0.1.10_aarch64.dmg | 4391555 bytes | b12b55e4c78e526d693d7deb9698f7922b9b2548f03befca63e9b9a1b88db877 |
| 原 EXE | 7914288 bytes | 5de76891f9f9ee9075dde9a4833bfa31a536b00e5984b81eb328f5650671799c |

原件 strict codesign exit 1（unsealed resources），宿主无有效代码签名 identity / TeamID，公证未验证。隔离副本的 ad hoc 签名通过不能替代原件正式签名。应用与 DMG 只存摘要，不进入文档仓库。

## 证据及后续

[归档核验](archive-verification.json)：40 项证据摘要、38 份复制来源逐字节 / gzip roundtrip 通过，2 项大文件仅摘要；归档时 138 个无关已跟踪脏路径未变。[来源清单](evidence/source-provenance.json)与 [SHA256SUMS](evidence/SHA256SUMS)用于复核。日志采用 mtime=0 的无损 gzip，没有复制用户载荷、缓存、WebView profile 或范围外资料。

本提交之后的原生缓存前台、Windows 同 SHA 候选与局限见 [deb740 前台及候选复验](../20261011-deb740-缓存前台复验/README.md)。八项 TASK.required_checks、配对性能、真实周期、双平台原生 / 安装 / 真实更新 / 事件和正式签名仍开放；IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。
