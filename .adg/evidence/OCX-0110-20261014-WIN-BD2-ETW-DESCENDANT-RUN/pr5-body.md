为 bd2 原查询失败取得自有 Node/cmd 的进程、线程、调度、初始化及文件 I/O 原始时序。采集器仅位于 test/smoke，基于 integration-candidate@bd2f6eb1，不修改产品、原夹具、工作流、默认并发/capture 或 2/4/5/8 秒预算。

固定 TraceEvent 3.1.21，先 kernel enable 再 Source，StopOnDispose=false，以 ControlTraceW 只停止唯一自有会话，不调用重设机器 CPU 采样间隔的通用 Stop。不写整机 ETL，不保存无关命令行/SID/路径或其他调度参与者 ID；按原 QPC 和 PID/TID 生命周期准入，队列/IRP/输出均有界。

首次 abc 录制在 MainModule 根映像校验处中止。629b9c6 改为已持有 Process.Handle 的 QueryFullProcessImageNameW，并立即异步读取原输出，异常时分别排空。629 的 push38567345341/PR38567352891已成功，但通用 CI 不执行 C# 采集器。

用户持续授权修正版管理员后台 ETW 后，629 于 2026-10-14 21:23:35–21:23:46 完成一次原失败 PR38537166560测试EXE的本机完整库：529passed/1failed/4ignored，七query均通过，唯一license.opened因编译时runner路径缺失；不改断言或造路径求绿。真实测试根 elevated=true/type2，consumer/writer排空与自有session stop成功、最终kernel loss0，原输出保留；未复现目标失败。

该轮暴露容量缺陷：测试根高频事件占已保存记录97.27%，64MiB在最初约0.64秒用尽，1339173条丢弃，终止事件缺失；CRLF只按LF计数，实际文件超上限182992bytes。183278条原JSONL已无损gzip封存，不能认定完整覆盖或用零kernel loss抵序列化丢失。

3a91c175接续修正：保留根身份/进程、线程及映像生命周期；高频调度/文件I/O限已准入Node/cmd后代，显式统计省略的根事件，不把省略说成根无活动。严格按UTF8+LF写入和计量，queue失败不占字节预算。保留64MiB/32768queue/32768IRP、120秒原库上限、原全部断言/并发/capture；不预热或无限重跑求绿。

本机Add-Type、原58检查及新增10后代范围/10UTF8-LF实际写入检查、摘要/目录/普通权限拒绝通过；仅自身句柄和内存流，无原库/Node/ETW新执行。3a91c175 的 push38572890687 completed/failure：Windows库527passed/三个version查询夹具失败/4ignored；PR38572902717 completed/success，Windows35组627passed/7ignored。两轮Mac各900passed/6ignored。push控制进程在约2秒query返回仍标记缺失，约5秒仍active、标记已出现；无调度/I/O根因证据。完整日志ZIP/SHA/CRC与实际检出树核验，原失败及本次失败均保留，PR绿不解除push红。通用CI未运行collector；仅在原夹具/预算/产品字节不变、78准备检查通过、失败精确限定并保存后作诊断准入，不是版本/发布准入。旧双绿辅助入口不执行，修正后实时采集仍待验证。下一次有界采集已获持续授权，只在新SHA准入后执行；UAC仍由用户手动确认。

保持草稿。原bd2七query失败及根因、macOS CPU/RSS失败、完整双平台原生/安装数据保留/更新备份事件/真实周期/官方0.1.9配对性能门禁仍开放。保留日常版与真实数据，不合并、签名、晋升、发布或关闭TASK。


3a91c175本机单次录制已完成：8673条/2818320bytes，UTF8-LF实际/预算字节相等，serialization/kernel loss及decoder error均0，consumer/writer/stop正常、根与七Node start/stop齐全。但无cmd及查询后代，完整家族覆盖未证明；七query均通过，唯一license.opened，不能解除原失败。

0407fdb2改为优先精确ETW实际映像；只有basename时仍要求同名且完整精确argv首参数，冲突路径/未知映像/仅裸文件名拒绝。已知映像子进程拒绝有计数与脱敏原因，不默默宣称覆盖。本机89检查通过（78+11实际映像/argv冲突与弱证据检查），未新跑原库/Node/ETW；0407同SHA CI与实时验证待实证，不混用3a结果。
