# r7并行异步协作：同源查询与严格恢复附件

用户本轮明确继续Mac/Windows并行异步，沿独占写集开发验证推送，缺接口仅停依赖项；主线自行接入。目标已建立且保持active，不以本批局部通过关闭Windows或发布目标。

源码分两笔：a7e60c347f85ed35a2193e1001f6b7ec9523022a拆出实际安全投影查询供轻量宿主复用；2f3b67b4169a587e3cf91c43ac014370b11b2b1d实现windows-before-resume-recovery-v1严格DATA附件与负例。随后自行fetch发现main462e2476a9275cdf82e9e3235ed84820ac2c28c7，核对仅运行包完整校验文件，无Windows写集交叉，合入并推送285989b2a0401c5db7ac1aa1469c8c0ffd9c0ee5。代码分支codex/024-windows-native，证据仍docs/024-windows-dev分离。

## 假设、结果和首错

- 完整产品MSVC内存不足是否应阻断全部工作：否。轻量MSVC宿主复用实际windows_official_runtime024、hash.rs及原生8投影查询源码，query02三项通过；不改机器策略/页面文件/日常进程，不在相同前提重跑完整构建。
- 查询逻辑拆分是否改变安全断言：query-equivalence.json核对逻辑tokens仅去注释、可见性、空白及rustfmt尾逗号后相等，原BoundedAccessV1能力实现字节完全不变。所含8投影/auditExcludedUnknown/fullfalse/普通令牌/原截止、原句柄保持；不从JSON构造原生能力。
- beforeResume附件是否严格拒绝未知/旧格式：首轮query-schema04检测Serde空unit事件接受未知字段，真实5通过1失败/raw101保留；改严格空结构事件后query-schema05六项通过，补齐事件缺字段/unknown/重复node/null/overflow负例后query-schema06七项通过/raw0。它只核DATA字段与链，不是native PreparedAbort/QuiescentLauncher/AbortRestored工厂。
- 新主线是否可直接接入：合入462e2476a无冲突，受影响的只读运行包合成校验8项通过/raw0。未执行官方代码、W2或发布，不把Mac候选单测当Windows生产通过。

本轮首错query01缺实际hash.rs测试使用的hmac依赖（E0432/raw101），仅在test独立Cargo清单补依赖后query02过；后续query-schema03因外部path模块的子测试定位E0583/raw101，显式固定测试path后继续。全部原始stdout/stderr/result保留，未改原断言/超时绕过失败。早期query01..05仅保原工具日志，不声称逐次源码已完整快照；query-schema06通过可复现driver保存25份实际执行源码，工作字节及producer Git仅CRLF等价逐份核。实际TEST EXE摘要cff4fb1638c198a0f02d4d920feeaae076844cb043de3674820c7d401b0bed5b，非产品/发布制品。

## 异步请求和当前依赖

93d4630c07b00b0ec75164a49db4059901975f30已通过现有windows-dev-handoff交blocking-request.json；7c98aeea2ff650b2ae01c02109ba6de7c68e6483先推before-resume-schema-plan.md，再实现。已交付不等于Mac接收/决定。请求Mac明确已加载357旧same-image没有只读Status客户端时原Job当前历史的可采边界；旧creator total1/active1不能自行冒作当前Job事实，新image不得绕过exact-image认证。缺此决定仅阻断当前R1执行就绪，不阻断独立查询/严格附件/后续原生接口与扫描开发。

严格附件选择r7允许的v3原节点事件＋类型明确新附件，未改旧v3 union/current或将seq9转换Closed/Aborted。固定failed session/action/source/policy/prefix与fresh recovery action/QPC；所有可空字段仍须显式出现。链拒绝缺字段、未知variant/字段、重复字段、旧schema、非法UUID/hash/QPC、超过原10秒、跨scope、重放/跳序、未quiesce动作、错误ID/parent或intent/result错配，以及normalStop/closed/publicStart=true。原生全树、实际锁、线程/Job、端点静止和动作窗口必须由未来native工厂独立证明；记录合法不会授予这些事实。

## 保持与未完

完整13项状态见status.json。当前r2全归档/实际native预检未运行，精确修复计划尚不具执行条件；不复用旧r1 ZIP或50580/54692授权，未请求新的现场force/restore批准。21500/75744、原custody/三原件/三work、旧seq9与40588均不动。自有原生测试tempfile场景由测试拥有者清理，失败日志、源码快照和TEST EXE保留；没有新增真实原包轮或持久测试进程。

后续继续partial anchor与原控制的capture前安装、fresh恢复origin/原Job/整树返还接线及同一次inventory计量优化；完整产品构建的旧OOM缺口保持，原预算30/5/10/Restart30含Stop10、未知拒绝与公开Start关闭保持。已过且不受影响的GUI/CLI/MSI与旧扫描计量复用证据，不重复空跑，不改官方2.99/日常PATH或配置/策略，不业务模型请求、不W2/双SID、不发布。
