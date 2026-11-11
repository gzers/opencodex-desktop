# 0.2.24 停止事实与客户端恢复结果分离

revision: 1；2026-11-12。归IMP-34 P24-04，承接[原生停止事件](macos-exit-events-r1.md)及[恢复计划](restore-plan-product-r1.md)。产品代码main 532298981c1703755372de16937be69bc09137cb；独立旧测试夹具修正8ef251554。accepted=false、非冻结候选，未开放实际文件恢复。

## 已修正的真实问题

上一轮实际原包已经正常exit0，且独立索引到RestorePrepared，但process_action的错误分支一律回退到动作前的Running，canStop/canRestart仍true；前端也会把失败Restart标为启动失败。本批使服务停止事实与原件恢复结果分别表达，后继真实恢复冲突仍需要此分支，不能靠删掉完整恢复断言绕过。

CompletedLifecycleStop在原context/data/HOME/运行对象及原deadline核对后产生私有StoppedObservation。它无公开构造器、无反序列化、无文件写许可，只是本次动作时点的停止观测。普通RuntimeManaged错误文本、runner报错码或数据库JSON不能生成该观测。

实际Mac原件流程在正常Stop后保留观测，再做原阶段/计划登记。后继失败返回lifecycleState=stopped、result=null、clientRecoveryRequired=true、canStart/canStop/canRestart=false，并保留原失败；没有把服务停止改写成客户端恢复成功。原引用缺失或PreStop不确认时仍只请求一次普通认证Stop，得到真实停止观测后才能投影上述状态。归属未知、正常停止未确认或超时仍不产生StoppedObservation，保持原保守状态。

## 应用内消费及限制

前端保留独立恢复标记，待恢复Restart不再触发lastStartFailed。普通stopped状态轮询、一次未知/超时动作和刷新均不能清掉该标记。概览与端口设置沿用既有布局，仅禁用当前不允许的启动/重启并给出待恢复说明；真实后续running观测的can_stop允许对当前服务再请求普通Stop，不授权重启。正常完整动作结果才可消除本实例标记。

本批是动作回执与当前应用实例的消费，不是完整持久恢复界面。重新打开应用、托盘和后端状态快照尚须从原受保护索引按实际身份只读核对恢复门槛；当前后端reserve_start仍拒绝未完会话，不会因前端重开而重拍基线。不能把本批当所有入口已经统一、显式恢复/回退已完成或原型最终对照已通过。原型checkout中其它工作保留，未覆盖其修改。

## 验证

10项DTO定向检查与3项动作边界检查通过，其中普通runner仅返回同名pending错误的Stop/Restart不能冒充真实停止，HOME不被创建；DTO的私有合成投影仅证明序列化/门槛，真实观测签发另由原包链验证。最初动作边界夹具禁止全部网络，实际端口准备被拒port_permission；保留失败，只给该用例增加loopback允许后通过，没有改产品/断言或重跑已过项。

前端59项相关检查、类型检查、编译、格式与全目标严格Clippy通过。首次前端组58通过/1失败；用改动前afa4fca03的全部相关生产Vue/store源码复核同样失败，定位旧环境用例未模拟runtime_preparation_status，undefined被新准备组件正确判为读取失败而隐藏旧检查列表。仅为该旧用例补明确idle回执，原3项检查/准备卡/三卡收起等断言全部保持，独立测试提交8ef251554；不把旧失败归为产品停止故障。

一轮真实隔离官方2.99产品入口：Start15616ms、Status1104ms、Stop含计划登记8515ms，原30/5/10秒未变。真实Stop原内核exit0、记录缺席，stoppedRecoveryObservation=true；结果是stopped、clientRecoveryRequired=true、result=null、三门槛false。独立RestorePrepared/revision6和原phase/plan摘要、原stage身份核对。实际catalog/cache/audit尚未恢复，完整原断言仍exit101/accepted=false，没有清成功原件/放行Restart/签发MutationPermit。

## 证据与后继

.adg/evidence/0.2.24/stopped-recovery-observation-r1/保留原失败、基线复核、最终结果、实际trace与原索引/phase/plan/stage、准确11件源码及binary摘要、manifest和清理事实。仅自有合成配置，无runtime secret/admin token原件。完整Darwin TCP表核对原端点缺席，自有镜像进程为空；新N/K/history数据库排他取得/释放后删除；自有程序/profile/binary/索引/stage及核对后的重复证据根清理。未改日常配置/Shell/正式安装/官方2.99，无模型/WebDAV业务请求，未构建新.app/DMG或发布。

继续完整进程集合/监听/写者证明和实际补偿、成功读回/关闭、持久快照/菜单门槛与显式恢复；config/Grok/Agent/全树、通用渠道图、新目录/跨卷/flags保持原范围。Windows无新回交，仍b2e89665d/0eef59a01；W0 HOME/Profile、W1/W2、NAME、npm System代理、新发布验证和固定候选继续，不关闭目标。
