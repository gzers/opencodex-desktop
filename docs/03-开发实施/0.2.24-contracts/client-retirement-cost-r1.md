# 根退役登记分段计量 r1

revision: 1；2026-11-13；IMP-34 P24-04；main264934c0936a1fab39e00d36c228b4813573b24b；accepted=false，非固定候选。承接 [累计优化原包失败](client-cumulative-native-r1.md)。

本批只增加测试计量，不做性能修复，不改变产品顺序／错误／预算。分开统计 storage+BEGIN、完整来源、计划准入、SQL、完整候选、COMMIT、完整提交后读回及最终登记事实；计划 live 校验内部另统计已认证帧前缀与当前Tree/Grok。顶层区段互斥，嵌套指标不能再次相加。

| 完整 Stopped 合成来源 | 第一代 | 第二代 |
|---|---:|---:|
| 登记总计 | 176.665ms | 538.924ms |
| 来源认证 | 33.438ms | 123.497ms |
| 计划准入 | 27.796ms | 56.493ms |
| 候选完整读回 | 56.205ms | 175.556ms |
| 提交后完整读回 | 55.718ms | 176.466ms |
| SQL+COMMIT | 0.698ms | 0.789ms |
| 其余顶层（存储／事实等） | 2.810ms | 6.122ms |

分段覆盖总计超过99.9%，未归属分别33.206／32.249us。第二代嵌套 live-prefix 三次合计157.157ms，Tree/Grok三次合计3.038ms，包含在上述来源／准入／读回中；完整历史观察仍3次、双遍内容审计仍6遍。先行只含 Captured 的夹具结果也保留，不能用其较少帧的347.687ms代替最终三phase来源。合成来源没有真实官方服务、writer marker及整份运行现场，不能把这个表当原包耗时或据此删除读回。

该表把下一步聚焦于完整来源／候选／提交后文件认证，而非SQLite写入。root_binding 的 movement 分支包含初次 Originals 打开和 Pending.inspect_live 的后续完整前缀；是否能在同一完整认证边界内消除重复，仍需逐项核对中间来源失效时的archive前拒绝、计划／帧／namespace及恢复分支，未授权跳过source/candidate/postcommit三边界或每帧门槛。

## 原包诊断入口

仅cfg(test)构建在 `OCXD024_RETIREMENT_COST_PROFILE=1` 且数据根确属 `/private/tmp/ocxd024-versioned-lifecycle-*` 时，才对实际 root registration 调用启用上述计数并输出 ROOT_REGISTRATION_COST。输出仅计数、耗时、是否成功，不含路径、配置、认证信息或原生归属材料。普通产品调用仍执行同一登记一次；诊断本身不提高deadline、不放宽断言。

下一轮应在固定源码／实际release二进制上开启该诊断，直接证伪“真实candidate/postcommit完整认证及live-prefix主导最后预算”的判断，完整保留两轮生命周期状态与失败；不能从本表直接宣称真实根因已全确认。这个后继诊断尚未执行。

## 验证与清理

1项完整Stopped两代计量、4项事务篡改／错误来源／提交后留存回归、4项命令回归，共9项不同测试通过，命令1项原有原包用例忽略保持；严格all-targets Clippy、fmt与diff-check通过。无必要重跑上轮完整原件模块。证据 [.adg/evidence/0.2.24/client-retirement-cost-r1](../../../.adg/evidence/0.2.24/client-retirement-cost-r1/manifest.json) 包括开跑前假设、全部压缩原日志、顶层/嵌套计量、五源摘要及测试二进制摘要、后继原包诊断假设和独立重验脚本。unit TMP为空，证据逐字归档后删除本批临时根，工程target保留。

原第二Stop10888ms/RootRetirementPrepared17失败保持。本轮无官方服务／日常配置／Shell／已安装app／业务请求／官方包变更或新候选。根中断、缺Ready、一般客户端图路径／完整进程集合与刷新、NAME、复杂代理、WindowsW1W2、交付验证和固定集成候选等IMP34全范围不缩减，目标active，未发布。
