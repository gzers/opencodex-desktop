# r6工作代次可写窗口与完整后像

TEAM16/IMP142/r6，源码c61327035fcd0f64aa38701d440162d6ead76d66/treea6a6bb730cc8695ddf31361c6f35b295018b9959。承接6dfdd33ad/64616427c。只Windows original/native实现与tests，原预算/共享trait/副作用policy/官方来源/日常策略保持。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 101 | None/None/None | 75071 |
| library02 | 0 | 6/0/0 | 275263 |
| library03 | 101 | 24/1/0 | 220952 |
| library04 | 0 | 1/0/0 | 100065 |
| library05 | 101 | 7/1/0 | 59454 |
| library06 | 0 | 8/0/0 | 130927 |
| library07 | 0 | 2/0/0 | 110353 |
| product01 | 0 | None/None/None | 179930 |

首编译library01 E0277：ArtifactFrame数据entry-copy闭包返回Entry却使用?与Result收集；raw101源/stderr保留，无本轮新EXE。改Result<Entry>后library02六过。扩展受影响node/lease/v3回归library03为24过1败：外部原子替换的新工作代次已冻结，测试再用通用单链接id查询旧工作观察句柄，旧对象链接数为0而InvalidData；不是原件移动或sideeffect内容通过。修为受限retained_work_identity，仅对已登记准确旧WORK ID/磁盘非reparse非目录句柄接受links0或1，拒绝多链；不能验证当前路径、original custody、rename或恢复。原通用单链接检查不动。target04通过；library05七过1败，新历史WORK检查误放custody-only前检，使硬链用例在取锁前拒绝而非原要求完整frame扫描。保留原断言，把历史WORK检查移到完整frame capture之后，当前ADS/硬链仍由原scanner拒绝。final06八过，原两份失败/EXE/现场保留，不改原子替换/双代次/硬链断言。

canonical work-open/seal意图先写同一v3不可覆盖phase/current，才实际原生句柄交接与后像读回。工作CREATE_NEW原句柄通过ReOpenFile开同对象只metadata观察句柄，保留原work generation而允许外部写入；分享三项只用于本代WORK观察，原件custody和锚点/稳定锁的严格原句柄不动。微软primary-reference原始网页摘要与使用边界单列，不凭文档推定本机成功。活claim/NLK+原deadline保留，完整Open/release后合成写可行，再取得原lease后冻结完整后像，后来写者拒绝。原先工作句柄和原子替换后当前代次独立保留。

完整native frame包含root/nested/empty/每file/两稳定对象的实际ID/8投影/字节/children/原parent。非产物节点原代次与完整内容/安全必须不变，根children只对准确3种Codex产物提供观察数据例外；Grok无例外。未知file/空目录、ADS/硬链、准确产物名却目录均拒绝且保留完整或部分frame。current同时持baseline和observed digest，不能把观测变化重新标成旧baseline；Open/Observed禁止普通节点退休/恢复。只有native全树后像与开放前完整相同，才持claim重新接准确工作对象并登记WorkUnchangedVerified后继续原件恢复。changed artifact仍仅data，不能代允许副作用或Stop恢复许可。

同卷私有快照分块<=1MiB/原始metadata，逐记录CREATE_NEW/sync/readback，保留实际目录/part/metadata ID/all8/摘要与完整名称集合。context来自活原Journal，同session/plan/binding/manifest/deadline+两个observed digests；不能从路径或摘要文本构造。1280KiB/两块实际重建相同，原子新work generation与旧observer分开；故意无效cache字节只验证数据保存，不签原policy格式。首NONEMPTY分块刷盘后故障保留真实chunk/frame并poison；archive目录外加空child拒绝。补归档读回失败永久complete=false的边界，library07只定向两项受影响archive用例通过，其余原8项复用。该引用目前还没有正式Ready/PreStop/Stopped phase linkage，未采用作恢复资格。

四处intent/native/result/current部分写故障保持准确Opening/pending前缀与句柄，树/journal同时poison，Opening/Sealing未完成不能释放租约，不能在unheld状态逃逸回调。受影响节点驱动/租约/v3纯反例回归保留；未重复旧raw5/空GUI/原1314或无变化shared route/deadline测试。

未完：完整v3 Source/Owner Store生产接入、正式阶段及snapshot reference、跨原动作Stop权限桥、源绑定Ready/PreStop/Stopped全影像+原共享内容policy、动态产物新增/退休与代次恢复、完整normal exit/supervisor/source/endpoint/typed Closed/history/successor及原包H-S-D。新生产factory仍编译未实跑；public Start无条件拒绝。原缺席Codex安全退根blocked单列，不能留目录/两锁冒Absent。不W2/双SID/真实业务模型请求/发布。

产品custom-protocol MSVC debug bins、最终输入/testRust与源码字节核验，前端32份复用。不是release预算或冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77694464 | 59d0c16022b59356d7889fc55c19c8c85fff48ab2ca7db9a1581799353d5aef8 |
| ocxd | 5398528 | 60936ebb48499afbb3f2d1cf988efb20d07b8731af9be669b12faea22f7c6337 |
| ocx | 306176 | 37157dc311ac43c6e651880c44cffbffd4169c25dd502dc6c668b6f67a2ee05d |

174场景私有ZIP逐内容核验；canonical phase/current与本卷work snapshot块/metadata原字节另归raw-scene-records，原生frame与archive引用归native-*。已知合成ADS额外单独复制为观察，不当NTFS克隆；4处已完成纯数据夹具核所有文件和目录后清理，170处失败/正式Closed未闭合pending原场景与83 D日志根保留。全部历史拒绝删除/账户/Profile/Public和broker保持，不重试；overallCleanup=false。
