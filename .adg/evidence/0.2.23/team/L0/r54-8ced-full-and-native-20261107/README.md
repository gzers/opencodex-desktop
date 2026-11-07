# 0.2.23 精确新源全量与 macOS 实屏

代码：8ced5d44212eb2b58e1dca5344c5be7145747377。

本机按仓库 CI 顺序先单独完成 metadata_query_rejects_oversize_and_kills_timed_out_process_group 原断言（1 passed，0.22s），再执行全 workspace 跳过已完成的同一夹具：49 顶层 binary 全部尝试，2214 passed、0 failed、37 原忽略，doc 通过；两命令退出均 0，合计 2215 passed。统计取每个顶层 binary 的最终摘要，不重复计算子进程回显。没有新增跳过或放宽原预算。

解锁后独立的实际普通首装通过：新固定用户目录、离线供应 2.97.0-desktop023.names.1、managed 来源、正式 release commit 与二进制摘要一致。人工查看 exact PID 截图：正常概览显示未运行/启动 OpenCodex，无陈旧安装引导；网络在操作系统层被拒绝，版本检查失败为预期，不声称线上更新通过。未启动服务、未真实业务同步。

测试进程退出、临时 HOME 删除、独立安装副本按 PID 退出与二进制摘要校验后精确清理；截图和成功日志保留。r53 原锁屏失败不覆盖。DMG 候选引用 r53，仍非正式发行。Windows 最新同源制品与安装/双 SID/保留和最终发布门未闭合，TASK 保持 in_progress。
