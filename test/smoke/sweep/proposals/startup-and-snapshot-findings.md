# 第七轮产品初始化与路径查询发现（尚未修改产品）

来源：42399308750，代码11b4266d8，17 pass / 8 fail，accepted:false。

- 两个真实普通用户分别于18:25:45.231Z、18:26:12.206Z记录 integrated runtime provisioned and verified。原路径prefix故障已消失。
- user1的supplied/driver-stderr.log表明model_paths_v2_snapshot所在组合只读请求超过现有10秒CDP预算。data_paths_v2::snapshot_with_paths每次调用validate_managed_installation；需独立定点测量具体耗时，不把日志成功代替实际snapshot/readiness。
- user1重启的child-stderr.log出现Tauri state() called before manage() for state::ProcessContext，导致异步请求无正常响应。lib.rs中该state注册晚于运行来源初始化和后台供应调度。需要产品初始化顺序及首次请求回归；仅前移一行是否足够尚须验证，不可据此宣称已修。
- IPC等待失败原因仍需实际偏好加载/端点结果，本轮辅助观察器链接失败而缺失。观察器改由Cargo临时example构建，不更改产品命令、身份和权限。

现有用户产品例外只授权Windows路径前缀修复及其回归。本发现涉及其它产品路径，尚未应用修复，不放宽断言、超时或就绪门。

2026-11-08用户明确授权初始化/查询修复。已将ProcessContext注册放在IPC线程派发之前；未改变公开CLI支持范围。查询保留每次完整摘要重算，只优化dev/test的sha2依赖代码，并记录真实10秒预算内的快照耗时。尚待新全量轮验证，不宣称修复通过。
