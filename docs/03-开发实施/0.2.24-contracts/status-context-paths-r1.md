# 0.2.24 状态路径与实际运行端点的区分

revision: 1；2026-11-12。归IMP-34 P24-03/04，承接[恢复状态重建](recovery-readonly-view-r1.md)。代码main f5fc1fef0d98fb5157244dc62ee673a44e9fb153。当前是共享状态事实修正，accepted=false、非冻结候选；不代表完整客户端恢复或原生验收完成。

## 原因及实际证据

上一批真实官方2.99产品快照中，已停止服务仍带原54109/54697端口，facts.opencodex_home为某个config.json文件，data_root为空。该真实快照原件继续保留，本批仅引用源SHA及提取副本，不把它改成新的成功结果。

直接检查未改的官方tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c内src/cli/status.ts：paths.config=getConfigPath()，是文件；listen.port来自已选择的监听配置，proxy.running单独表示运行事实。原Desktop映射把paths.config当HOME、无条件投影listen.port及残留PID，导致展示混淆。不能靠对config.json取parent来猜实际HOME，也不能把配置端口当运行端口。

## 已改行为

StatusSource.manager_paths提供启动时已解析的D与OpenCodex HOME。生产OfficialStatusSource从现有显式Desktop root及真实启动environment中返回它们，不额外查磁盘或运行CLI、不猜旧安装路径。collector成功映射、轻量/完整去抖及查询失败时都优先使用此上下文；官方payload的任意dataRoot或HOME不能覆盖Desktop启动位置。上下文路径只证明当前配置来源，不证明服务就绪、进程归属或文件恢复许可；查询失败仍保持原Unreachable/未知分类。已知D消除MissingDataRoot降级原因，其它异常不被路径事实抹掉。

纯官方payload仅提供paths.config时，HOME保持未知，不再把文件当目录。明确stopped/not-found结果或proxy.running=false折叠出的Stopped/NotFound，运行port/PID清空；配置端口仍归既有runtime_port_load及设置保存事实，不删除或改写配置。只有未知status沿用旧状态时，不额外推定新的端点缺席。运行/待就绪/风险/不可达原状态映射及恢复门槛不因这批变更获得成功权限。

产品原包定向入口另强化未来验收：reopened快照必须D/H等于本轮显式上下文、port/PID均空；完整恢复断言保留。本轮没有再执行该完整启停入口，不能宣称此增强已通过官方真实服务整链。

## 验证

32项相关检查通过：19项原状态组（18项首轮通过、1项更正错误期望后单独补验）、3项新路径/去抖/失败/未知端点检查、1项恢复门槛、5项DTO、3项轮询、1项真实子进程命令消费。未无假设重跑所有已过项。

原状态测试official_schema_maps_startup_status_and_facts确实断言config.json文件等于HOME，首轮失败原stdout/stderr保留；按照已核对官方源码改为无上下文时HOME未知，其它运行、port、health、version等断言保持。新实际上下文测试另证明D/H正确，不能把修改错误期望当作完整原生验证。首个测试launcher从tauri目录误读repo相对binary路径，copy阶段FileNotFoundError，尚未运行测试；仅原工具输出和分类留证，未捕获独立raw stderr文件，边界明确。

真实命令检查在OS沙箱/private profile内经生产OfficialStatusSource/collector执行一个明确合成/bin/sh状态输出，确认stopped/端点空、实际注入D/H、D目录无写入。它不是官方2.99 binary，也没有启动OpenCodex服务。官方事实由原tgz源码及前批真实快照提供；不能把合成状态当新真机服务通过。最终编译/格式/全目标严格Clippy通过。

## 留证、清理与剩余

.adg/evidence/0.2.24/status-context-paths-r1/保留官方源member/完整SHA及相关行、前批真实快照源SHA/提取副本、首轮错误及后继结果、生产源码/binary摘要、manifest和cleanup。OS沙箱实证拒绝日常HOME读取和外网；自有测试/命令进程已空，测试binary和全部私有profile/合成配置/重复回执根清理。没有运行业务模型/WebDAV、改日常配置/Shell/安装或官方包，未构建新.app/DMG或发布。

前批Stop正常exit0但索引停在revision4的内部失败仍未归因，固定phase/code诊断保持；本批未重试或宣称修复该问题。实际客户端恢复/完整集合监听写者、显式继续/回退及其余全树/通用图范围保持；WindowsW0 HOME/Profile/W1/W2、NAME、npm System代理、官方发布验证与固定候选继续，目标不关闭。
