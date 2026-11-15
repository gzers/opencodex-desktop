# Windows r4 收口证据

日期：2026-11-15；代码最终提交：42e73081163d29df7977a12a84ffdc8fc0254072；基线 main：2ebdd3ae0；docs/governance-main：84ec482ee；合同：TEAM14 / IMP140 / windows-security-coverage-r4。

## 已完成

- 固定 windows-bounded-access-v1 八项投影及 0x1f7 合并查询，逐项 QuerySecurityAccessMask 均核到 READ_CONTROL=0x20000；审计 SACL 与未请求扩展保持 ExcludedUnknown，fullSecurityProven=false。
- v2 manifest、phase、current 的 policy digest、完整八项集合、未知字段/旧代次/PartialDiagnostic 降级拒绝和同对象 CAS 已定向验证。
- 原存在整树与原缺席 .codex/.grok 的同卷原对象保管、工作副本、精确 ID/字节/投影读回；外部新增/漂移拒绝并保留原件。canonical v2 tree manifest/current 与中断 pending 已验证。
- 产品 MSVC debug --bins 构建通过；制品摘要在 product01-artifacts.json。

## 未完成与门禁

资源属性、CAP、trust label、access filter 的非空原生正例及更深条件语义仍未验证；它们没有用空响应冒充通过。稳定锁释放、完整 ClientOriginals 生产 trait 接线、真实 GUI、完整 S/D 生产组合和 ClosedEligibility 仍未完成。Start 继续拒绝，未启动官方服务。

真实 GUI 需要恢复 marenben 的交互桌面；当前 query user 显示该账户 session 7 为 Disc。此前 empty-host 证据的 actualGUI=false 不替代真实 GUI。

## 首错与清理

library02 首错为 JSON 宏数组 .map 表达式缺括号，raw exit 101、0 tests；修复后 library03 通过。后续 library04 首错为嵌套 opaque 类型可见性（E0365），raw exit 101、0 tests；修复后 library05 通过。两轮原始 source/stdout/stderr/result 均保留。

library07 的预期外部编辑失败现场已在当前 SID 私有目录归档，4 文件、7290 字节，归档摘要在 library07-scene-archive.json。删除该自有合成目录的请求被 automatic approval 以 blocked by policy 拒绝；没有重试或换机制，目录保留。历史被拒绝的失败树/Profile/Public/schema 清理也未重试。日常 PATH、策略、官方 2.99、账户和官方进程未改动。

完整状态见 final-status.json；每轮原始证据和产品 source manifest 在同目录。所有诊断 accepted=false，allAccepted=false、productionExecuted=false、actualGUI=false、未跑 W2/双 SID、未发布。
