L1 / 本版事务历史恢复回执

提交904cfb221f594c26754e4bc307dd6be31ce91dc5。新增model_transaction_v2_history child复用父Context/Dir/checksum/validate_journal/load_images，不复制引擎，父只末尾注册。新config_file_v2_history.rs/tests；必要parent保存/header guards及export backend trusted render薄wrapper按明确窄授权。

接口config_file::v2::history::list(d,h)、prepare(d,h,HistoryPrepareV2)、accept(d,h,recoveryId)，均schema_v2::Result。list给id/createdAt(null无可归因真实时间)/phase/channelLabels/recoverable。prepare输入sourceId/side before|after/scope affected|all/restoreDefaults/expected ReadBaseV2，输出recoveryId/sourceId/side/scope/restoreDefaults/changes/defaultChanges/channelLabels/notices/requiresRuntimeReview/targetDigest。private plan0600原子写且deterministic request身份，同请求重试不重造。无UI policy。

accept仅同guard stage恢复draft，重复不递增、可修复stage后私有reply中断；配置/库保持不变。原save核exact恢复身份/target policy/defaults/source images/currentlib+base且禁止混入新Model/Channel/Key/部分scope；重新read/rematch、stage/copy/key及project新intent不能擦除恢复身份绕过。save在原统一事务额外library CAS Write（仅restoreDefaults真实图存在时），同config/Policy/ledger/versions原子写，savedPolicy不残recovery授权。历史删除复用原Delete transform。

只读历史reader限定本版root/home绑定、UUID目录、校验和/图尺寸/路径与symlink防护，最多256事务，累计declared images128MiB，单图32MiB，journal64KiB；不recover、不mtime造时间。只读取本版事务精确Policy/config/ledger侧图。普通事务可省略未变化mapping：只有历史raw+Policy可证明的anchor-only源无需额外mapping；nonanchor/未知来源缺图明确拒，不借当前或更晚mapping。缺历史side LIBRARY则restoreDefaults明确model_history_defaults_missing，不猜旧默认；exactrefs必须当前库真实存在且与历史库图一致，不造版本。source bytes/坏校验/根绑定/缺图失败保原件，list不夸recoverable。

只恢复typed规则/连接，不恢复旧Key；同端点保当前凭据，跨端点明确model_history_credential_direction_review而不转发Key。requiresRuntimeReview=true。affected比较忽略来源evidence摘要噪音，保留无关draft编辑和顺序；all按历史顺序。不用v1恢复兼容。

80项配置边界回归（10history+70旧边界）及最后12history定向全过；最后增加真实adoption缺ref/预算及cross-endpoint两项，修改仅history新模块，旧70不重复跑。严格cargo clippy --lib --tests --no-deps -- -D warnings、rustfmt/diff通过。源码/日志SHA见L1-history-v2-manifest.json。日志history-v2-complete-tests.log、history-v2-exact-source-tests.log、history-v2-exact-source-clippy.log及初轮/缺mapping失败保留。

公开命令注册、history页面consumer由L0/L3负责，已发送DTO/保存消费建议。没有日常配置、真实推理/同步/发布。测试TempDir析构、目标缓存留证。本机未上传。
