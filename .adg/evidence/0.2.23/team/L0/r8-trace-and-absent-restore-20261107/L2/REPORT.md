# 原 None 范围恢复后的只读现状验证

结论：当前生产接收预览没有因本机对象已恢复为缺失而停止。底层原选区直接 capture 拒绝，不等于完整产品预览会拒绝；生产 mapper、Host preview、完整 SyncServiceV2::preview 均选 Download / LocalEmpty，并生成真实 DownloadPlan。

本验证没有确认后继下载，没有执行实际复活。两例在预览后原目标仍缺失，全部原业务 Image 相等，远端 PUT=0。若声称本版已经阻止这类原选区重新下载，现证据不支持。

| 恢复组 | Restore ID | 原选区直接 capture | 生产 Host/Service 预览 | 后继真实下载计划 O |
| --- | --- | --- | --- | --- |
| channels | 4d37634a-a533-462b-a695-6e82b53cfd87 | model_source_invalid | Download / LocalEmpty | 41267cf5-e583-4b52-b432-a54dfa1f4167 |
| extensions | a3e1c6be-ac52-46b5-bb1f-1ad2c342489b | capture_source_invalid | Download / LocalEmpty | 10aaf43e-b18c-4c73-963f-b29c36d9bee8 |

两例使用独立临时 D/H，原接收 O 均为 99999999-9999-4999-8999-999999999999。每例先用实际 ProductionServiceBackend、LocalApplication、ReceiveFileHost、typed transaction 完成原接收，实际 scope restore Applied 后恢复 channels 的 config.json 或 extensions projection 为原 None。随后保留原远端选区，运行直接 actual capture、实际 mapper/Host preview，再调用完整 Service.preview，包括 core 的 DownloadPlan 准备。项目没有名为 prepare_preview 的生产函数；测试走实际 Service.preview→preview_semantics→Host.prepare_with_backup 的真实链。

渠道：原选区直接 capture 返回 model_source_invalid。mapper 的渠道槽为 expected_absent，映射捕获只剩已存在模版；Host 和 Service 均 canDownload=true、canUpload=false、noChange=false，选择 Download / LocalEmpty。

扩展：直接 capture 返回 capture_source_invalid（缺失文件的来源路径校验）。mapper 的本机选区不再包含缺失扩展，捕获为空；Host/Service 同样选择 Download / LocalEmpty。

合成云包含原已知对象，全部 9 个 portable 原子均附匹配内容的 Saved 时间证明。Host 暴露 cloudSavedTime=null 是其当前缺对象/不同本机域投影语义；没有把 Unknown 云当 Empty，也没有借本机 mtime、远端头或虚构删除时钟改变方向。

每例内存读取 14 次，PUT=0；传输对象只有不可变 BTreeMap，任意 PUT 会 panic，未绑定真实网络。全业务 Image 在后继预览前后相等，原 None 仍为 None。全树审计无已有路径变更/删除，新增仅 sync-service-v2 original 与 sync-v2-transactions payload/receive/seen 等私有方案/观察记录，逐路径见 observations.json。scope 未作调整，没有 confirm 后继下载。

## 来源、验证与清理

测试提交 fc5d6b28fd06f0134afdd05a0d4eca9f64db2b03，父基线 d3c838759d7d153f6ced2be3a94a94e1fc21d0d4，仅新增 sync_local_v2_restore_tests.rs 中两个案例及隔离辅助函数，产品文件、DMD、IMP、AC 状态均未改。与 Main 已核对提交 4f087eaeab7e5c8fbb8a00805e904efb7131f4ac 的 12 个相关生产文件原字节一致；Service 仅增加 readonly result-owner/accessor，compose/preview 逻辑一致，差异原件附后。

正常应用 crate 两项定向通过（38.30 秒），严格 Clippy --lib --tests --no-deps -- -D warnings 通过（45.96 秒），rustfmt/diff-check 通过。首轮测试编译仅因本模块 Result 别名遮蔽 transport 的双参数 Result，改为 std::result::Result，失败日志保留；没有放宽源规则、忽略缺草稿/保护或修改产品实现。

两个 tempfile 数据根均由用例显式 drop 并断言不存在，路径与删除回执见 observations.json。仅本线临时编译目录在进程退出后清理；原日志与受控归档保留。没有真实 WebDAV、日常配置、远端清空或发布。

这是一项现状验证，不是产品策略选择或修复，不代表删除传播获得授权，也不将相关 AC 标完成。用户删除策略仍待明确。
