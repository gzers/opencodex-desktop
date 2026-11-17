# r6同次Status与原动作完成历史读取

源码fcc7a688da39c6e289dd6478b919bbabfed2da45/treeef4e08d40a98ff992b454dce077571cef643881f；承接a653a51ac/ecb3b27c8，TEAM16/IMP142/r6与fetch共同源不变。

| 运行 | profile | raw | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
| library01 | debug | 0 | 50/0/0 | 142620 |
| library02 | debug | 0 | 51/0/0 | 134434 |
| library03 | release | 0 | 8/0/0 | 765938 |
| product01 | debug | 0 | —/—/— | 65842 |

本轮无非预期产品/测试失败。首轮50过后静态纠正ActionWindow截止映射Other为TimedOut，补真实已到期窗口不能续用；最终51过。原first source/EXE/75scene保留不归末轮。记录器重复create-only旧scope文件被FileExists拒绝，原快照留存，精确新文件名source-scope-verified记录最终源；未覆盖/放宽验证。

HistoryReadWindow为private nonserde/nonClone，只是read locator，无native许可：Status准确when/end<=5s，Action直接借原ActionWindow.deadline，复制原不可变窗口后前后remaining核验；不生成now+budget，不改变原Stop mutation cutoff。CaptureSlot/Journal source/audit pins/actual completed exit-supervisor/canonical/archive全读继承同一request。实际passed elapsed action窗口read与再次构造均TimedOut，Start/Stop/Restart原parent收窄、Restart30/Stop子10保持。

新增native97在真实原件完整关闭/原10秒到期后传fresh Start30 read，仍InvalidData拒绝旧handle-close补签finalStop，phase/current/原immutable事件不变。旧正常/Unknown/partial/late/history5秒用例保持；不是actual ordinary fixedofficial final-history正例。

批次开头核对发现H route config/祖先pin不能单纯take/drop声称完整释放；scope-adjustment在源测试前保留说明。本轮RoutePin/原actual mutation/close/finalseal等21处源/函数逐字不变，routes仍保管。下一步同一canonical显式route-pin close intent/result/actual owned handle状态，随后typed/public Status/dispatcher/后继。原包副作用/初始化/ownerreceipt/原缺席Codex安全退根/最大batch容量/普通MSI/CAP-trust/历史清理保持未完，Start拒绝。

180scene私有ZIP/hash/membership完整核验，全部plan/native/canonical现场保留，1828份原canonical/快照/history另收存；本轮无删除/历史拒绝重试，ZIP非NTFS元数据克隆。最终data/release/product全部核工作/Git明确CRLF等价，产品190输入/最终release144Rust。预存schema未手改/暂存；无官方/日常配置/PATH/账户/策略/业务/模型修改，不旧raw5/空GUI/W2/双SID/发布。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77809152 | 8e9f2c9aebaa47944123d9ceaef09739d4ac0e961f7433ffbaed1f8a97515f0d |
| ocxd | 5398528 | 3eb1d264e99ead4aebed2cf5170ae78fa007859fa3615325e155a3a83b05df88 |
| ocx | 306176 | 921be20b58dec911cbb3884ec86f385abb7555f92c8c06450c2f3eb64fd04890 |
