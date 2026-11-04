# 0.2.21 G0 渠道直接改名：隔离实测交接

**结论：本机直接改名的事务方案可行，改名技术前置已有运行证据，可供主线接续正式软件任务。** 本结论只覆盖 DMD-OPENCODEX-UI-021 / AC-021-02 的技术可行性，不代表原型冻结、生产改名已实现或整版验收。

代码根为 ocx-0219-fixes，接续分支 codex/0.2.21-fixes，基线 28d7f62e994beed57d291c904ab2c775491bc928；改名相关源码与 b1750f5c 相同。已先读 DMD-UI-021、0.2.21 清单 §3 和 IMP-29。名称与 providerId 一起更新；不引入旧标识/别名兼容，不迁移历史会话。

## 最小结果与证据

最终隔离运行：退出码 **0**，**4.949 秒**；62 个行为探针及 1 条完成记录全部 PASS。另复用主线现成的 1171-test 二进制，精确运行用户指定的两项测试，均为 1 passed / 1170 filtered，未重复运行全套测试。

| 实测 | 结果 | 原始证据 |
| --- | --- | --- |
| 旧 scoped_policy 保护 | provider_identity / “已有渠道的运行绑定或管理身份不能修改”；五份受检文件字节不变，凭据零写入 | [最终回执](logs/run-03.jsonl)，OLD-SCOPED |
| 旧 mapper 保护 | 全页保存仍报 provider_identity / “已有渠道的 providerId 不能修改”；同样零业务副作用 | 同上，OLD-MAPPER |
| 旧方案反例 | 仅改 label 的 changed=false，运行 provider 仍是 Krill-1；仅换 JSON provider 键会丢稳定 ID，credential 与 provider_auth 均阻断旧归属引用 | 同上，OLD-LABEL-ONLY、OLD-RAW-KEY-ONLY、OLD-AUTH-* |
| 候选事务成功 | Krill-1→Krill；实际写入 config、layout、preferences、既存 source-ids 共 4 份文件，迁移 4 个内存 credential 账户 | 同上，TX-SCOPED、TX-ALL-PAGE；[逐步事务日志](logs/run-03.stderr) |
| 当前引用与身份 | provider 键、defaultProvider、modelPickerOrder、disabledModels、Agent provider/连接摘要及既存 provider→UUID 映射一致；channel/node/template/UUID 引用保持，协议分组文件不写 | 同上；断言见 [main.rs](spike/src/main.rs) 的 assert_saved |
| 完整原始型号与未知项 | vendor/a:8b 与 vendor-a:8b 分开保持，Krill-10 不误改；原数组位置、opaque 路由、未知字段、环境引用保留 | 同上，TX-* |
| Key 池 | 真实值、池顺序、slot.id、label、addedAt、活动 Key 保持；不对新引用字符串重新哈希生成池 ID；UUID 账户不迁移 | 同上；PORTABLE-AUTH-CONSUMERS |
| 失败回滚 | 每个凭据暂存点、每份文件写入后、读回失败均恢复原文件字节与完整内存凭据 map；另注入真实 atomic_write 替换错误和内存 credential API 错误 | 同上，ROLLBACK-*、FS-WRITE-FAILURE-*、CREDENTIAL-FAILURE-* |
| 漂移与恢复 | 预览后文件/凭据漂移阻断；读回发现外部新文件时不覆盖它，返回 RecoveryRequired；提交后的清理失败前向补偿，未提交日志回滚 | 同上，DRIFT-*、JOURNAL-REPLAY-* |
| 单渠道与全页 | 单渠道忽略另一渠道的无效草稿；全页可批量改两个渠道；可选引用文件缺失时仅写 config/layout，不创建无关业务文件；取消零业务写入 | 同上，TX-SCOPE-*、TX-ALL-PAGE-BATCH、TX-MINIMUM-FILES、TX-CANCEL |
| 两项既有基线 | protected_connections_identity_and_unrepresentable_shared_edits_fail_atomically；keychain_migrated_pool_identity_is_preserved_instead_of_hashing_the_reference | [基线1](logs/baseline-1.log.gz)、[基线2](logs/baseline-2.log.gz)、[二进制身份及命令](logs/baseline-tests.json) |

