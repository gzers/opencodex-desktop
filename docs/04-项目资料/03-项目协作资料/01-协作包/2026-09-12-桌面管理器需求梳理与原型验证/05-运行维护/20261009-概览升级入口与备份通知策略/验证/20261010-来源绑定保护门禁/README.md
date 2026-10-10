# 运行来源绑定保护门禁与原生复验

源码 feature/0.1.10-maintenance@d3e4df32eba37b087957a175d70b2b7f0097e12c 已提交推送；版本仍为 0.1.9 开发态，未合入 main。

macOS 隔离原生复验 56c8462c 时发现：损坏保护记录保留，安装及目录修改受阻，但“更换运行来源”“恢复自动发现”仍可操作。旧截图与 AX 保留为失败证据。d3e4df32 在运行变更锁内、候选验证及绑定写入前补上保护核验；前端在未知、读取中、未决或不可读保护状态禁用来源按钮，并在文件选择返回后再次检查，避免选择期间状态变化后继续写入。

修复后 macOS 27.0.1（26A434）arm64、custom-protocol debug、隔离 bundle com.gzers.opencodex.desktop.sandbox.sourcegateD3：损坏记录不展示其版本 / 前缀字段；两来源按钮及安装 / 离线导入均禁用。本地重试返回 protection_reconciliation_pending，提示与禁用状态保留。保护记录及四个备份文件共五份摘要前后完全一致。没有执行安装、卸载、迁移或真实面板更新。

构建前二进制 SHA-256：07649084359fb46d46a314273ad851463274c615c8fb0403855c133481dfab89；隔离 bundle ad-hoc 签名后二进制：2f93ced429aea703756b6e12ac52e2ba0e0eb397ad6f88a9ff340c87edd803a4。隔离 HOME、数据与 bundle 均在 /tmp/ocx-0110-source-binding-native-d3e4df32；自动检查关闭，未改日常安装。

最终定向 UI 12/12、Rust protection 16/16；类型检查、生产构建、Clippy all-targets -D warnings、fmt、原生 custom-protocol debug 构建通过。文件选择期间出现保护状态与正常可选来源路径均有 UI 回归。Rust pending / verified_pending / corrupt 拒绝绑定，记录不存在允许继续。本提交没有重跑全量；56c8462c 的 765 / 447 结果保留为此前基线，不冒充本提交全量通过。

原始失败 / 修复后 AX、截图、文件摘要、源码与制品身份及定向日志：.adg/evidence/OCX-0110-20261010-SOURCE-BINDING-GATE。SHA256SUMS 可逐份校验。历史 exploratory current.png 实为 JPEG，未纳入证据，保留正确编码的 before.jpg / after.jpg。

这是来源绑定与限定重试的局部原生通过，不抵实际更新 / 重启、安装器、Windows 或整版本门禁。损坏关联受控修复、全领域事件、配对性能 / 真实周期仍待完成；TASK-182 / IMP-20 in_progress，不回写稳定核心，不发布。
