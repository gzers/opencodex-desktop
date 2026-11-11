# 0.2.24 原会话只读观察与状态／菜单恢复门槛

revision: 1；2026-11-12。归IMP-34 P24-04，承接[停止结果分离](stopped-recovery-observation-r1.md)。代码main b92dc31f4d2f63e5439f1e824e3f8609bbbf10f8；accepted=false、非冻结候选。此批解决重新创建采集器时丢失恢复门槛的问题，不是实际文件恢复或完整后代证明。

## 原生只读观察

macos_client_session_observation::inspect仅消费既存schema2索引；不初始化/修权限/迁移/重拍原件。先固定D、manager目录，打开独立action lock并验证owner/普通单链接/0600/ACL/flags/xattrs，再取得共享flock；busy立即未知，不打开同进程写者持有的SQLite inode。连接／pin按顺序释放，避免额外fd关闭释放本进程fcntl锁。真实跨进程探针证明同进程写事务在一次busy观察后，另一个OS进程BEGIN IMMEDIATE仍明确SQLITE_BUSY，父锁未泄漏。

持锁后验证索引大小、原生身份和保护、rollback-mode头／无journal/wal/shm；URI通过file-path编码处理Unicode及?#%，SQLITE_OPEN_READ_ONLY/NOFOLLOW/URI、immutable=1和query_only保证SQLite不执行恢复或建立sidecar。长度上限64KiB文档、16MiB索引、busy=0；只接受原三表精确DDL，拒绝view/触发器/未知结构。核对当前真实UID/D/profile/HOME/client目录代次绑定、Head/schema/原件引用类型、最新事件与当前document一致，之后复核VFS_HAS_MOVED、原锁/目录/头/sidecar并关闭连接。没有用扫描原件目录重新赋权或从JSON声称服务已退出。

索引缺失且无原件时才是无会话；一半初始化、索引行丢失但有事件、比当前更新的事件、旧schema1、遗留日志、错误权限/重定向/另一HOME均未知且原内容保持。client-originals目录仍在但原索引/当前引用缺失时未知；写侧也拒绝初始化或reserve_start，防止误建新基线。该收紧没有旧版兼容/自动迁移。真实缺失目录没有被观察API创建。

## 共享状态合同

RuntimeFacts.client_recovery是Desktop本机独立枚举，不由官方JSON解析：not_required、active、required、unknown。显式当前D作为来源输入；Ready索引仅表示原件会话active，不能被当作真实运行/健康证明。其它已提交未完阶段required；busy/错误/输入或平台后端缺失unknown。非托管来源没有这套会话时not_required，Windows托管原生观察尚未接，明确unknown，不拿Mac数据库或缺字段冒充已完成。

官方状态采集失败、原payload声明not_required、轻量/完整去抖缓存都不能清掉本机恢复事实。它不改运行状态：原服务状态仍由原认证/OS观察提供。Required/Unknown禁止start及restart，Active禁止另开start但保留既有restart流程准入；stop只按真实运行门槛，不因恢复未知阻止普通Stop。后台事件、显式状态查询、CLI IPC facts、托盘和应用菜单投影共用同一枚举/门槛，既有collector实例换成新实例后仍读取原独立索引。应用启动接线显式传D，不猜旧路径。

UI初次状态快照可重建待恢复标记，unknown仅阻止启动/重启并显示待核对，不冒充“恢复失败”或“服务已停止”。晚到not_required快照不能清掉同实例已获得的动作回执；完成恢复／完整动作结果仍需独立权威处理。概览及端口设置继承同一门槛，未增加忽略/清索引按钮。

这不是所有恢复UI完成：显式继续/回退、成功读回与关闭索引、完整客户端范围、所有控制进程/监听/写者证明仍未实现，MutationPermit无生产工厂、apply仍不执行。

## 验证与失败边界

57项后端相关检查通过：最终20项索引（另1项旧现场probe因无该旧现场明确ignored，不算通过）、19项状态、1项去抖/失败独立事实、5项DTO、6项托盘、3项菜单、3项轮询。commands::tray::tests筛选实际为0项，不计入57；真实collector集成用例直接调用tray_state_from_parts核对三门槛。索引检查包含实际OS跨进程SQLite锁、原DB逐字节未变、无创建/日志、Unicode路径、已停止plan重开保持原引用、旧结构/篡改/孤立原件等反例。诊断日志/只读data accessor后未重跑整组，最终原生轮直接执行最终源码。

64项前端相关检查、类型检查、编译/格式/全目标严格Clippy通过。首编缺std::fs引用、首类型检查夹具遗漏/重复新增字段、首Clippy一处测试初始化写法均保留原stderr；修正引用/夹具/初始化，无放宽断言或新增ignore。

真实原包第1轮Start15534ms、Status1104ms、Stop6005ms，原正常exit0、动作stopped待恢复，但索引停在StopPrepared/revision4，没有Stopped frame/plan。新采集器从原索引读出required、原官方状态stopped，三门槛false，reopened794ms。该轮内部错误被旧pending映射隐藏，不能凭时间推定归因，也没有声称已修复。保留原索引/阶段及trace。

为定位此具体失败，仅增加固定phase/code诊断（native-coordination/seal-stopped/index-stopped/prepare-plan/index-plan/release-coordination），限制安全分类字符和长度，不记录路径/secret/错误body、不改变判定/预算。第2轮用全量重验的同一不可变镜像及全新profile，没有重试原失败会话：Start13611ms、Status1148ms、Stop9904ms，RestorePrepared/revision6、原plan/阶段/原stage身份吻合，reopened825ms、required及全部门槛正确；该失败未再现，不把新增诊断或一次成功记作原失败根因已消除。两轮原30/5/10秒保持，第2轮Stop贴近预算，实际apply后仍须满足原总预算，不能分配新10秒。

两轮实际catalog/cache/audit均未恢复，完整原断言保持exit101/accepted=false；没有把恢复门槛/计划准备通过冒客户端还原通过，未用原件JSON/单PID退出签发恢复许可。

## 证据、清理及后继

.adg/evidence/0.2.24/recovery-readonly-view-r1/保留两轮独立原索引/phase/plan/stage、原失败/最终输出、源码与binary摘要、锁探针输出、manifest及cleanup。第二轮实际源码与最终30件提交逐项SHA匹配；首轮及单测的差异仅后继诊断/accessor，原时点来源明确。仅本轮合成配置，不留runtime secret/admin token。

完整Darwin TCP表framing核对两原主端点缺席，自有测试/镜像为空；两轮精确新N/K/history数据库分别排他取得/释放后删除，副本程序/profile/测试binary/索引/stage及核对后的重复证据根均清理。未改日常配置/Shell/正式安装/官方2.99，无模型/WebDAV业务请求，未构建新.app/DMG或发布。

继续实际客户端恢复及完整集合/监听/写者权限、显式中断恢复/回退和全树/通用图/首次目录/跨卷/flags。revision4首轮未归因失败保持待追踪，不靠重跑凑通过。WindowsW0 HOME/Profile/W1/W2、NAME、npm System平台代理、官方新交付发布验证和固定候选仍按IMP-34推进，目标不关闭。
