# 0.2.24 Mac/Windows双线协作与Git交接

revision: 4；2026-11-11。实施顺序/状态唯一归[IMP-34](IMP-OPENCODEX-DESKTOP-34.md)，采用范围见其源链接。此文件只定义角色、排他写集、交接与提示词，不继承0.2.23 TEAM的旧派发状态。Windows已按r1接令并回交W0两批，main2d68a28bc已按净树注册；共享程序核心推进至ef4c46767，W1接口接线与W2验收仍等待本机条件满足。本对话不额外开Mac贡献对话。

## 角色和排他写集

| 负责人 | 写集 | 其它边界 |
| --- | --- | --- |
| 本对话/Mac集成 | 跨平台路径/安装/状态/事务/保存/导入/网络、全部Vue/TS产品页、macOS打包/CLI、共同模块和命令注册、版本清单、.github、Cargo/npm根清单和锁、main集成、正式docs/计划索引。 | 不同时改Windows独占文件；从r5提取测试素材仅取精确源，不merge整条旧分支。 |
| Windows W0立即可做 | 新增apps/desktop/tauri/src/infrastructure/windows_official_runtime024/**；新增test/windows024-native/**（含独立原生测试宿主/Cargo清单锁/定向脚本）。 | 平台模块由独立宿主编译引用，真实Win32，不改产品父mod.rs注册；初期未接产品必须如实标记。无假host或全局进程扫描取代归属。 |
| Windows W1条件放行后 | 新增apps/desktop/tauri/src/infrastructure/windows_cli024.rs及test/windows024-native/**；按本机固定的程序根/稳定入口/清单修改apps/desktop/tauri/windows/installer-hooks.nsh、nsis023.nsi、wix023.wxs。 | 保留既有文件名不表示目标仍0.2.23；版本号由本机统一。默认/自定义根等共享语义先有真实接口提交，再改安装器/PATH。 |
| Windows证据贡献 | docs/分支仅.adg/evidence/0.2.24/windows-dev/<不可覆盖轮次>/**与docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md；最终验收另用windows-final-handoff.md和windows-final/<轮次>。 | 不改IMP/TEAM/README/共享索引；本对话接收后回写。路径创建时按真实结果，没有预填accepted。 |

每个文件同时只有一位写者。未分配旧文件默认只读，包括windows_private_create.rs、windows_query_job.rs、runtime_lifetime.rs；可在新模块复用现有接口，若必须改旧接口，回交具体原因/补丁，由本机接或明确转交单文件。不要因为文件名含windows就接管整条路径。

现有shared-path-policy revision7的共享入口保护继续生效。本表是人工明确写集，不宣称已生成OWNERSHIP或跨机锁。真正进入正式TASK前本对话须将新IMP/TEAM及合同登记到有效共享策略/所有权计划；不手工伪造runtime记录。Windows贡献不触及这些入口，正式状态由本对话维护。

## 来源与分支

- 代码主线：main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b；Windows贡献建议codex/024-windows-native，独立干净checkout承接该基线。先fetch核对main，若主线已演进先报告并承接新的共同提交，不reset覆盖本地修改。
- r5技术输入：codex/official299-windows-proof的test交付cba03b42151843c753f25f53ca71a6b2b31dccef、实际执行源66afa61133d7654d3a2dfd172f49845bfb5272c3；只读提取test/official299-windows相关原型与不可变回执，不能全分支合并旧release/维护包改动。
- 当前文档主线：docs/governance-main；Windows证据贡献建议docs/024-windows-dev。代码和文档必须分支分离，代码不入docs分支，证据不入main。
- 最终Windows验收必须改用本机发出的集成代码SHA/安装器/官方包摘要，不能继续拿贡献分支的旧可执行包验收整版。

## 分批提交与接收

1. Windows先交W0第一批真正可编译/可审查的平台原语与定向证据，及时commit并push；不要攒到W1或末轮才交。
2. 每批回交代码分支/SHA、docs分支/SHA、基线与实际执行源码SHA、变更文件清单、实际命令/结果、环境与首错/未完、测试残留清理、共享入口最小接线建议。
3. 本机先核对实际diff/依赖/结果，再cherry-pick或按明确范围集成；由本机接共享入口并及时push共同main/集成基线。Windows继续独占模块，缺共享接口只停接线，不修改Mac负责的类型凑编译。
4. Mac集成引发Windows专属编译错误，Windows修本线文件后小批再交；不反复跑原r5技术可行性链，不把同一问题跨机中转一轮又一轮。
5. 所有代码分支只贡献，不合main、不打tag、不发版、不改官方2.99。最终main/release事实由本对话统一。

## W0首次启动话术（已执行，仅保留来源）

> 按docs/governance-main的docs/03-开发实施/TEAM-0.2.24.md revision1和IMP-34开展Windows W0开发，先fetch并核对基线main 0828a3955。只写新infrastructure/windows_official_runtime024/**和test/windows024-native/**，将官方2.99 r5已证明的SID/私有记录、保留进程句柄/Job、正常退出/监听与路径事实做成真实Rust原生后端和可编译定向测试。r5 test输入cba03b42/实际源66afa611，只取相关原型，勿全分支合并。原30/5/10秒、未知拒绝和两锁有限例外保持；共享mod/入口/Schema/持久格式/安装器先只给建议，等Mac接口提交再接W1。小批提交推送代码及独立docs证据并回SHA/结果/首错/清理；不跑全量/双SID最终验收，不改日常配置或官方包，不发布。

W0已按该提示交付，当前不要重复执行这段启动话术。该提示只曾放行W0独占范围，不让Windows在无接口时猜安装/CLI接线。Windows须自行先读完整TEAM/源合同；此处不是“测试一下旧原型”，输出是准备接生产的独立平台模块，定向测试服务于本次新实现。

## 最终验证交接

最终提示词由本对话在IMP-34 P24-05达成后生成，必须带当时真实Git SHA/制品摘要/受影响验收表/已过可复用项，不现在编造候选编号或承诺已通过。由用户复制到原Windows对话，沿Git回交完整表。本轮只提供当前开发话术；最终话术等集成候选固定后交付。


## 2026-11-10本机已开工及共享接口检查点

用户已授权本对话目标模式执行，本机P24-00路径核心代码main 9f1ffe677已推送，[program-layout-r1](0.2.24-contracts/program-layout-r1.md)为共享接口只读入口；此批只有位置解析/版本路径类型，未冻结current.json持久内容，W1仍等待。Windows W0不受此等待影响；Windows是否已开工不从Mac状态推断，以实际回交为准。

有效shared-path-policy由provider重封为revision6并读回核对，新增DMD-POST023/IMP34/TEAM024/共享合同受保护，原型共享路径保持。未伪造正式团队/OWNERSHIP/派发回执；已有r1提示词的独占W0写集保持，本修订不扩Windows路径。


当前Windows下一步：保留本线成果及4个清理未完的失败目录事实，等待本机完成共享供应/安装/CLI消费者后通过Git给W1精确接线合同；不重复原r5或W0已通过组件。当前本对话继续本机开发，不要求用户再次授权开工。最终W2提示词仍等固定集成候选后交用户。


## 2026-11-11 W0接口修正及当前共享提交

本机真实消费者已推进至main9125f7134，实际官方输入见main3d3c90943，[消费者r2](0.2.24-contracts/program-consumers-r2.md)为后继共享入口；版本更新/CLI/卸载未全齐，W1/W2未放行。内部ocx-runtime(.cmd)与用户显式ocx(.cmd)分离，Windows不要按r1早期稳定入口假定用户命令已启用。

W0新增可独立修正项：[user_profile与opencodex_home分别传入](0.2.24-contracts/windows-w0-integration-findings-r1.md)，本线原样模块当前把二者强制写成同一路径。继续在原独占写集按真实fixture补组件/反例并Git回交，不改共享层/官方包，不额外跑r5或最终双SID。这里只是Git材料已就绪，不冒充Windows已消费/执行。


共享路径策略已由provider递增至revision7并核验，仅新增本机单写的消费者合同r2及W0接口finding，未将Windows贡献handoff列为本机独占，不阻断正常Git回交。历史revision6/早期检查点按原时点保留。
