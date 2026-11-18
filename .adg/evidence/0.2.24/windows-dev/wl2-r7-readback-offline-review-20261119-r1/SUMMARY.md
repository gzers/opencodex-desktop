# r7 独立读回的归档验收

输入 Windows 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7、main 040f411e6d48d97bde6e7b2e6c22860e547b6173、治理 534e392dd54295d760c28a33a837ef8ec7975e73，TEAM67／IMP206／r7。独立观察脚本负责真实同轮计时。review-readback.py 在该轮结束后只读审查归档副本，既不启动任何动作，也不提供新的计时区间。

审查时从已核验 Explicit 回执和批准配对传入三个 SHA-256。脚本核对原 SID/session、一次性尝试绑定、Explicit 拒绝中的同一持久 Pending、正式诊断回执的原字节摘要以及两份导出的实际大小、SHA-256、动作、顺序和摘要链。用整数原始 QPC 校验 0 ≤ 差值 < 5 × frequency，并核换算值。计时上界显示 5000 ms 时，以原始 tick 判断是否严格在期限内。

Python 3.12.10 用两个旧归档合成记录和七组模拟回执验证。距截止一 tick 接受；恰到截止、显示时长被改、launcher 错配、Explicit 摘要错绑、重新计算摘要后的动作错绑，以及 Pending 多一个字节均拒绝。整体 raw0，stderr 为空，合成临时目录已清理。真实固定配对、会话入口及现场没有改写。

正式回执到齐后，可用 review-readback.py ARCHIVE --explicit-sha256 SHA --cmd-sha256 SHA --launcher-sha256 SHA，参数来自现场此前的独立核验与 Mac 固定配对。先保全实际失败及原始回执。离线通过只证明证据绑定和时限检查，现场原 worker 和持久化结论仍取自已验收的同一轮。
