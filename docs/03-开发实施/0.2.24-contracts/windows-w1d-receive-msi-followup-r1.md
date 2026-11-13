# W1-D接收复核与MSI唯一补验交接

revision: 1；2026-11-14；IMP-34 P24-W1/03/04。W1-D代码b476fa933c35e08b4ca5f58a3f03e370d445ee8e、证据7d8e5db851de4d5e57fd9fa17d5139dbfc57efef。用户告知Windows已完成后实际fetch核对；三笔按8e2c74690/053b0a826/a02b244e9接入main，整树与贡献一致。本机再做单元素循环等价化fd2538e4d，不改变判定/持久字段/调用次序或已通过原生产品语义，原始贡献源摘要和后继源分别固定。

## 接收结论

| 事实 | 本机独立复核的原证据 | 当前状态 |
| --- | --- | --- |
| 原生I/ocx.exe与schema3 | 安装器拥有EXE，启停只改当前用户PATH/私有登记，EntryPin持原句柄/文件身份及当前安装来源；动作间换代不把旧摘要写死进登记，pending仍绑定原来源 | 已接入；不是签名真实性证明 |
| cmd/默认PowerShell | out04两套逐项混合/空/Unicode/引号/特殊argv均相等，二进制stdin相等，stdout/stderr与exit23；PowerShell7.6.9，默认Windows passing未改 | 定向通过；不外推未测Shell版本 |
| 裸ocx | 新Token userenv环境Get-Command/where及实际运行指向I/ocx.exe，停用后命令缺席 | 定向通过；不是手工拼父进程PATH |
| 原官方2.99 | --version原输出opencodex 2.99.0；自定义H config get port --json原输出18241；同current Node/bin/ocx.mjs，原config字节与完整树复核 | 定向通过；不使用合成9.0.0替代 |
| PATH/pending | schema3 Enable/Disable×5边界由新进程继续10格；4类外部变化拒绝且保留字节；旧schema2拒绝不迁移、已有PATH无删除权、通知注入中断继续 | 定向通过，通知失败是明确测试注入 |
| NSIS | 原宏增量5格，未启用/pendingEnable拒绝/升级pending与active保留/非升级撤销 | 宏探针通过；未当完整NSIS卸载 |
| 实际MSI | out01原探针未返回，自有PID原退出-1；out02路径语法1619；out03真实安装1625，原UTF-16日志确认DisableMsi=1及Non-assigned apps are disabled for non-admin users | 保留失败，真实正常/失败卸载未执行；allAccepted=false |
| 清理 | 顺序5个测试账户/Profile均删除、隔离账户PATH精确还原、临时桌面ACL恢复 | 账户维度已清；非全机进程证明 |
| 文件产物 | 新两根1944490289字节及3881472字节unit文件仍在；精确删除自动审批blocked by policy，未执行/未绕过，旧W1/W1-C拒绝不重试 | 清理未完，与MSI缺口分别记录 |

211份贡献文件共819674字节按原Git blob逐字接收，94份组件raw及51份MSI失败raw独立清单逐项验证长度/SHA。9个最终源绑定2个逐字一致、7个仅CRLF/LF，保留两套摘要；不得声称9个执行字节全同。out04旧测试EXE没有独立保存摘要的限制照原交接保留，只核实际执行源码/编译日志，最终out05输入摘要不冒充旧二进制摘要。dev编译/真实模板装包证据不等于release候选；程序/大包本体未传Mac，本机只核归档记录/输入一致，未重算远端EXE/MSI本体。

本机接收证据见.adg/evidence/0.2.24/windows-w1d-receive-r1。21项terminal/3项layout通过（2原ignored保持）；严格all-targetsClippy初报single_element_loop，原失败保留，后继等价修正后严格通过，受影响外部修改拒绝exact用例通过；最终fmt通过。未修改Windows原生判定、测试断言或门槛，不因此重复已过PowerShell/官方消费者。没有新Windows运行、账户、日常配置/Shell/PATH写入、真实同步或模型请求。

## 只补真实MSI，不重跑整轮

