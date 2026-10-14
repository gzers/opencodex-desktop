# 面板查询持久提交与失败合并

2026-10-14；Mac继续后台，Windows继续原有验收，不启动/操作平台原生实例。本轮修复属于0.1.10 W1/P2查询与终态合同，代码/测试在feature/0.1.10-integration-candidate，文档/.adg仅在docs/governance-main。main仍eae85322，原焦点be与其他草稿PR未自动合并。

## 审阅发现及最终行为

原8be源码在query_panel_metadata结束后先发布PanelQuery.value/sequence，再异步提交缓存/持久截止/事件；持久写入失败时排队调用者可能取得未提交的成功结果。9cf213170459ac04927e369ea3bef899041c21fb先等待提交成功才更新内存，修正成功误报，但该中间实现失败时序号不推进，排队者可能再次派生元数据查询。

a7feba5fda9422cd177d31fefb18054b513524a2保留先await持久提交，并把“本次已结束且失败”与“新元数据成功提交”分开：提交成功才接受新value；提交失败保留旧value但记录本次具体AppError，completed_result优先返回该错误；任一已接受查询的终态均推进sequence，等待该次的调用者复用同一终态。后续显式请求读取最新sequence，仍可正常重试并在真正成功后清错误。错误字段只属内存状态，不增加文件/schema/缓存范围或更新API DTO。

互斥锁仍贯穿metadata和commit；获准任务继续独立于界面观察者持有WriterGate准入至终态。没有新周期/自动重试，未更改2秒查询截止、86400/21600秒周期、300/1800/7200秒退避或事件身份/注册策略。事件实际提交回调仍在official_remote_latest_owned内，调度/缓存失败先返回，不调用其publish；已注册通知持久化失败的现有尽力投递合同不变，不能声称事件历史与缓存为一个原子事务。

## 红绿及有界回归

同一新增回归函数3654字节/SHA256 ca26e1f87c7793089fc2ca36134befb283d722459f745bd63baf30872785c402在9cf生产源码实际0通过/1失败（发布sequence仍0，无法完成失败合并）；最终源码通过。初轮测试async借用编译失败保留，不计产品红。完整测试文件因既有测试错误的“失败不推进序号”预期修正而不同；相同字节声明仅针对该新增回归函数。

回归使用真实临时根：先持久化管理器stable/beta状态及面板reserve，以cache文件/目录冲突触发真实atomic_write失败；排队future已实际poll到mutex等待，提交失败后返回同一序列化错误、不暴露旧value成功，原reserve文件字节不变、无新元数据缓存；移除自有冲突后显式重试持久完成86400秒截止，管理器两通道历史保持。等待分支使用与生产相同的observed-sequence/mutex/completed_result；不把该夹具当完整原生IPC命令测试。

另验证延迟commit期间序号未发布、mutex仍持有、真实reserve字节保持；metadata Timeout只有真实退避提交完成后才可投射，之后next_due=101+300、failure1。原观察者取消回归已接入同一生产commit_panel_query_result，保留实际Schedule/cache和受控metadata工作者；成功/失败事件仍为合成标记，不证明AppHandle通知投递。被冻结存储不进入setup的原断言保持。

最终Rust35组/895通过/0失败/6忽略；5项所有权用例均包含于全量。fmt、strict Clippy all-targets/features integration-test和diff检查通过。完整原日志和命令/耗时/摘要保留，最终工作字节与a7提交blob一致；ignored不计通过。本轮UI源码未改，不重复宣称全量UI或原生已验。Windows30图标路径与bb原字节一致，五共享文件与docs/main一致，原未跟踪协作探针保留。

## CI精确身份与本轮限界

9cf push CI38517002166已completed/success，Mac35组/893通过/6忽略、Windows35组/621通过/7忽略，五必需job成功、main-only build条件跳过。9cf PR CI38517007573最后仍in_progress；new a7 pushCI38518361054/PRCI38518366179最后仍in_progress。各run/head/event/状态原件独立保存，9cf不能证明a7；PR实际二进制属于合成检出，后续终态需从checkout日志核验，不把关联head当独立head制品。

9cf成功push全日志ZIP/CRC/安全条目、测试数量与Windows实际checkout原件已核验；本轮没有下载或执行新安装器，没有触发新候选。原fd/fb/8be失败及“独立control Node也无及时启动标记”的未解线索见[查询阶段终态](../20261014-查询夹具阶段CI终态/README.md)，既有成功轮次不撤销失败，未认定为环境波动或本轮面板修复导致。没有减少并发、超时或清理断言求绿。

PR4说明已更新并读回draft/open/head=a7、merged_at=null。本轮仅推进源码、局部回归和证据；不会因CI单轮成功自动制作新候选或批准合并。Windows若继续夹具诊断需绑定当前a7源码，旧bb产品制品不包含9cf/a7修复；后台测试避开性能采样，不增加原生实例。正式新SHA制品验收仍须按门禁协调唯一实例切换。

## 原门禁保持

原macOS配对CPU+1.817629百分点、平均RSS+117.008965MiB失败未解除；冷热每版各≥20、启动p95≤100ms且≤5%、10min闲置CPU≤0.5百分点/RSS≤10MiB、主线程≤50ms、有效缓存100切页/10重启零自动网络和查询进程、真实24h/6h周期及双平台/安装保留/更新备份事件矩阵仍开放。单元或CI不替代它们。

原TASK182保持3878字节/SHA bc7965b4…/revision2/in_progress/pending，acceptance_results为空，新回执overall failed。不执行未获许可的FZ50脚本，不迁移真实数据/替换日常安装，不提前晋升稳定事件核心事实，不自动合并、签名、发布、stable晋升或颁发完成证书。Mac切原生必须先协调Windows实例结束并确认0，目标仍active。
