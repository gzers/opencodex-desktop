# Windows后续：管理员MSI离线接收入口与生产接线缺口

2026-11-14，Asia/Shanghai。用户清除旧目标后明确要求继续后续并启用目标模式，已创建新的active目标；不是覆盖未完成旧目标的失败请求。目标保持Windows范围持续收口，消费最新合同、按影响范围复用证据、不改官方2.99/日常配置/机器策略、不跑未冻结W2/双SID、不发布。

fetch确认main仍为fd2538e4dae124b7c521b5b84b7ca5b5c6eeef6b，docs/governance-main仍为242f46e71e79501e11610c9a7aa7d6d7bc2e9fd8；TEAM revision10/IMP-34 revision136未更新。管理员MSI代码1c6a0881c29ad014ed1644b9a57f8b2441e27df0、原证据b0e7c6d6a8757b301cbcd47c5ed7082ad86e30ae已在贡献分支，不据此声称集成线已接收。hypothesis.json在本轮新增源码前保存待证伪假设及全部独立项状态。

本轮完成允许test/windows024-native写集中的离线接收器verify-msi-evidence.py及同目录说明，代码0f1dc195ed2e39bbbeffd23eff05f97a78905c57已推送。接收器固定读取原两笔Git对象，校验完整原始manifest、执行源与最终Git blob、真实MSI安装/拒绝/正常卸载日志的退出码和调度/实际脚本顺序、原SID继续、程序身份及PATH/config/D/H/pending/selection前后像。首次失败和清理未完明确重核，不依赖旧总结中的绿色结论。

本次实际命令在verifier01.json/stdout/stderr中对应：

```powershell
python test/windows024-native/verify-msi-evidence.py --docs-repo <docs工作树> --code-repo . --self-test --package <原开发MSI> --test-executable <out02产品lib测试EXE>
```

原包本体重新计算316784640字节，SHA256 9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282；原out02测试EXE本体92022784字节，SHA256 1dcba506281e02bdbf3714779294f32752b5ff6251198bb2ede14ad5eba09622。这是只读核本体，不重建/复制/执行包，也不把当前新增Python提交当原EXE生产源码。异机无本体时可省略两参数，输出明确只核归档摘要，不冒称重算远端制品。

| 独立项 | 本轮结果 | 证据与边界 |
| --- | --- | --- |
| 原134份证据及清单 | 通过 | 133份payload681196字节加manifest；固定b0e7c6d6a Git blob，不修改原证据 |
| 执行源/最终源码绑定 | 通过 | 执行源按原executed02字节核SHA/长度，最终源按固定1c6a0881c Git blob核；保留原CRLF/LF差异 |
| 管理员MSI安装/失败保留/正常卸载 | 原证明复核通过 | exit0/1603/0，helper实际exit1；同SID原pending继续；没有新的真实事务 |
| 提前删文件/保留内容变化/pending残留 | 3个反例拒绝 | 在内存中改原事实并重算内部manifest，仍需语义门拒绝，不仅检测摘要不同 |
| 损坏原字节/抹去首错/切换令牌 | 3个反例拒绝 | 原证据与身份/失败历史不能静默被换 |
| 隐藏清理失败/误报全通过/路径逃逸 | 3个反例拒绝 | 不把管理员证明或退出0当整版/清理通过；不访问逃逸路径 |
| 原MSI与测试EXE本体 | 只读摘要通过 | 两份本体与原归档SHA/大小一致 |
| Python语法/源码diff | 通过 | ast.parse和diff --check；未生成pycache或测试临时目录 |
| 集成接收/生产生命周期/W2 | 未完成 | 无新接收合同/接口与固定候选；本轮没有改共享源码或提前运行最终验收 |
| 普通用户MSI | 未验 | 管理员通过不替代普通用户；旧1625保持 |
| Profile及Public根清理 | 原阻塞保持 | 精确SID1040仍Loaded=true；原error32/System/Registry占用证据复用；删除自动审批拒绝不重试 |

verifier01 exit0表示上述原证明复核一致，输出仍是ordinaryUserAcceptance=false、cleanupComplete=false、allAccepted=false；不是当前机器全量进程核验、集成接收回执或整版通过。本轮没有新产品/MSI运行失败；九类拒绝为明确内存反例。

## 生产接线的具体依赖

对当前main及贡献净差只读核对，详integration-gap.json：本线相对main只有10份允许测试/控制器/说明文件变化，产品源码消费者和安装器模板未变。Windows024 OwnedRuntime/create_suspended/wait_normal_exit等在新infrastructure/windows_official_runtime024内部定义/引用，当前src没有生产调用点；终端消费者使用其私有记录/路径设施，不代表完整生命周期已接。已有attested_status::stop_official仍读取record.desktop_lifetime并核binding.verify_proof，再建立runtime_lifetime::native::Witness。旧共享路径确实存在，不把“未接新024后端”误写成“没有Windows生命周期代码”。

上述attested_status/runtime_lifetime及进程编排消费者属于现TEAM未转交的共享写集；直接改会需要新增生产接口/持久状态/来源及预算合同。现可review的下一输入是：集成线先接收本批管理员MSI证明并明确剩余普通用户/清理事实，再给Windows024生命周期消费者的精确共享接口、允许写集、当前代码/包SHA及同30/5/10秒的受影响验证表。未取得该输入，本轮不猜新Schema、不修改共享消费者或安装器、不复跑旧r5/已过组件，也不提前W2。

代码建议接收增量为e2560f904、ad3dcce02、1c6a0881c、0f1dc195e；均沿codex/024-windows-native贡献，不合main。原证据固定b0e7c6d6a，本轮新证据随docs/024-windows-dev交付。不要merge整条旧Windows历史替代精确净差审核。集成线更新正式IMP/TEAM，本线只写本报告及原windows-dev-handoff，不越权回写正式状态。

清理：本轮只新增两份测试源码/说明及本证据目录，无新账户/Profile/任务/VM/MSI/真实PATH写入/外部临时根；只读核包/EXE没有新大制品。原Profile85文件118556840字节与Public根83文件184522112字节沿原时点清单保留，不声称本轮重算大小；原删除拒绝未重试/未换机制绕过，原共享系统占用未强杀/卸载hive/重启。历史其它拒绝项全部保持。
