# 恢复计划登记的同事务来源认证 r1

revision: 1；2026-11-13；IMP-34 P24-04；main c3f2fc97dd7f323be98b0c6430d920deb167db58；accepted=false，非固定候选。承接 [连续目录取得](client-descendant-pin-r1.md)。本批没有原包服务轮次，不宣称正常 Stop10 已通过。

## 实现与验收边界

root-aware record_restore_plan 原先在 write 事务外完整 registered_root_originals，再打开计划并核验索引引用。现在复用已有私有 write_checked_admitted：同事务完整来源审计和 exact Head 比对成功之后、首次 SQL 之前，以同次借用的 Originals 执行原 Pending::open 与 require_indexed_references。计划打开的完整帧链和引用核验保持；候选写入后与 COMMIT 后的完整来源／索引读回保持。没有把 Originals 或其它可复用凭据经回调输出。

根来源仍只准 actual Stopped 前缀，不能以 Ready／缺阶段或改 live 引用进入正常恢复。普通来源及 Ready／StopPrepared 的既有 recovery-source 分支逻辑保持；此次不实现根中断恢复。提交前来源或准入失败沿原事务回滚，提交后来源读回失败保留 RestorePrepared，不伪造已恢复，也不删除计划或原件。

登记仅发布独立引用，不是 Native Stop／恢复／应用许可。候选／提交后原机制核验的是来源与索引事实，并不保证计划文件随后永远不变；实际恢复入口重新打开计划和完整核验的边界保持。本批专门验证登记后篡改计划时重新打开失败，四份原件阶段帧字节不变，不能把该合成检查冒称实际 native apply 已执行。

## 测试与收益

四项新增测试覆盖首次及第二代合成 Stopped 来源、陈旧 source／错误阶段／live 引用改绑／缺阶段／错误摘要／缺计划与准入中断的 SQL 前拒绝；SQL 前、候选写入后、提交后三个边界分别篡改旧帧、当前帧和 namespace，仍拒绝并保留正确的存储阶段。另覆盖后继计划改动不被旧登记结果认证。夹具不构造 typed native Stop proof。

最终新增 4 项、索引回归 28 项、版本化命令 4 项，共 36 项不同测试通过；索引 2 项和命令 1 项原有忽略保持。all-targets Clippy -D warnings、fmt、diff-check 通过。本批只跑受影响索引／命令及新增用例，未重复上轮整个原件模块回归。新增断言曾因 RestorePlan 故意没有 Debug 而编译失败，保留该轮；改用相同相等条件的 assert 后通过，未为日志新增产品 Debug 或降低断言。

四个独立同形第二代 Stopped 夹具按旧／新／新／旧测量：旧参考复现事务外完整来源／计划准入后写相同 Head。完整历史观察 4→3，双遍历史内容审计 8→6；耗时 592.087／465.477／462.552／589.709ms。测量不含夹具建立及额外读回，不从局部计量推断真实服务停止预算已满足。

证据 [.adg/evidence/0.2.24/client-restore-registration-r1](../../../.adg/evidence/0.2.24/client-restore-registration-r1/manifest.json) 含开跑前假设、所有命令及压缩原日志、编译失败说明、四个变更源文件及测试二进制摘要、独立重验脚本、清理回执。unit TMP 实际为空，原字节归档／解压逐字核对后删除本批临时根。无日常配置、Shell、已安装 app、官方 2.99 原件、真实 WebDAV 或模型请求变更。

## 后继

原包上一轮第二 Stop12143ms／RootRetirementPrepared17 的失败保持。后续有界审查完整只读帧前缀内部重复的 root provenance／namespace 观察：先证明帧不在完整前缀返回前外泄，保留每帧摘要、归档、保护以及首末与最终 namespace 边界，再决定能否实施，不能先删检查或重跑旧源。实际 mutation／当前归属与原 deadline 不变。根中断／缺 Ready／一般客户端图路径与完整进程集合／刷新／NAME／复杂代理／Windows W1W2／交付验证与固定候选等 IMP-34 原范围仍待完成，目标 active，未发布。
