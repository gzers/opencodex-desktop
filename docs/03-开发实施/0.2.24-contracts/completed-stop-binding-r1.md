# 0.2.24 完成停止结果的原上下文与预算交接

revision: 1；2026-11-12。归IMP-34 P24-04，承接macos-normal-exit-r1、native-client-coordination-r1及macos-client-restore-race-r1；代码main5a5732524558b3f72c7d24268935451a725379d3。当前是完整普通Stop结果的内部引用交接，尚未保存启动前原件关联记录或接入客户端恢复，accepted=false、非冻结候选。

## 当前实现

此前stop_lifecycle在完成官方Stop/OS证明后只返回()，退出证明和原运行记录身份随函数结束丢弃；后继恢复若只查询Stopped，就会混淆“本次确实停止了原服务”与“调用时已经停止”。本批提取stop_lifecycle_verified返回私有字段的CompletedLifecycleStop，原普通Stop包装入口调用它并验证后维持原有()及前端DTO/API，未改官方CLI/HTTP行为。

内部结果保留原ProcessContext运行句柄Arc、工作用户目录、OpenCodex HOME、规范Desktop数据根路径、checked_owned取得的原受保护runtime记录摘要，以及实际取得的Mac NormalExitProof和原有效停止截止时间。没有Serialize/Deserialize/Default或接受前端布尔授权的构造器。数据来自既有实际普通Stop检查，不新开过程或从当前晚到文件重新取基线。

- 只有本轮Owned→官方已验签Stop→原生退出/无运行残留等原有门全部完成，才带原record摘要；Mac带原内核exit0证明。
- 调用时已Stopped的分支保留正确的Stopped结果，但record摘要与新内核exit均None；不能用这个结果授权恢复另一轮startup snapshot。
- require_original同时核对同一runtime Arc、profile、HOME、数据根、原记录SHA256和原PID/uid/出生代次。错误引用、格式错误摘要、缺新退出证明均拒绝；相同文件路径不代表另一runtime句柄可冒用。
- 证明内deadline来自本次原stop_timeout与原action_deadline的最小窗口。后继验证一旦过原预算即lifecycle_timeout，不给文件恢复再开一个10秒窗口。普通包装入口本身也检查原预算/上下文，避免旧结果过期后仍被宣称成功。

Windows仍运行原既有native witness/停止检查，本批只改变共同函数结果结构和原数据根/预算验证，未接新官方Windows后端或替Windows执行原生验收。Mac-specific新退出事实仍由上轮kqueue门产生，本批未变该内核算法或原正常退出断言。

## 实测与证明范围

本轮4项引用交接测试和10项既有归属回归通过，最终编译/格式/全目标严格Clippy通过；原需显式Node路径的额外模拟服务项未执行，不计通过。新增检查验证：

1. 实际普通停止函数的AlreadyStopped分支不给新退出/原记录权限；此测试使用已有明确FakeHost，并未冒充真实活服务。
2. 真正自建gated子进程的内核exit0证明，配合显式合成外层结构只测试引用比较：正确引用可对比，不同runtime/profile/HOME/D/root记录或出生代次均拒绝。
3. 不合法原记录摘要、没有本轮内核退出的记录结果不能过原件引用检查。
4. 原证明预算已过即使有子进程真实exit0也拒绝，不能重置秒表。

这些测试不把手工构造的合成外层结构当产品Stop发行证书，也没有创建MutationPermit或写客户端文件。上轮官方2.99 Start/Status/Stop4433/1092/2392ms仍只对应main57c201bd7原轮；本次为结果交接和附加预算检查，不重复无影响供应/官方服务整轮，也不冒称旧证据已自动升级至当前完整候选。最终源码/binary摘要单独保留，后继完整恢复接线需自己的受影响原生实测。

## 留证与后继

证据.adg/evidence/0.2.24/completed-stop-binding-r1/包含本轮假设、编译/Clippy/14项结果、准确源/binary摘要、隔离边界、清理及manifest。测试仅自己的合成目录/进程/loopback监听，OS拒绝日常HOME读取、外网和夹具外写入；未操作真实服务、业务同步/模型、日常配置、官方包或正式安装。测试进程均结束，复制binary/私有目录与核对后的重复临时回执清理，未出新.app/DMG或发布。

**未完成项保持：** 还没有把启动前Originals与这个Stop结果持久绑定，没有完成自动Start/Stop客户端补偿。数据根在此结构只绑定规范路径；实际目录代次/原件及各phase/plan摘要必须由Originals原生保护并在后继编排中对上。单PID退出+同一上下文不是完整后代/监听集合或config/Grok/Agent/全树恢复许可，生产MutationPermit签发器仍关闭。下一步继续启动会话/原件引用的持久绑定，使用实际已验证normal stop结果而非Stopped布尔，再接三件及完整适用客户端恢复链，保留原30/5/10预算及未知拒绝。WindowsW0 HOME/Profile/W1/W2、NAME、System代理和发布/固定候选按IMP-34继续。