成功读回实际调用未修改生产库的 config_file::snapshot / credential、execution_directory::bound_directory / bind、agent::settings，以及 portable_channels / provider_auth 导出消费者。连接摘要与生产消费者计算结果逐值相等。另启动探针自身的新进程重读 config/layout，确认保存后仍为同一 channel.id；这不是官方客户端重启。上游目录输入是本地静态夹具，不访问运行服务。

## 事务方案

实现原型在 [transaction.rs](spike/src/transaction.rs)，凭据后端在 [memory.rs](spike/src/memory.rs)，用例在 [main.rs](spike/src/main.rs)。生产 Rust 库作为原路径 Cargo dependency 链接，没有复制修改受检函数。

1. 按稳定 channel.id 确定保存范围；检查完整 provider/协议分组占用、合法标识、原/目标保护来源和凭据归属。旧导入 receipt 不能因改名继续使用。
2. 内存形成精确文件 before/after 和账户 from/to 清单；保留认证内容、池身份及无关字段。先读回源凭据、确认目标账户不存在。
3. 取得既有 preferences 事务锁，再按生产消费者的 config→layout 顺序持锁；重新核对文件/凭据摘要与可选文件存在性。受检业务文件和普通读取共用实际 TargetFileLock。
4. 在临时根内写 0700 事务目录、0600 before/after 保护文件及只含路径/摘要/账户归属的日志。先创建并读回新账户，再依次原子替换文件；完整读回之后记录 committed 决定。
5. committed 前失败：仅当当前字节仍属于本事务时恢复 before；文件恢复完才清理暂存账户。外部漂移或回滚失败保留日志及两侧凭据，明确 RecoveryRequired。
6. committed 后仅清理不再引用的旧账户；删除失败返回 CleanupPending 并允许重放前向完成，不把已提交文件当普通失败回滚。旧账户暂留只用于事务补偿，不形成运行别名。

## 隔离、边界与接续

- 全部业务配置和事务保护文件使用系统 tempfile，凭据通过现有测试模式改造的进程内 CredentialBuilder 或既有 SafeStore 内存夹具处理。未访问日常配置/钥匙串，未重启日常 Codex，未真实联网。
- [runtime.sb](spike/runtime.sb) 拒绝全部网络、日常配置与 Keychains 读取，拒绝生产树写入，并仅允许探针临时根写入。[构建沙箱](spike/build.sb) 同样拒绝网络及生产树写入，Cargo 离线构建。没有归档配置载荷、凭据内容、生产源码副本或 target。
- 日志重放采用“丢弃 Plan 后重新读取磁盘 journal、继续使用内存凭据存储”的进程丢失模型；未做 OS kill、断电、原生 Keychain/Windows Credential Manager 或官方隔离客户端重启验证。
- 本方案是互斥锁、读回和可恢复补偿，不宣称多文件与系统凭据之间存在平台级原子提交。正式实现须统一所有受影响写入口的锁序、恢复路径及本机凭据锁，并沿现有 safety 边界检查 journal/保护路径；不应原样把 test-only 探针作为生产模块。
- 正式改名还须接入真实内容保存版本事务、普通编辑与改名的组合保存、原型冻结后的 UI，以及隔离客户端重启后目录/路由和原生双平台。见 [待实施最小写集](minimum-write-set.md)。

期间主线并行修改了同步相关三份 Rust 文件及若干 UI 文件，已记录在 [源码身份核对](source-verification.json)；本探针未写它们，不宣称生产全树干净。16 份改名核心源码从读取至交接始终 SHA-256 相同；[初始摘要](source-manifest.before.json) 与最终探针二进制摘要一并保留。未创建软件 TASK，未提交/推送，未在代码分支写 docs/.adg。

可直接重跑保留的临时探针：run-context.json 的 spike 目录执行 python3 run.py。临时目录回收后，执行 python3 spike/reproduce.py；它复制本探针至新的系统临时目录，复用仍存在的隔离 target，使用已归档锁文件离线构建。需使用与本次改名核心摘要相符的代码根；后续生产改名实现会使旧规则反例按预期不再成立。

机器结果：[result.json](result.json)。最终日志为 run-03；run-01/02 及构建期间一次访问私有字段的编译错误保留在原日志中，最终源码通过公开序列化接口检查，没有放宽断言或修改库的可见性。


归档说明：全部原始文本证据另保留在raw-source-evidence.tar.gz，条目名及原始字节保持；归档摘要见archive-manifest.json。两份原始基线日志另以gzip保存，解压后逐字节等同原输出。
