# 捕获前持久归属预留与部分原件保全

revision: 1；2026-11-12。归IMP-34 P24-04。后端main47ad91a93、界面main562d655ef。承接[历史原件](client-vault-history-r1.md)、[捕获前取消](client-prepare-cancel-r1.md)和[Stop成本](stop-fresh-prefix-r1.md)。accepted=false、非冻结候选；无预留旧残留不认领，不开发迁移。

## 写入前登记

原先运行时随机生成attempt，Original/scope发布后才登记refs，捕获中断可能留下无归属目录。新生产流程先reserve_start Preparing，再独立synchronous FULL/CAS提交CaptureReserved，沿原ticket作为唯一attempt；事务完成前不写attempt目录/blob。CaptureReservation不可序列化，由同一Index借用，保持原action锁，每次require重读完整head且校验D/Codex/Profile路径和索引/目录安全事实；错根/旧revision/已转换头均拒绝。

生产只调用StartupSession.begin_reserved；无预留便利begin限制cfg(test)。以预留ticket mkdirat EXCL，已有目录不重用、不覆盖或收养；创建目录、Codex归档、Grok归档、Original发布边界重核reservation。完整捕获/scope通过后，CaptureReserved→Captured必须refs.attempt等于ticket；然后才可官方Start。新正常生产事件为Preparing1→CaptureReserved2→Captured3→Ready4→StopPrepared5→Stopped6→RestorePrepared7→Restored8。额外事务只占原Start30秒；Restart/重复会话测试真实计数相应Ready12/最终Restored16，不降低断言。

## 保全取消的证明边界

只有独立CaptureReserved允许本次精确ticket槽位存在。先按原索引闭合历史核对旧集合，再加唯一预留槽，未知另槽/文件、错ticket、链接/硬链接/不可读或变更树/超限拒绝。槽中即使有完整Original或scope也不生成References，不解析为恢复来源；只用有界非协调TreeSnapshot记录整个保留目录原生身份、完整内容摘要/成员及权限ACL/xattr/flags。它是当前保全证据，不证明内容能恢复。

取消不删除/搬动/回写客户端或部分原件，仍须原完整官方镜像、当前OS停止/原runtime-pid记录缺席、当前Codex/Grok稳定和原10秒、私有create-only回执/不可序列化Proof/CAS。独立终态CancelledCapture，refs仍None，不能伪装CancelledBeforeCapture/Unchanged/Restored。回执绑定原CaptureReserved、历史摘要、当前两树和可选预留槽快照；关闭后及下一明确Start核对保留部分证据。历史只从已登记取消receipt/原事件prefix取得槽身份，核对ticket/revision/binding/原历史摘要，直接比对快照，不从扫描生成来源引用。部分证据改变/丢失仍Unknown/拒绝。下一明确Start另ticket、正常捕获；旧部分证据保留。

产品cancelPreparation支持capture_reserved，其他能力互斥、not_recorded/phaseCount0和原运行/更新/binding门保持。文案“原件捕获已中断，服务尚未启动”，确认“保留已捕获文件，再取消本次启动”，明确不恢复/删除或自动启动。成功只接cancelled_capture+同attempt/revision+1，并由新live同D/H/Stopped/NotRequired核对才清限制；捕获前取消回执不能代用。成功“本次启动已取消，已捕获文件保留”，不声称原件完整。唯一总原型同步capture/slow/error内存场景，无文件/API/持久化，总index/其它原型未改。

## 检查与真实链

77去重后端：index27、session26、originals23、historyParser1；reservation先于文件、错head/root/旧预约/重复目录/错refs拒绝。实际子进程exit91覆盖directory_created/codex_archived/grok_archived/original_published四边界；重开均CaptureReserved无Original refs，取消保留精确部分树，未知另槽拒绝、取消后改证据Unknown、下一明确准备另ticket。初check重复enum pattern warning已修；最终格式/编译/全目标strictClippy过。

73前端/类型/生产build通过，部分捕获不调finishUnchanged、不同取消状态不能代用，原关闭重开/状态重试保持。WebKit/Chromium各10软件/唯一原型场景过（明暗1180×760/900×600、确认/返回、slow关闭重开1mutation、失败限制及原prepare），pageerror空；实际查看暗色和真正原型确认截图，生产排除DEV handle/固定UUID/mock文本，含专用文案与状态。

新隔离Profile供应未改官方2.99，Codex归档后注入中断：Start2279ms，两个blob保留、Original未发布、服务未启动/recordnull，原exit101/index2 CaptureReserved/refsnull保留。独立取消错revision/digest、未知另槽和synthetic runtime记录均拒绝索引不变/注入内容保留；自行清理合成注入后正例3163ms，原10秒内/index3 CancelledCapture，真实新状态NotRequired/仅Start。独立全树SHA匹配取消receipt、receipt摘要匹配index3；两个blob字节/inode/mode与取消前完全一致。

随后同现场另一次用户明确普通Start/Stop（不是取消后自动动作）：Start9441/Stop9455ms（入口Stop9448），原30/10内；新attempt等于新预留ticket且不同于旧捕获。完整原件/阶段/live/plan/receipt和index11 Restored、父子normal0/捕获TCP缺席、全客户端安全属性/两锁释放通过；7原文件字节/缺席还原。另进程完整Codex/Grok树SHA匹配新receipt，真实NotRequired/仅Start；两个旧部分blob在后续运行停止后仍字节/inode/mode完全保持。仅此样本不冒一般大目录或最终候选预算稳妥。

原中断/取消用build-native及native/source.json；后加明确followup测试helper另build、准确源/binary独立记录；最后仅加cfg(test)限制无预留begin API，生产原已只用reserved，最终Clippy另证，不冒早期binary包含后加helper/注解。未改官方、预算或完整性断言，无新.app/DMG/固定候选GUI→IPC整链。

证据[本批目录](../../../.adg/evidence/0.2.24/client-capture-reservation-r1/)及manifest保存index2/3/11和11events、2部分文件、完整新attempt四帧/35内容引用/15blob、各阶段准确source/binary、独立全树/真实状态/截图。OS sandbox拒绝日常HOME/外网，仅loopback，无真实模型或业务请求。自有服务/launcher/测试进程与原端点缺席，3精确新协调DB排他重获释放后和隔离程序/Profile/所有夹具清理；globalL inode/字节/权限/owner保持、排他重获释放、无sidecar。unit TMPDIR空、browser/Vite退出，归档逐字核对后清重复临时根，target/dist保留，不发布。

下一步有独立原件但早期客户端已改变、尚未形成plan的恢复；首次缺根/一般客户端与路由图/完整进程集合/旧revision4首错、Windows贡献集成/NAME/复杂System代理/官方验证/README和固定候选/W2继续。无预留旧残留仍拒绝且不做自动迁移；Windows远端b2e89665d/0eef59a01无新贡献，目标active。
