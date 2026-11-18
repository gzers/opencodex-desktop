# r7 保留句柄下的 V2 整对附件读回

输入 main 040f411e6d48d97bde6e7b2e6c22860e547b6173、治理 534e392dd54295d760c28a33a837ef8ec7975e73（TEAM67／IMP206／r7）、Windows 源码 36b70383eac5c44dffc51d34a4c4673cd8a3dbf7。代码分支的 V2 storage 将成功写入和读回的句柄移入 files 持有，原生创建访问为 0xc0020000，share 为 1。

待证伪判断是兼容读共享能让固定 CheckAttachment 的整对字节和语义校验通过，并保持原写互斥及拒绝条件。probe.ps1 从固定脚本 AST 提取实际函数。候选只将 CheckAttachment 的读入改为 Read access、ReadWrite share。原归档中的合成 V2 Intent/Pending 逐字节复制到临时文件，使用相同关键 CreateFileW 参数和 Flush(true)，两个写句柄保持打开。快照与绑定输入取自归档样本。本轮覆盖 CheckAttachment 和文件读取，真实原 Worker、完整 DiagnosticAfter 和原 5 秒期限须现场另验。

| 检查 | 结果 |
|---|---|
| 固定读法 | 首份记录 raw32，未导出 |
| 候选读法 | 3,523 和 3,669 字节，两个导出逐字节等于归档，语义检查通过 |
| 第二写者 | 两个文件均 raw32 |
| 再次导出 | raw80，原导出摘要未变 |
| 原失败动作错绑 | 在 Owner/action 绑定处拒绝 |
| 多余目录成员 | 导出前拒绝 |
| Pending 只追加 JSON 空格 | 在精确字节摘要链处拒绝 |

整体 PowerShell 5.1 raw0，stderr 为空。应用拒绝在 stdout 中保留异常消息及 HRESULT，低 16 位 5377 非 Win32 原错误码。两个合成句柄已关闭，合成目录已删除并检查缺席。固定脚本、测试镜像、计时观察脚本和实际现场未改。

从证据工作树启动 Windows PowerShell 5.1 子进程，仅从子进程环境移除继承的 PSModulePath。复现参数如下。

```powershell
-NoLogo -NoProfile -NonInteractive -ExecutionPolicy Bypass -File .adg/evidence/0.2.24/windows-dev/wl2-r7-range-readback-pair-20261119-r1/probe.ps1 -Launcher C:/Users/Public/ocx-r7-isolated-owner-range-r1/launch-r7-partial-owner.ps1 -Archive .adg/evidence/0.2.24/windows-dev/wl2-r7-initial-scan-gate-20261118-r1/verified/desktop-fixture-failure/D/before-resume-pending-33333333-3333-4333-8333-333333333333
```
