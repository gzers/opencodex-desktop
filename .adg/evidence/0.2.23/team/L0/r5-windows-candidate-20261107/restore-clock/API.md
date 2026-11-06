# 后继恢复时钟与持久操作接口

源码：c8280f15535a8e75dea7caf5ab5c06e799eac054（operations）、d3c838759d7d153f6ced2be3a94a94e1fc21d0d4（clock）。两片均只改 sync_local_v2_restore.rs 和其 tests，继承 26484af3066c95e20b0c8116ad0d87f5f725e863 的三入口。

## 跨重启操作发现

operations(D,H,originalOperationId:Option<&str>) -> ModelResult<Vec<RestoreOperationV2>>。

返回 restoreId/originalOperationId/planDigest/groups/receipt?，receipt 沿用原 RestoreReceiptV2。只对固定 manager-state/transfer-scope-v2-restores 目录有界枚举本版 plan/progress/lock，核私有文件、严格 JSON/checksum、D/H、原备份图像和原操作证明。损坏明确报错。返回所有匹配操作，不猜 latest，不新建 preview，不使用 UI localStorage 自授权限。历史回执验证不读今天的业务配置；完成后当前配置损坏仍能只读展示原成功。partial 由实际 owned 日志给出原 id/digest 继续；成功重复只返回原 receipt。

## 本次恢复是新的显式保存

原备份 typed before/after 图像继续作为来源证据原样保留；实际恢复目标派生为原 before 业务内容加本次新保存事实，不能再把 saved-content-versions.json 回滚成旧时间或原缺失。

在真实写锁和当前目标 CAS 成功后，恢复首次执行冻结 clock_time 和每组实际 SaveState delta 到原 restoreId 的 progress。重试复核 delta 并沿用冻结时间；每个 derived ledger 的 before/after 在写前持久化，先验证保护全部当前 selected targets 与版本账本，再写。

模型 SaveState 先复用 provider_import::read_policy_v2 对原 raw/Policy/ledger/mappings/精确 binding 得出实际 observation，再调用现 config_file::v2::saved_state；直接 metadata Policy 与实际 capture 原子不同，不能冒用。然后使用现 content_versions::plan_save_at：当前账本 + 实际当前/恢复内容 + 冻结 time + restoreId。derived ledger 与模型 raw/library/Policy/ledger/mapping 等 typed writes 同一 commit_protected_restore 提交。未变化、未选择原子及可信 clock floor 沿单一版本引擎保留。

manager/assets 只在原备份目录的临时私有 stage 重建 backend 冻结的组 Image，复用实际 portable_manager export、portable_assets snapshot/export，组装合法空模型 v6 SavedObjects 后调用 file_objects_v2::save_state。没有根据组名构造 atom key，没有第二套时间引擎，没有网络、凭据值查询或写日常根。每组实际业务 readback 完成后才提交该组保存事实；partial 仅已完成组有新 clock，失败组保留待续。续做沿同 restoreId/time，已完成组不再次产生时间。

## 实证与边界

13 项实际恢复/时钟/operations 定向测试通过，普通应用 crate 离线 strict Clippy --lib --tests --no-deps -- -D warnings 通过；未重跑长 Service 集。核心用例为同 UUID 两次真实 receive 将模型 8192→16384，显式 restore 回 8192；实际 saved_objects_capture 得到内容匹配的新 Saved 时间/event，面对有已知旧保存时间的 cloud 快照，实际 automatic_direction_from_saved 选择 Upload。重复同 restoreId 的版本账本和时间原字节不变，未变化原子记录保留。partial 真实只读 Skill 失败时 preferences 已有本次新保存事实，Skill 尚无；修复权限续做后两组使用同一冻结时间。

原 None 仍恢复为真实删除，不写空 JSON。跨设备删除传播未获裁决，本片没有实现；现 central 删除 tombstone 规则及自动方向边界不变。旧云若是 Unknown 时间，原方向策略仍默认 Download，本片不宣称改变该策略。上述核心方向测试的 cloud 是实际明确保存、可匹配内容时钟的旧快照；不是把 Unknown 云伪造为旧 Saved。

本说明修正前批 scope-restore-evidence/API.md 中将版本账本原样恢复的旧实现描述；历史证据原字节不覆盖。只读 preview/严格模型 restore 草稿守卫/partial 禁刷新等合同保持。
