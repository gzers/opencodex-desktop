# r7同源扫描计量及独立R0采集器

消费docs/governance-main14653e88196ccb62c44d05ea86a3e69a24484e1a的TEAM18/IMP144，写集及r7安全边界不变；文档贡献分支已合入该提交。Mac接收回执只确认前批73d8eb155只读准备，并不补新产品、当前r2归档或恢复授权。Windows继续独占scope，旧Job当前历史请求仍待决定。

源码98bd879265ff3444a34c38887262bdc4f706ccea新增cfg(test)完整扫描阶段与Windows测试；cd427f938d6fe84e29b1e49ad447eb10d662b7c8交付独立原生只读采集器。两批已推送codex/024-windows-native。本批不改生产扫描算法，refactor-equivalence.json核对去除明确测试计量/注册后production tokens相同；原对象observe/compare和ADS查询逻辑仅移模块，函数tokens保持。完整Tauri产品构建仍缺失，本批没有同前提OOM重试或内存/页面文件/日常进程操作。

## 假设与实际结果

1. 低内存是否能编译和执行实际完整inventory：能。轻量MSVC构建时逐字消费program_image.rs到verify包装器之前的inventory核心、原backup/safety.rs的真实路径/open/single-link核心及原clean_absolute函数，只适配外层文档注释、未使用的backup文档类型import和错误envelope；没有替换任何原生调用/扫描函数或模拟Owner/成功。source-section-verification.json及4份generated-sections逐字核对，生成物只在ignored target，未建另一份算法事实源。
2. 原完整扫描阶段成本：固定旧自有副本sourceDigest ab9314d3df4475481736245580b24a4d1a6fa52e8030512d9afc9340b6d6c3b6，两次都得tree306e50d0620aa99b61200ec4c0af8c200fb2f12f3b45ee2068dd85dc35f5fd55，覆盖12,213项/623,970,751字节。worker1 elapsed14,552.660ms、default4 elapsed4,892.017ms。worker1枚举1,679.200ms，read/hash8,933.310ms累计；default4枚举600.522ms，read/hash10,317.645ms累计、path boundary2,600.237ms累计。worker span重叠，join wait含等待I/O；仅两个先后样本，冷热顺序不同，不推断精确加速比/生产时限或原30699ms唯一因果。统计里的全树成员与旧解包8660 TAR成员不是同一种集合。保留每byte/fullhash/空目录/全边界扫描，无跨动作摘要缓存、删检查或扩大预算。
3. 完整产品OOM是否仍阻断R0工具构建：不再。新独立采集器复用实际8投影native_stamp、原ADS查询和同一完整inventory段，固定19对象/25原记录以及原image/current摘要；无旧Owner/Job连接、force、resume或restore API。它仅是原SID只读工具，不是生产CLI/GUI或NativeSession。collector-source05在同自有固定副本实际完整读回manifest/current的8项投影及原全树，1通过/raw0、5777ms；新standalone EXE observer-build06 raw0。

library01原生/附件/扫描11通过、1显式固定副本ignored；fixed02单独运行原ignored固定副本计量1通过0ignored；抽取ADS模块时library03 E0425/raw101（遗漏原invalid helper）未执行测试，原失败保留。补回与原phase helper完全相同的错误构造后library04 14通过0失败、1显式副本用例ignored。新增全source collector用例在collector-source05另单独运行1通过0ignored。原native thread/Job/恢复未运行，不把测试数加成生产通过。

首次直接从Cargo输出路径做wrong-controller反例时，原单链接准入先正确拒绝Cargo的多链接EXE；observer-negative07保留该原始拒绝，预期“错SID”的测试判定失败。未放宽硬链接规则：按真实部署方式create-new复制为单链接后，observer-negative08缺参raw2、错phase raw2、错SID raw1且正确原因，三项通过。准备制品1,175,552字节，SHA256 **67a8d01e8358f4516b3407510d741f17def4283e8a47ee137b1f4ca5251e3220**。实际执行36份源码与最终工作/Git仅CRLF等价逐项核对；生成4段源码另核。源码/原日志与精确缺口完整保留。

## 当前r2只读入口和依赖

新Public入口C:/Users/Public/ocx-r7-r2-readonly-20261117-r1/archive-r7-current-r2.cmd已准备，input-manifest为显式v2且固定standalone-native-shared-inventory-section，全部5份输入含EXE摘要已核。仅此新任务root授原测试SID继承RX、out01授Modify，原r2/客户端/custody及机器策略ACL未改；新私有archive根由原SID自己CREATE_NEW固定私有DACL。prepared-public-input.json保具体原件/manifest摘要，任务ACL前后回执保留。

用户已收到一次原普通marenben/session7桌面运行请求：r7要求同SID真实采集，控制器不模拟原账户。启动入口后先独立collect all native rows/process/history errors，再完整归档r2 I/D/H、两custody、准确客户端全树、三协调对象、原Public输入输出/locator与原始setup；前后native/ADS/membership和ZIP每member/hash核验。任何冲突保留逐项证据、原件与部分ZIP，不以窗口文字签成功；不运行官方程序，不清理或恢复，不需要当前两目标force授权。该包记录准备时点；后续用户已实际运行，完整归档/前后44项及保留的WMI首错另存wl2-r7-r2-readonly-20261117-receive-r1，不重写准备制品或原始回执。

R0工具构建依赖已通过独立宿主解除；完整Tauri产品构建仍受旧OOM阻塞，当前Job历史/旧same-image无Status客户端仍为R1依赖，见dependency-progress.json。本轮无越过exact-image、外部kill、线程Resume、旧记录升级或用ZIP重建原ID。21500/75744/40588和旧seq9/三原件/三work全部保留；原SID完整R0回执、修复器合成中断反例与可审查精确计划未齐，不请求或推断恢复批准。future beforeResume partial anchor/原native恢复工厂与P1单次inventory优化继续未完，Start/Closed/allAcceptedfalse。

测试自有tempfile场景通过后由测试拥有者清理；仅复用旧固定副本做只读扫描，没有再解包或运行官方镜像，副本原件保留。Public输入与本轮原日志/源码/collector EXE保留，未重试旧删除拒绝。不改官方2.99、日常配置/PATH/账户或机器策略，不真实业务/模型、不W2/双SID、不发布。
