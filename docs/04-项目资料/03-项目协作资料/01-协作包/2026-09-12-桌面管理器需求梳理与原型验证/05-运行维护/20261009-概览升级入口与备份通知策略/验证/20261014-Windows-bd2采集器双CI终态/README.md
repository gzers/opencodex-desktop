# Windows bd2 采集器分支双 CI 终态

2026-10-14（UTC+8），自动目标接续只推进无需新权限的后台检查，**没有把自动接续当作管理员录制批准**。采集器与原失败制品仍按[已封存准备方案](../20261014-Windows-bd2实时采集准备/README.md)执行边界等待批准。

## 新的确定事实

`feature/0.1.10-windows-query-etw@5fa36a60a6867124cf13f27e1e1856ae495ef29b` 的 push CI38555890377、PR CI38556292712均completed/success。每轮五必需job成功、main-only build skipped；macOS/Windows各35组，分别900/627passed、6/7ignored，ignored不计通过。原七query夹具在两轮通用回归中均通过，原PR38537166560失败继续保留。

独立push实际checkout5fa36a6；PR实际检出a813c1dc964e5f4cc441aa75d79a7d2277823dbd。两checkout tree均fa1d4d544f3df3b4f4e11747bb59f20d27a6b48d，不混同两个commit或各自编译制品。5fa相对bd2只新增三个测试工具文件，apps和原CI工作流未变。

完整日志ZIP原字节、CRC、安全条目、两个后端各自actual checkout与测试原日志独立封存。首轮解析误把Post checkout也当初始checkout，触发不覆盖保护；修正精确step选择后复用原ZIP和已保存字节，不重下载改写原件、不重派CI。一次读取默认GBK失败改为显式UTF-8，原文件不改。

有界只读等待器exec67983/PID33408终态已验证，耗时166.016秒，小于600秒等待边界；网络观测错误没有被解释为远端停止，也没有重启工作流。最终两个CI均终态后等待器正常结束。

证据：`.adg/evidence/OCX-0110-20261014-WIN-BD2-COLLECTOR-REVIEW/`。`verify-evidence.py`按路径映射/摘要复核原件。

## 保留范围

这些CI运行原产品回归和既有Windows首屏冒烟，**没有运行实时ETW采集器或38项纯scope检查**；38项编译/范围检查仍仅归前轮Windows本机准备证据。因此不能从CI绿推断管理员record、实时事件覆盖、数据过滤/排空或实际失败复现通过。

本机desktop、官方Node均0，预定管理员输出根不存在；PR#5最新读回仍open/draft/未合并/无auto_merge。没有新的本机原库/Node/ETW执行，没有自动提权或手动CI重派/候选打包/原生交接。管理员单次后台录制仍等待用户批准，范围、120秒原库外层保护与短时收尾详见原方案。

目标保持active，原失败、macOS性能failed、原预算与八门禁/TASKrevision2/in_progress/pending/FZ50许可边界均保持；不提前关闭、签名、发布、晋升或替换日常安装。
