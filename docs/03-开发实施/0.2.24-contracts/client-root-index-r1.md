# 整根归档独立登记与缺根状态核验

revision: 2；2026-11-13；IMP-34 P24-04；main 7131fdadc。承接[整根归档原语](client-root-retirement-r1.md)。accepted=false，非冻结候选。完成存储与独立索引接线；生产移动许可、缺根启停及下一次Start未完成。

## 独立阶段与当前证明

record_root_retirement在当前独立source为Captured/Stopped/RestorePrepared/RecoveryPrepared时核对已登记原根Absent、准确Original与不可变movement计划及当前完整执行树，CAS登记RootRetirementPrepared，保存sourceRevision与plan id/SHA，不更改原件/恢复计划。示例Captured3→Prepared4。普通Failed等转移不能丢弃原movement引用，普通既有根binding不能声明原根Absent。

close_root_absence只接受本次apply生成的AuthorizedReadback，不能从磁盘JSON构造；它绑定原source、plan、data/profile identity、Retire方向和原deadline。apply采用caller和permit两期限的较早者；提交前后都核对deadline，提交后超时保留已写事件并转回Prepared，不以“曾有回执”续期。完成CAS示例Absent5，保留原source、Original/计划和当前完整归档读回摘要。schema版本仍3、原三表不变，仅Head增加可选字段；不开发旧版迁移。

RootAbsent是存储终态事实，当前status仍Required，preview仍Pending，无普通继续/收尾/回退/取消/Start权限；不把它当Restored或Stopped。不存在生产MutationPermit factory，test-only fixture_apply_for_index使用合成权限，只证明目录/账本行为。没有放宽Ready/capture guard。

## 缺根核验与路径来源

受保护独立SQLite先打开，严格核对RootBinding/初始事件、当前Head及准确source event；source/refs/原恢复计划必须与不可变movement计划逐字段相同。原profile/.codex缺席才解析该attempt下固定root-retired槽，按原RootBinding/Original/scope/完整tree/Protection/方向intent核对物理对象；logical scope始终原路径，不因物理槽位在data vault内而误拒，也不放开任意物理路径。原逻辑根存在则Prepared只能读回准确live tree，Absent遇新用户根拒绝，不覆盖。

只读核验不创建root/索引或补封历史。writable reopen先事实定位，取得排他action锁后重复完整核验；调用方传任意missing路径不能被当成profile/.codex。投影结束再次核对scope、锁代次和原逻辑根仍缺席。终态按独立completionSHA核对原Retire readback及当前准确树，磁盘readback存在但未独立关闭只能Prepared。错source、篡改readback或新用户根均保持失败/原对象。

软件与唯一原型增加两种固定内存场景：客户端目录恢复待核对、原目录缺席已核对；复用现有只读弹窗与诊断详情，只有重新检查/取消。矛盾advertised flags也不能触发普通变更或native调用。

## 验证与证据

[证据及manifest](../../../.adg/evidence/0.2.24/client-root-index-r1/)104文件；84个去重相关后端函数通过（indexed6、普通index27及拒降级1、registration6、capture9、session26、retirement8及deadline1），原session两项ignored不计通过。88项前端、build、fmt/diff-check和最终all-targets严格Clippy通过；WebKit/Chromium各6检查，无pageerror/变更调用，浏览器与Vite已退出。实际查看软件小窗深色与原型截图。

三个真实child exit91边界：plan_registered为Prepared4/根在，root_moved为Prepared4/根缺席，root_closed为Absent5/根缺席/独立completion绑定；父实际wait91。正例及三个中断按原独立SQLite/Original/plan重开，账本字节不改，无新Start。offline-byte-projections与verify_archive.py逐字校验事件1..4/5、source3、RootBinding/schema5 Original/scope/plan及已关闭读回，副本不作活体身份或服务证书。

保留初轮2过2失败（overlap，chat原输出而非独立log）、二轮2过4失败（投影尾部physical scope，原log）、Clippy冗余模式、漏测试permit deadline字段编译失败；修正后定向通过。较后origin拒降级和期限专项有各自日志，不把较早执行冒称最终源码。source-final固定提交/逐文件与最终test binary SHA。

归档逐字核验后删除自有重复临时根，unitTMP空；测试child/浏览器/Vite均结束，无官方运行包/服务、全局协调DB或日常配置写入，无真实WebDAV/模型请求，无新.app/DMG/发布。

## 后继

真实NativeLease仍绑定逻辑路径，官方writer holders也在逻辑config路径检查/释放；不能去掉校验来让目录移动通过。下一批要把当前正常service/launcher退出、TCP缺席、两官方writer排他与稳定锁释放、SQLite/claim原身份保全接入重定位权限，在原动作预算中完成根移动及独立关闭，再接下一次Start与明确中断操作。完整原包2.99隔离原生启停后才能宣称产品通过。

本批不新增Windows原生验证；W1独占贡献和最后固定候选W2仍按TEAM。其它IMP-34剩余范围与发布边界保持。


后继勘误：本批source-final中finalCompiledTestBinary误选应用测试宿主，不能作为库测试二进制摘要证明；原文件不改，准确范围及来源见[OFD后继合同的证据勘误](client-claim-ofd-r1.md)。原源码/测试日志/字节投影保留。
