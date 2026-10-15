# Windows bd2 采集器生命周期修正

2026-10-14（UTC+8），管理员录制批准仍缺，自动目标接续只完成不需提升权限的代码审阅、准备测试及来源回写。本轮没有再次执行原测试/Node/desktop或ETW会话。

## 发现并修正的采集器问题

旧collector5fa36a6的38项纯scope及普通权限拒绝检查没有覆盖真实TraceEvent会话创建顺序；旧两轮CI也只跑原产品回归，不执行这个C#工具。因此旧绿不证明正式录制可用。

按固定Microsoft TraceEvent3.1.21源码核验：

1. `session.Source`在inactive时会调用EnsureStarted创建普通session。旧collector先取Source.Kernel再EnableKernelProvider，而该库明确拒绝在已started的session上开启kernel；这是采集器准备缺陷，不是原查询产品失败。新顺序先Enable，再访问Source。
2. 库通用Stop会不依赖本次Profile是否启用而重设机器范围的采样间隔，并调用heap tracing清理。原“不改全局环境”方案因此不能直接用该Stop。新collector关闭StopOnDispose自动Stop，使用Win32 ControlTraceW STOP按唯一 `OpenCodex-BD2-<GUID>` 名停止自有session，记录该session最终loss/real-time buffer计数；不调用TraceSetInformation或其他会话Stop。实际API调用仍待批准实跑。

依赖源原字节、oldcollector源码blob和Win32官方依据绑定到 `.adg/evidence/OCX-0110-20261014-WIN-BD2-COLLECTOR-LIFECYCLE/`。TraceEvent非自动Stop的Dispose只释放解码/会话handle，SafeTraceHandle源CloseTrace已复核；不把源码审阅称作实际终态验证。

同时修正背景writer异常可能退出controller、单个cleanup阶段错误跳过后续清理的路径：writer保留错误并持续排空队列，cleanup逐阶段尝试并报告具体错误。consumer未排空不能与handle list竞争，完整覆盖立即失效，不编造“已停止/零丢失”。ProcessExit尽力收尾只作用唯一自有session，不能保证强制进程终止仍有收尾机会。writer异常分支仅编译审阅，未实际注入writer线程故障；cleanup单阶段异常后继续的规则有纯检查。

## 实现和准备检查

`feature/0.1.10-windows-query-etw@abc3a9140fc6f1b5d2aa6e739299af260f0f5d94`接续5fa，仅改原三个test/smoke工具文件；apps、原夹具、CI工作流、共享文件和预算未改。PR#5更新标题/正文，读回draft/open/新head/未合并/无auto_merge；新push/PR CI观察仍进行中，旧5fa双CI不用于证明abc。

本机Add-Type编译及53项纯准备合同通过：

| 检查 | 数量/含义 |
| --- | --- |
| Scope | 38，PID/TID重用、原时间戳、非自有拒绝等 |
| 生命周期 | 7，旧getter-first独立错误模型拒绝、新顺序通过、Enable失败后不访问Source、逐阶段清理 |
| Native stop | 8，x64 WNODE/EventTraceProperties布局、唯一名称拒绝/Buffer内容等；不调用ControlTrace |

摘要不符、输出越界、普通权限前置拒绝检查继续通过。全部检查均无ETW/原测试/Node执行，不能据此声明真实kernel enable/过滤/数据完整/排空/stop通过，也不能解除bd2原CI失败。

## 更新后的待批准入口

当前可审阅入口：`.adg/evidence/OCX-0110-20261014-WIN-BD2-COLLECTOR-LIFECYCLE/record-request.ps1`。固定abc新SHA并要求无本地变化，DLL来源仍为前轮已验证metadata SHA512/ZIP CRC；旧2fd84aa准备原件保留，旧入口固定5fa会拒绝当前新HEAD，不能继续沿用旧命令。

默认不加参数只显示方案。实际录制仍需用户明确批准、手动管理员PowerShell7和 `-ApprovedByUser`；不自动提升。只执行一次原失败PR测试EXE/SHA497814ca…和官方Node24.21.0/SHAba4e6d11…，默认并发/capture及2/4/5/8秒不改，原库外层120秒保护另加短时收尾。系统kernel事件短暂内存解码，只有自有家族原事件字段持久化，不写整机ETL/外方SID或命令行，不改防护/全局环境，不FZ50脚本。数据/queue/IRP仍有界。

有界CI等待器本轮600秒终态结束，后续直接读回push38561752521/PR38561759502仍in_progress，四通用job成功、Windows处于最终EXE构建。没有将网络读取错误或等待器截止当作远端停止，也没有重派；25份修正/准备原件冻结校验通过，不在原目录追加后续CI终态。

原七失败尚未复现，官方Node/desktop目前均0，采集输出根不存在；53检查或新源码不是失败场景证据。原门禁、macOS性能failed/预算、TASKrevision2/in_progress/pending/FZ50许可以及日常/真实数据边界保留，goalactive，不合并、签名、发布、晋升或关闭。
