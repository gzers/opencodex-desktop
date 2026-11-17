# r6原Ready完整双树只读校验

源码14b3198a8cfe0ce92c4db405def22aa7725f7e6c/tree0fed7c78ab77256f0fb536066a9f56a7c87313fc；承接7c0bc016b/d75d9a8f7，TEAM16/IMP142/r6及新fetch共同源不变。

| 运行 | raw | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 101 | —/—/— | 29215 |
| library02 | 101 | 53/1/0 | 200065 |
| library03 | 101 | 54/1/0 | 142237 |
| library04 | 0 | 55/0/0 | 132533 |
| product01 | 0 | —/—/— | 137597 |

首library01 E0616/raw101无测试/新EXE：测试触私有WorkFrame.archive，保privacy改真实父目录membership不增快照断言。library02 raw101/53过1败：正例新artifact普通write继承未protectedDACL被原private文件检查拒绝，只修合成创建；samebytes换对象反例移保存文件到根外，PrivateRecord新原路径，membership/bytes相同仅ID不同，要求InvalidData而非上游权限/超时。原失败source/EXE/88scene保留。

library03 raw101/54过1败：临时view复用原observe模式，sharing允许写入，原raw32断言遇Ok；保留断言并改专用无DELETE读data/control普通文件shareRead，目录/协调文件sharing不变。准确source/EXE/90scene保留；不通过删拒写断言改变目标。

Actual内部Ready只读gate需fresh原Source/retainedNativeAction身份/typed ServiceObservation/canonicalReady/Released/Open与无pendingStop/content资格；不新动作、不延旧变更截止。完整BOTH-root read views只读无DELETE，原件custody/anchors/全部非产物work ID/bytes/8/children/Grok严格核，source/typedReady前后核毕才释放临时视图；错误保留准确部分视图与poison。无claim/lease重取、journal事件或snapshot目录创建。三已声明catalog叶仅允许文件形状DATA差异，目录同名拒绝，不凭它签policy/restore。

原生测试完整两树持有时config写raw32、claim仍可取、journal序列不变；config edit/unknown emptydir/artifactdir/Grok edit/samebytes differentID均InvalidData，原custody字节保持。实际旧fixture10秒截止后BOTH-tree读取<=5秒可通过，原journal变更check仍TimedOut，准确序列不变。不是actual普通Owner/typedReady/privatefactory正例；公共Status/typed adapter/后继和真实原包仍未接。

268scene私有ZIP/hash/membership核验后全部留，829份canonical/快照/history另收存，ZIP非NTFS克隆，不重试历史拒绝。本轮不新建账号/Profile/PATH/策略或改官方日常；195产品输入/149Rust核工作/Git（CRLF明确等价），预存schema未手改/暂存。14处原变更/完整Stop/typed传输保持。Start拒绝，原生产副作用/初始化/ownerreceipt/原缺席退根/最大容量/普通MSI/CAP-trust/历史清理未完，不旧raw5/空GUI/W2/双SID/发布。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77810176 | 454a54e69352fad56ae9116b377aeadc981d114c562b7f39589010c7d86b3993 |
| ocxd | 5398528 | 63220332d8ebefacbe2691b8f89a42a4c6e5c48b2843e4cbb58fdf2050ee8265 |
| ocx | 306176 | a838e8ba5a1c6df662d91cdeab356f56eabd47ad0f895b92058529213c2fd68f |
