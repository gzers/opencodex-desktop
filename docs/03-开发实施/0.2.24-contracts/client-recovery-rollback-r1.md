# 恢复回退终态与原对象重新准备

revision: 1；2026-11-12。归IMP-34 P24-04，承接[显式继续](client-recovery-continue-r1.md)。后端main ea78757ea、界面状态main3de2c6370。accepted=false，非冻结候选。本批完成原生回退执行/独立终态及重新准备核心和真实隔离验证，尚未注册rollback/rearm产品IPC或开放相应确认按钮；不能宣称用户已可在软件完成全部回退流程。

## 两类完成证明

Complete仍要求启动前原件和完整原树；Rollback仅把原计划的3件返回冻结的Stopped后像，绝不重新拍基线或声称启动前原件已恢复。复用原同进程NativeLease/当前原service与launcher/port/manager及记录缺席、批次前后完整OS枚举、每次publication原生身份/内容/保护和冲突保全。新原生rollback.review/rollback执行器只接原attempt/revision/digest，另起原10秒动作，不重造旧正常退出证明，不强杀。

回退后必须Pending整组终态确为Rollback、3件目标/原暂存/归档匹配原Stopped代次；完整Codex树中非这3件的原内容/目录身份/安全属性、Grok完整原件、两稳定锁身份与真正EXCLUSIVE重获/正常释放通过，读回稳定后才发布rollback-readback原生回执。独立ReadbackProof携带不可序列化decision，Index.close_restored拒绝Rollback证明，close_rolled_back拒绝Complete证明。回执、plan、original/scope/live引用与独立索引摘要全部匹配才登记RestoreRolledBack；原索引schema3新增阶段，无目录/旧版迁移或新baseline fallback。

观察者只读核验已登记的rollback回执及固定decision，不把锁存在当持锁，也不因文件存在当Stopped。该阶段对外仍Pending/Required，reserve_start及普通Failed转移都拒绝，不清理原refs、不当NotRequired。界面/原型只新增准确状态：“本次恢复已撤销，文件已回到恢复前的停止状态”“启动前原件仍保留，尚未恢复；服务保持停止”。本批不露回退/重准备按钮，避免只接半条流程。

## 不丢失重新恢复入口的原生核心

新rearm.review/prepare在原既有索引排他锁/当前完整官方准入与原生协调下核对已登记Rollback、当前精确原文件对象和完整余项，生成NEW逻辑计划，而不复制/移动/发现新客户端文件。schema2派生plan只存新id/attempt及父plan的独立id+SHA；父链逐项核对私有原文/摘要/同attempt及不可变Rollback终态，最多16层。schema1仍是当前版本的初始固定计划，不是旧软件路径或旧运行包兼容。

派生计划沿用父链最初的物理暂存/归档位置及原inode、内容和保护。每次Pending重新载入都验证该原对象链；原回退outcome不可改成Complete，新intent/outcome归新逻辑id。派生计划的完成/回退回执用新逻辑id区分，旧回执保留。父链中的旧回执证明过去事实，不要求历史文件现在仍位于当时位置，但新执行必须核对其原生对象的当前计划状态；不能靠旧成功布尔签发写权限。

准备过程只create-only发布派生元数据，无客户端mutation。若在新索引提交前中断，只留下未登记计划，不出现新客户端stage或未知inode；旧已登记回退仍可核对。重试可生成另一新逻辑id，绝不扫描或收养未登记计划。Index.rearm_rolled_back必须核验原已登记回退、当前头和新plan精确parent，append事件后回RestorePrepared、保留原refs/旧events，仍禁止启动。实际恢复是随后独立用户确认的Continue；准备动作返回restore_prepared，不能记成已恢复或为一次动作续预算。

## 验证与来源边界

- 75去重后端函数：session22、index23、completion6、restore24。新增回退完整原生读回/错decision不能关闭另一终态/坏回执拒绝不覆盖/外部修改保留/旧revision拒绝/原refs保留/准备后继续到Restored、未登记plan不收养和旧回退原文保持。实际子进程exit91现在覆盖普通与派生计划各11个发布/归档/终态中断边界，按同原计划重开核对/继续。不可把这些合成引用中的PID当真实服务退出或生产权限。
- 新测试首编译因References无Debug被assert_eq要求而失败，terminal-rearm.log保留，仅改测试比较；未放宽产品判断。最终格式、编译及全目标严格Clippy通过。前端28项、类型/生产build、WebKit软件和唯一总原型2个只读回退状态场景通过并检查截图，无执行按钮或错误解除限制。
- 一个全新隔离Profile运行原样官方2.99，精确Start/Status/Stop耗时见native/native/summary.json。原服务与父normal0/4捕获TCP缺席后，在第1目标原生交换后按预设中断，原完整assert exit101/index6/Required保留，不冒普通Stop通过。
- 同保留现场新进程rollback：错revision/digest与外部config编辑拒绝，原索引不变/外部字节保留；实际回退8130ms，独立终态rolled_back/index7。重复旧确认拒绝；真实OfficialStatusSource重开Required，PID/port均无、canStart/Stop/Restart全false。
- 不重装/不重新启动服务，同现场新原生rearm动作9557ms，index7→8 RestorePrepared；随后另一明确确认调用Continue9434ms，index8→9 Restored。每次动作各原10秒，没有把两次预算拼成一次动作或自动继续。准备和恢复输出分别保留，不以整个测试wall时间判单次预算。
- 两阶段分别用独立只读native完整Codex/Grok快照核对其回执摘要；保存index6、index7+7events、index9+9events、旧回退与新逻辑plan/完成回执。最终旧rollback回执逐字未变且摘要匹配原index7；派生完成回执摘要匹配index9。原4帧/35内容引用/15blob与最终7客户端文件原字节/缺席均独立核对，真实最终状态NotRequired，仅canStart为true。

证据[本批目录](../../../.adg/evidence/0.2.24/client-recovery-rollback-r1/)及manifest分别记录初始服务源、回退构建、派生计划构建与最终新增只读helper源/可执行SHA；不把后加源码冒为早期binary。受OS sandbox限制，外网和日常HOME读取拒绝，只允许loopback，无真实模型/业务请求。产品官方包未改，无新.app/DMG或候选发布。

自有进程和原主TCP端口缺席，3精确新协调DB实际EXCLUSIVE重获/释放后和自有程序/Profile/所有原件/stage/archive/测试副本清理；既有global L的inode/字节/模式/owner保持、排他重获释放，无sidecar。unit TMPDIR为空；browser/Vite均退出；归档按字节/SHA核对后删除重复临时根，正常工程target/dist保留。

## 继续项

下一步将已验证rollback/rearm核心接产品IPC、各自确认文案/能力/摘要、执行中关闭重开、失败保留及重新准备后明确Continue，保持原服务与原件恢复两种完成的区别。不得把本原生核心或只读状态UI标成整个交互完成。更早Preparing/Captured等阶段、首次缺根/一般客户端与路由图/完整进程集合/旧revision4首错、Windows贡献与集成、NAME、复杂System代理、官方发布验证/README及固定候选/W2继续，整版目标active；本批未查证新Windows回交，不更新其完成事实。
