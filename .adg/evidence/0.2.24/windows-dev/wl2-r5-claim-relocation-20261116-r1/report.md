# Windows claim与原件根移动的原生边界

TEAM15/IMP141/r5，源码3e19075846dfea81ba8392bb60a3d3031afbf116/tree0f89de855699082084baa7b3290caad5e98419f9。仅Windows原生metadata/private检查及coordination/test变更，原四trait/HMAC/预算/r4排除保持。原claim文件共享恢复strict，保留实际有效的metadata-only父根观察：READ_CONTROL|FILE_READ_ATTRIBUTES0x20080、shareR/W/Delete，原目录FD/ID/8投影/privateDACL及外部祖先pin/当前路径FD一致。N/L/K原pin/共享方式不改，RootTree始终持有原DELETE句柄。

| 执行 | raw | 结果 | ms |
| --- | --- | --- | --- |
| library01 | 101 | 0过/1败/0忽略 | 112380 |
| library02 | 101 | 0过/1败/0忽略 | 87948 |
| library03 | 0 | 1过/0败/0忽略 | 106003 |
| library04 | 101 | 1过/1败/0忽略 | 116776 |
| library05 | 101 | 1过/1败/0忽略 | 84405 |
| library06 | 101 | 1过/1败/0忽略 | 96450 |
| library07 | 0 | 1过/0败/0忽略 | 104044 |
| library08 | 0 | 10过/0败/2忽略 | 102021 |
| product01 | 0 | None过/None败/None忽略 | 77327 |

首错library01夹具额外create_private_directory pin阻止RootTree原DELETE capture/raw32/coordination_tests:235，原夹具多余pin释放后library02真正NativeLease父目录pin仍raw32。只将claim父目录改为元数据观察后library03共存1过；根移动library04在意图记录前因夹具journal父根非private拒绝，改新任务私有journal，不修任何旧ACL。library05真正rename raw5；library06仅claim保留文件试shareDelete也raw5，否定了最初共享flag足以解决的假设，原始错误/源/EXE/现场均保留，未重写成通过。

library07一次采齐5个独立root：

| 条件 | move | 原码 |
| --- | --- | --- |
| 无额外句柄 | 成功 | 无错误 |
| 仅元数据父目录句柄 | 成功 | 无错误 |
| 仅只读claim文件句柄，shareDelete | 失败 | 5 |
| claim文件＋元数据句柄 | 失败 | 5 |
| 实际NativeLease | 失败 | 5 |

矩阵的测试raw0只表示诊断记录和原字节保全断言完成，后三格未移动，不能签生产。primary-reference-review提供微软FileRenameInformation的目录内文件不能open条款，符合本机实测，不授权POSIX/force回退。未调用其它move/delete工具绕过，也未关闭无关句柄。

最终library08为10过2忽略：原namespacehelper由父测试实际执行；另一个是明确已阻塞的“continuous-open-claim root rename”正例，原全部断言保留，原真实失败library05/06仍在。不是已通过；标ignored是冻结后继前提而非把原码当0。其它受影响native/kernel/private/security/真实SQLite/Bun反例重新验证；无旧GUI空宿主重跑。无效shareDelete生产实验已撤销，原严格文件分享保持。

最小后继决定见claim-handoff-decision-request.json。原固定tgz的claim-only-writer-source核main-account.ts:463/394：reauth在exclusive claim中直接atomicWriteFile凭据，该函数和claim helper均未取得N/L/K。仅静态核对，未执行凭据写入；不能以N/L/K自动替代claim。ClaimRelocationPending空窗只能是另有充分排他证明的条件备选。优先请评估保留原root和两稳定锁原位/claim连续排他、其它原节点同卷custody的设计，须明确新的canonical节点意图/进度/原身份恢复规则，不兼容迁移旧记录；若仍选空窗，必须覆盖claim-only writer。当前两方案均未实现，不drop/reopen/force/POSIX/copy-delete，原预算/失败保全不变。

metadata原语完善不代表完整acquire_existing实际Owner构造已证明。原SharedRoute与同5秒采样接口已消费；副作用frame和原生四域/Bun互操作组件已有前轮证据，但canonicalStopped/typed restore、official初始化/ownerreceipt、broker原Owner与原包H-S-D以及actualClosed采样正例仍未完成，公共Start拒绝。此轮在具体方案决策处回交，不以接口存在或diag绿灯代整组成功。

最终118产品输入/72最终Rust对工作字节/Git blob行尾等价，前端32份复用，MSVCcustom-protocol debug bins0/77327ms。非release预算/签名冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77689344 | 60ad586be75a29c0b0d3cccea9055e848941cb017e6f355926378bebd101faaa |
| ocxd | 5398528 | 208e958336ae6279bd45cf471595b1cd62f006c594806a1cd84d47a419239b39 |
| ocx | 306176 | b0458645ce7e1cb4b9938536ca7318715af3986361ecaeb9479c2a1a3da17a54 |

30场景/155文件观察/1217258字节私有ZIP内容核验，已知合成ADS仅另存观察，原现场保留，不是NTFS身份安全克隆。无账户/Profile/Public/GUI/官方服务/日常PATH策略改，无控制器删除尝试和旧拒绝重试；最终回归的8个Bun子进程只是独立合成SQLite竞争者，不导入官方服务。旧失败broker40588/Public20文件8拒读、CAP/trust非空/普通MSI/全部历史清理不关，overallCleanupComplete=false。不业务模型请求/W2/双SID/发布。
