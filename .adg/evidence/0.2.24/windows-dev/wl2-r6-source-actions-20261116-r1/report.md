# r6原Source复用与Owner真实动作窗口

TEAM16/IMP142/r6，源码6dfdd33ade6abaa29dc4b5c6aa3fabaf50a9158f/tree1e0ba1e02bd8764b7069ae96e56b733826a8cf1a。承接e7c18e97c/80366fdb8，不改四trait/官方/共享ActionWindow/Status算法/日常策略。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 101 | None/None/None | 61899 |
| library02 | 0 | 7/0/0 | 195276 |
| product01 | 0 | None/None/None | 220920 |

首错library01 E0624：Owner::bind_stop_action位于子模块但私有，父生产入口不可调用；原源码/原stderr/退出101完整保留，无新EXE不能冒前轮制品。修为pub(super)仅父模块可见。复核将Start ActionOrigin准入置于create_suspended之前，避免新增可失败操作在已创建native后/Owner入库前丢失保管。library02七过0败，产品MSVC debug bins通过。

原v2 CurrentPin/Source移入Windows originals/source.rs，原v2继续消费同实现，不另造v3官方准入规则。五个原方法（CurrentPin bytes/capture/check、Source capture/check）Rust方法体token与e7c18基线逐项完全一致，证据source-extraction-verification，保留原固定2.99/archive/image/current文件原代次/8投影、普通Token、原broker图像/noJob、scope/peer/SID/Profile/D/H/I/来源复核。两项实际current原生pin回归通过，未知schema/字段/硬链/写/rename/外来selection/过期继续拒绝。原v2 pending/history不迁移。

Owner保留原共享入口传入的ActionWindow；创建新原native launcher前登记Start段原outer/end与stop子界。真实Stop或Restart在原Owner开始stop前只登记一次原传入窗口，部分stop已开始/已有origin拒绝换窗口重试。新增private NativeAction非serde/nonClone，只有实际Owner在source/check_current通过后能发，绑定Owner session、scope、Token、origin UUID及精确原end/outer/stop；不now+预算。纯/边界反例验证Start parent29、Stop parent3、Restart原stop10、late restart尚有outer却stop已过、重复/已部分stop与错end；原有Restart StopReceipt→start输入保持原outer的两测试一起过。Start腿的窗口也可来自同次Restart外层，未重开30秒。

新Source::journal_context重新检查实际Owner和原Source/current与完整Binding，返回只可由该方法构造的原生peer/四目录pin/Binding/NativeAction上下文；v3 create_from_source消费它与活claim，原动作UUID入deadlineScope，check_source_owner鲜读原actual Owner。该工厂已编译、尚未真实执行，不能以纯Origin测试或current pin代native Source生产正例，不把数据decode变权限。v3仍未切生产入口，下一步要将此原权限接正式phase/新动作段及完整树/封存/副作用，而非继续使用任意deadline构造器签运行。

完整继承状态表见final-status。工作区可写区间、源绑定Ready/PreStop/Stopped封存/官方允许副作用、v3原始Owner与完整Stop/Closed/history/后继、H-S-D生产仍未闭合。原缺席Codex初始化与安全退根blocked，公共require_durable_capture仍无条件拒绝；不W2/双SID/发布，未重复旧raw5/空GUI/首1314。

最终产品输入及最终testRust对工作源/Git字节（只允许记录的CRLF等价），前端32份复用。MSVC custom-protocol debug不是release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77694464 | 2bbc79e69c122676c87f5149302432023aff5c7974698438f6899004f28e0baf |
| ocxd | 5398528 | a593635cc8337787caa8a861b7f070c14d8f9a7bfd0378dbabb4b0a2cfe10dd1 |
| ocx | 306176 | 7af91a76fc88d205934993f906c97e0b2e6cd6a30d0a17b12e9af739e46d24e1 |

本轮无新保留scene，current-pin合成夹具成功scope清理；编译失败未运行native测试或创建测试树。账户/Profile/Public/GUI/官方服务/日常配置无新增，旧失败/pending原scene继续保留，所有旧拒绝删除不重试，overallCleanup仍false。
