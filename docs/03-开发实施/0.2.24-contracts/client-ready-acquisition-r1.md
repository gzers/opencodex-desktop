# 封闭Ready会话取得与恢复阶段诊断 r1

revision: 1；2026-11-13；main 4163514fe；IMP-34 P24-04；accepted=false，非原包或固定应用验收。

## Ready取得边界

Stop原Active::load在无外部回调和写入的连续构造中，分别在Index setup、COMMIT后、额外index.head和open_indexed.registered_root_originals做四次完整审查。新Index::open_ready_session保留setup和COMMIT后完整checked_head_originals，立即消费最后一次得到的Head/Originals构造Session，移除后两次仅为取得相同事实的重复观察。

该入口不接外部Head/Originals/callback，不暴露中间证明或可延后使用token；Ready状态仅在新入口限制。原公开open/open_existing仍完整观察后丢弃附带Originals，open_indexed共享构造没有新增Ready限制，普通canonical客户端仍走原验证。root缺已认证Originals绝不降级普通入口。

Session继续用调用方实际runtime context读取当前scope/live/program selection及Original/Ready，返回前核Index storage/action、完整Head、profile/HOME和全部Session references一致；不把持久Live的PID当作当前就绪。verify_current_ready、PreStop签名Ready、typed phase及后续事务/Guard/移动/absence的完整核验均保留，不复用本次取得结果跨操作。减少连续同观察内重复读取不声称捕获所有外部瞬态ABA。

ReadyAcquisitionError只为Rust内部区分Index IO和Session AppError，无serde/IPC字段：Active对Index映射原original-session failure，Session scope/live错误原样传递，避免优化改变错误码。初始创建的三个交接检查点保持，新增COMMIT后测试注入仅在无creation transfer的重新打开使用。

## 诊断保持结果与预算

测试构建输出固定阶段、区间微秒、同一个原deadline的开始/结束剩余预算、expired标记和预定义错误类别。覆盖load/preStop、RootStopAdmission scope/live/lease/pair/absence、restore_root_artifacts admission/guard-image/plan-open/authority/apply/结束检查，以及permit/writer错误映射前的底层类别。无路径、配置、密钥或原始错误detail输出。

正常生产构建不输出上述trace；包装闭包按原顺序执行、短路条件和结果原样返回，不增加/重置deadline。原permit/writer IO PermissionDenied和固定文案不变，仅在cfg(test)记录被映射的原AppError。trace不授予Ready/Stop或恢复权限。

## 验证与证据身份

新增4个Ready用例直接覆盖首代/第二代真实文件系统及合成程序选择/scope/live；计量显示第二代history before/after从4组到2组、frame50→26、Protection77774→40442，返回相同完整Ready Head/refs。是合成持久绑定观察，不是活体服务就绪或整体Stop减半。

取得事务完成后、末次audit之前篡改旧Original、receipt、binding、event或namespace均拒绝。SQLite篡改通过外部子进程，父不打开关闭辅助SQLite inode fd。字节篡改恢复0400权限，避免仅凭权限拒绝掩盖hash验证；不声称所有archive blob组合都新增覆盖。fakePID/live不能通过current Ready；load后改帧被下一Index/session读取拒绝。另2个结果测试分别验证timeout与仍有预算的binding拒绝能区分，以及writer映射仍是原IO类别/文案且不输出detail。

95个唯一检查函数通过：Ready4、其余Session29、index56（初套件55pass/1testhook契约失败，原断言不变修接线后精确复验该函数）、命令4、step结果1、writer映射1，8项ignore不计。最终all-target Clippy/fmt/diff通过。core六源binary完成主要回归，最终七源binary只增加writer诊断等价helper/test及限制reopen testhook；两个源/binary身份分别保留，差异精确校验，最终mapping/首根锁/Ready audit负例另测，不宣称一份binary跑完所有日志。

证据.adg/evidence/0.2.24/client-ready-acquisition-r1/保留假设、每份构建身份、compile失败、testhook失败、精确复验、计数与诊断结果。无运行官方runtime/service/全局协调DB、日常配置/Shell/安装app、真实WebDAV或模型请求；临时夹具归档核验后清理，工程target保留。

## 后续

前轮236ed43dd原包第二Stop10736ms、RestorePrepared16/client_original_restore_pending仍失败。下一原包轮须冻结实际release来源，用本批trace区分root-artifacts-held处是原deadline耗尽还是具体来源/Guard拒绝，核对load/preStop减少的历史观察成本；不从错误名或总耗时猜原因。原30/5/10及Restart30/Stop子预算不改，旧失败不覆盖。

根中断/动作回执容量边界、缺Ready、一般客户端图路径/完整进程集合和刷新、NAME/复杂代理/Windows W1W2/发行README与固定候选等IMP34完整目标保持；本批不发布。