Windows下一输入为main **fd2538e4d**及本合同、TEAM revision9。W1-D组件事实可复用；这次只证伪“原非升级卸载hook在真实MSI事务内能先撤销且错误能阻止RemoveFiles”。不跑已过PowerShell/官方读取/PATH10格/NSIS宏、旧r5、W2、双SID或全安装器sweep。

测试环境须本身允许普通用户安装该per-user MSI。当前机器DisableMsi=1的拒绝是明确系统策略，**不改当前机器策略、不提升测试用户令牌、不转per-machine、不用另一安装器/复制文件/条件表冒充MSI安装成功**。优先使用已有允许per-user MSI的隔离测试机/VM。没有可用环境时，回交环境能力/策略事实与缺口，不在同一1625条件下重跑；由用户决定另行准备环境。本合同不授权更改现有机器策略。

可复用的明确开发测试包：Windows本线原 test/windows024-native/target/w1d-bundle01/tauri/target/release/bundle/msi/OpenCodeX W1D Proof_0.2.24_x64_en-US.msi，316784640字节，SHA256 **9a2af03c31f35e1c87bcfe2c839a01142ca50cad8b58d1c79450a05381755282**。输入带独立产品/upgrade身份，不是日常产品或release候选。跨环境转移前后重新核本体摘要；如因实际路径/包输入需重建，记录新源码/模板/最终程序清单/包SHA及与原差异，不替换原失败证据，避免无故再复制完整大资源。

在一个新的普通测试账户/真实OS Profile及精确测试I运行两项；先记录TokenUser、非提升令牌、环境策略只读快照、假设、源码与包摘要，不输出凭据/完整日常PATH：

1. 实际msiexec安装该测试身份包成功，证明三个程序和安装器来源清单确实由MSI落到选定I；如安装仍失败，记录完整原日志/exit/门位置并停止依赖卸载，不将“安装未完成”记成功。
2. 用最小合成运行镜像作为终端登记前提即可，避免重跑原官方供应/模型/同步。构造本次精确pending Enable后真实msiexec /x必须非0；核原程序身份/摘要、MSI产品登记、PATH、D/H及pending原字节仍保留，明确日志是哪条custom action及在RemoveFiles之前拒绝。不能仅运行helper替代此事务。
3. 显式继续原Enable完成后，再执行真实msiexec /x，必须成功撤销本用户PATH/终端登记/pending，并移除安装器拥有Desktop/ocxd/ocx/source组件；本机D/H及哨兵、非安装器拥有live runtime/其它用户对象按原保留合同保持。分别记录Windows Installer/动作/子进程exit与实际后像，不用tasklist exit0推断清理，不把I非空视为程序卸载失败或误删其中runtime。

正常/失败两项的真实日志和前后像同时回交，首次失败不覆盖。新增独立MSI-only测试入口/账户脚本可写terminal_cli_windows_native_tests.rs及test/windows024-native/**，只抽取当前原用例需要的MSI/私有配置/原生终端最小步骤；原isolated_account_terminal_w1d已覆盖其它事项，不整段再跑。三安装器文件仅在原日志证明本批实际缺陷时修，产品共享逻辑若需改先报具体最小原因/证据；本批不为环境策略放宽安全门或改Mac语义。主线停止上述Windows写集直到Git回交。

清理本次测试身份的安装登记/精确自有进程/账户/Profile/临时ACL与PATH还原，再归档验证本轮证据、按允许范围清精确新文件；先前自动审批拒绝项不重试、不换工具绕过。未清项及字节数如实记录。代码仍codex/024-windows-native、证据仍docs/024-windows-dev的新不可覆盖目录及原windows-dev-handoff；分批及时push，回SHA/两项状态/首错/环境/清理。未发布。

## 本版后继仍未关闭

MSI定向未完不冒充整版验收；W1生产生命周期/完整进程门、固定候选/W2、官方交付装配与最终签名后sidecar来源清单生成、根未移中断/创建取消、一般客户端图/进程集合/活跃刷新、NAME、复杂代理及README等IMP-34完整范围保持。当前只是接收Windows终端组件，不宣称已经可以发布。
