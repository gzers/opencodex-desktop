# r6单phase有界节点批记录与原预算闭环组件

源码c259886cf5723409a1c77d41d1f9b42196bd6b61/tree9413be943fefab8c3129a91c04dceef37885353f；实际fetch fac8573e5/578230717、TEAM16/IMP142/r6。承接d20103fee及本地候选958c0deb，旧预算失败/CPU证据ac622c2d8原样保留。本轮仍Windows既定写集，原预算/安全保证/未知拒绝不变。

| 运行 | profile | raw退出 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
| library01 | debug | 0 | 29/0/0 | 400571 |
| library02 | debug | 101 | None/None/None | 27121 |
| library03 | debug | 0 | 30/0/0 | 292634 |
| library04 | release | 101 | 39/3/0 | 1271730 |
| library05 | debug | 0 | 31/0/0 | 293034 |
| library06 | release | 101 | 41/2/0 | 1290369 |
| library07 | debug | 0 | 32/0/0 | 266279 |
| library08 | release | 101 | 43/1/0 | 1292256 |
| library09 | release | 0 | 44/0/0 | 864899 |
| product01 | debug | 0 | —/—/— | 197729 |

批封套非递归、最多512行/1MiB，仍同一个manifest/current/不可覆盖phase链。每批只占一个phase.sequence；内部row用显式连续ordinal，row原event.sequence/parent等于该phase.sequence/parent，保留原eventId/node/root/identity/all8/parent/from-to/expected-state/claim/epoch/原action scope。完整ordered plan正好覆盖操作所有节点；每行经未放宽原单node/recovery validator，在隔离DATA projection全部通过才一次更新Progress，晚行错误不泄漏部分进度。current仅batchId/kind/planDigest/count/prefix，4KiB不变。旧失败不转换或catch-up。

真实native驱动只借准确原两树与活claim。初始original move/work create、Stopped current6work退休/3original准确返回，每组完整意图phase/current先刷盘，再逐个samehandle native操作；每步仍完整BOTH-tree before/after，底层affected-tree比较不变，保留实际native VerifiedBatchValue。批结果仅使用这些真实保管值与原逐节点校验；actual file/security/ID不从JSON/count构造。释放后所有原anchor以独立row实际读回封到单phase，未移动原根/子目录/两稳定对象；原完整Restored/release/历史/close条件不缩。

31负例为初始14与恢复17，覆盖两个原3file操作各0..3 native前缀、current退休0..6及还原0..3、结果不可变记录领先current/真实1字节current，并实际持久化prefix1后中断。canonical完整intent保持pending、准确native对象位置/ID/bytes/all8/children与retired/custody全保留，phase/current失败poison不补结果。新phase plan/ordinal/scope/错根/缺row/duplicate/nesting/重算plan摘要仍错native-parent/新work别名反例拒绝原子性。原raw变更/security及旧测试断言核字节等价，不以批处理豁免ADS/hardlink/reparse/未知节点。

完整新native交易断言56物理event文件（原逐节点88；实际旧trace核对），phase序列仍为物理文件序号；批row不冒物理单事件。原三阶段/四域lease/current替换、恢复/显式稳定range释放/原件不可变history/187准确句柄关闭/及时完成receipt均沿原10秒fixture，不重开/延长期限。原预算失败候选只改后新source实际通过，不追溯改旧失败；编译profile参数未改。

跨原窗history13实际场景也已全部执行：及时完成后真实等原截止过期，新调用者same-sample最大5秒只读通过、actual expired append仍原门拒绝；before/partial/unknown/result-current/1byte/最终完成迟到无receipt，source binding漂移/真实foreign archive/超窗/未来/过期/实际current破坏均拒绝。完整journal原比较体仅加batch摘要比较；非serde完成对象绑定准确current ID/tip/原action/history/raw。此为native组件，不当生产Status/typedClosed/Source/Owner资格。

首library02 E0603/raw101为BatchKind Windows内部可见性，未运行测试/无新EXE，修正精确导出后data回归过。工具级补丁格式与invariant空白匹配失败独列，原断言/输入未被改松。CaptureSlot prepare新批lane编译接入，真实Source/Ready/Stop尚未调用。旧单node测试路径仅cfg(test)保留原序号与故障断言，新端到端用例消费批witness；不会把旧测试都改成新布局以掩盖差异。

实际production接线仍需original routes/PID/audit home/原时间窗/独立四阶段原sharedpolicy在current转移前准入，actual Source/Owner/NormalExit/Supervisor/StopAccepted/current/listener/完整inventory角色及typedClosed/Status5/后继/普通desktop-H-S-D证据。原first-use/ownerreceipt、原缺席安全退根、普通MSI/CAP-trust/历史清理继续未完。Start无条件拒绝，不旧raw5/空GUI/相同1314/W2/双SID/发布/官方或日常配置策略改变。

本轮712场景完整私有ZIP核验，28完成纯数据scene核fresh准确路径/全文件hash/目录membership后清理；684失败/pending及254个D根保留。10077份canonical/快照/history原字节另归raw-scene-records。ZIP非NTFS元数据克隆，历史删除拒绝不重试，overallCleanupComplete=false。完整继承矩阵/原退出码/source/实际EXE与局限归final-status。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77791744 | 927a9b1dacdad97e44d24ff2430b0cd752cf33da1d8ce044feca859812b17226 |
| ocxd | 5398528 | a28482c77039e12cebad1678fe76e8e3f4aa8590178674886aa210fda66939fe |
| ocx | 306176 | 2c3bc009702101eb715ccbd18df29d08467c89061cf3f240a0b5bdbe7d224906 |

前轮library04优化39过3败、library06优化41过2败仍未通过完整原预算，原EXE/source/raw/current及失败现场完整保留。仅批记录尚不足以收口；后继复用私有不可变计划DATA摘要及本地不可变typed事件以减少重复编码/hash/untagged解码，全部native记录/原件/归档比较及独立进度规则仍运行。摘要cache不序列化、计划无修改入口，changed DATA/注入cache字段反例均过；新current进度摘要用planDigest绑定原完整计划与全部mutable/pending/anchor状态。typed entry与原create的encoded(event)同值，磁盘原native ID/all8/raw/ADS/membership检查体逐字不变，不省略native检查。最终新源码通过才接受本轮，不追溯改旧失败。验证细节归invariant/data-digest/typed-record三份证明。

library08为43过1败，离线completed verifier正确指出case84实际未到1字节current，泛化失败断言曾让上游超时满足“无receipt”；原结果保留但不冒覆盖。library09加强全部指定负例的exact native prefix/error/1bytecurrent/late状态断言，同时把新history/close相邻无变更的重复journal读回交给原append完整BEFORE/AFTER边界。关闭前后及结果后三次原路径后验、native archive BOTH-tree边界、原receipt均保留，final-io-boundary证明独列；最终原优化profile44过0败/45个完整native trace才证明本轮。旧原件/安全/字节/children/claim/期限/故障断言不删改，新增断言加强，旧失败不追溯改成功。
