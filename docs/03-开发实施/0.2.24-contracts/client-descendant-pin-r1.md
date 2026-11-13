# 连续只读私有目录取得 r1

revision: 1；2026-11-13；IMP-34 P24-04；main acd9b871ee08722024c1d5b58fcdc13bc3b4f68c；accepted=false，非固定候选。承接 [根退役登记事务](client-retirement-transaction-r1.md)，本批不运行官方服务，不宣称 Stop10 已通过。

## 取得边界

新增封闭的 descendant_private，只用于取得已有目录，不创建、不读取内容、不外泄中间 pin。起始目录保持原 private 检查；每一级以已持有的父目录 FD 禁止跟随符号链接地打开，保存 identity 和完整 Protection，并通过原 require_private 后才继续。最后双遍核对完整路径／FD 与权限、ACL、flags、xattr，再对保存的叶节点属性断言私有性，末轮完整重验叶节点的直接父目录。这个父目录继承原期望和句柄，不按路径重新采样，也包含全部中间祖先。

仅替换三段连续只读取得：Originals 的 manager → client-originals → attempt 中后两级；历史归档的 data → manager-state → client-originals → attempt；root origin 的 manager → client-root-generations → id。Originals 的 manager 来源／pending 检查、generation namespace 枚举及前后检查、所有文件／帧／来源摘要认证和读写边界原位保留。创建目录仍走原 child_private，未改 SQLite 文件 FD、原生归属或 deadline。

这保留的是相同期望、内容读取前完整准入和最终 parent 边界，不声称与旧逐级扫描具有完全相同的并发拒绝轨迹。被删去的中途扫描时刻若恰好碰到又恢复的短暂属性变化，观察结果可能不同；取得期间没有内容读取、写入或中间结果发布。后续每次内容读取自己的完整链重验保持。只读审查针对实际 helper 与三处调用未发现阻断遗漏；不把该审查当测试执行。

## 验证与收益

- 新增取得测试覆盖完整 pin 在父对象销毁后保持有效、逐级／union 后直接 parent 的替换／符号链接／模式／xattr 变化拒绝、不创建、不修复、非法组件及后续调用重新观察。
- 初轮反例夹具以 base.join(空串) 保留尾斜杠，符号链接注入 syscall 返回 ENOENT；该轮 2 过／1 失败原日志保留。改为无尾斜杠 base.clone 后，原变更矩阵和拒绝断言不变，三项通过。
- 单次原件模块完整回归 290 项通过、9 项原有忽略，覆盖归档、ACL/xattr、外部进程与崩溃、恢复、历史、来源和 namespace。版本化命令另 4 项通过、1 项原有原包测试忽略；不同通过测试共 294 项，不重复计入先行三项。all-targets Clippy -D warnings、fmt、diff-check 通过。

三层纯取得的属性读取从 156→56。四个独立同形二代夹具按旧／新／新／旧计量完整退役登记，属性读取 31030／29423／29423／31030，耗时 360.617／343.937／346.187／369.110ms。每次完整历史观察仍 3 次、双遍内容审计仍 6 遍；frame、capture_file、plan load、root provenance 次数也逐项保持。完整登记收益约 5%，不能用局部取得的较大降幅代替，也不足以单独判断真实停止预算已满足。

证据 [.adg/evidence/0.2.24/client-descendant-pin-r1](../../../.adg/evidence/0.2.24/client-descendant-pin-r1/manifest.json) 包含开跑前假设、全部命令和压缩原日志、失败说明、实际计量、六个变更源文件与测试二进制摘要、只读审阅边界及可重验脚本。unit TMP 实际为空，原字节解压逐字比较后删除本批临时根；未改日常配置／Shell／已安装 app／官方 2.99，未跑真实 WebDAV 或模型请求。

## 下一步

原包上一轮第二 Stop12143ms／RootRetirementPrepared17 的失败保持，本批没有新原包轮或新 app／DMG／Windows 候选。下一处有界核查是 root-aware record_restore_plan：事务外 registered_root_originals／计划准入与事务内完整 source 审计是否重复，可否按已有私有 write_checked_admitted 消费同次完整 Originals；先测量和保留全部前后边界，不直接盲跑原包。record_first_root_phase 已没有另一次事务外 full Head，原生 proof 的提交前后检查不能删除。

同时通过远端精确 ls-remote 确认 Windows 代码 b2e89665d 与文档 0eef59a01 暂无新推送；不推断对端未开展本地工作。W1/W2 和 IMP-34 根中断／缺 Ready／一般客户端与完整进程图／刷新／NAME／复杂代理／交付验证／固定候选等原范围保持，目标 active，未发布。
