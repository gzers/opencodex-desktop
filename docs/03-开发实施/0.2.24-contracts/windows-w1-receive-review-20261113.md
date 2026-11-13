# Windows W1三批贡献接收复核

revision: 1；2026-11-13；归IMP-34 P24-W1/P24-04，非整版或固定候选验收。用户告知Windows已推送后，实际远端代码为`codex/024-windows-native b8f1bba91ab611765f23aad6935c21bb66490577`，文档为`docs/024-windows-dev 61996e034fb9f4501e802d21a38d5cd59f893431`。原共同代码cbc6174de、文档61f0f99d8。

## 接收与实现事实

逐项审阅13个净变更文件，未越出TEAM revision5的Windows写集。按原主题cherry-pick四笔代码，main对应2a3ad1b09、1d779e02c、c06dfdd12、803b4abca；此时整棵代码树与交付分支相同，未merge含旧产物的历史。再由主线单写父模块注册，当前main **f8e3cf4238f3ae507499a3f592b32344b03f1181**；只多`infrastructure/mod.rs`的Windows条件注册。

| 项目 | Windows实际证据与本机核对 | 尚未证明 |
| --- | --- | --- |
| user_profile/opencodex_home分离 | 真实子进程HOME/USERPROFILE取Profile、OPENCODEX_HOME取私有H；RootBinding继续只pin H。受影响process目标7项、fmt、clippy通过，旧SID/PID/FILETIME/Job/错误环境反例保留。3个源文件摘要逐字一致。 | 生产生命周期仍未消费OwnedRuntime；本机没有运行Windows服务。 |
| 当前用户PATH原语 | 当前TokenUser、受保护Global SID互斥、固定HKCU/Environment/Path；保留类型/UTF-16原字节/缺失，expected比较后写并读回，外部变化不补偿覆盖。日志8项通过，其中child_replace为供父测试启动的入口，不另外夸大独立场景数量。首轮clippy101及修复后原件均保留。 | 尚未接terminal_cli确认、持久事务、恢复及卸载撤销；WM_SETTINGCHANGE是每接收者1000ms，非全桌面统一预算，既有终端不自动更新环境。 |
| 安装器024资源 | 原官方2.99摘要不变；真实Windows stage/descriptor、MSI/NSIS原生编译exit0。MSI只读数据库/Costing两格与NSIS原生成路径函数四格通过，均未创建安装目录。 | EXE是明确标记的测试fixture，非Desktop候选；未执行产品安装、升级或卸载，未通过W2双SID矩阵。 |

本机独立宿主`cargo check --target x86_64-pc-windows-msvc --all-targets --locked --offline`通过；当前Mac完整all-targets Clippy `-D warnings`及fmt通过。新Python驱动只做语法检查，未执行Windows代码、安装器、官方服务、真实同步或模型请求。没有重复旧r5/已通过正常Mac双轮，也未生成新产品包。

## 证据与源码绑定

82份贡献文档/证据共2,605,757字节按原Git blob逐字复制，来源和本机摘要见[接收清单](../../../.adg/evidence/0.2.24/windows-w1-receive-r1/ingest.json)，docs接收提交3eda03484。安装器52份raw-manifest原文件逐一核对字节数与SHA256；其余原始日志与三个result一同保留。既有无完整stderr的工具预检错误仍只作为原交接观察，不重建伪原件。

13份执行源绑定中10份与Git源逐字一致；3份安装器模板的Windows执行源为CRLF，本机Git源为LF，逐个验证只有行尾转换。两套摘要明确保留，不能宣称这3份执行源逐字相等。stage驱动执行发生在增加dont_write_bytecode之前，原执行版另存，不能冒充执行过最后一行防缓存调整。

本机实际有descriptor/pin/生成NSIS/WiX及原日志，可重算其摘要；274,033,721字节运行归档与MSI/EXE本体仍在Windows，本机只核对它们在报告/原日志中的记录一致，**未下载重算本体摘要**，不得当作固定候选。descriptor为42539a22dde96977d7575077d3e27e4537fe2794a46c07c565a73cedb7a11a1a，包摘要14ef6025a9554e9b37ae2bce0c32d6206a4f82ccacb3d07ade20d5e3adaacba7。

复核工具[verify_received.py](../../../.adg/evidence/0.2.24/windows-w1-receive-r1/verify_received.py)检查Git原字节、52份原清单、执行源、descriptor/pin/官方输入、原失败、实际组件通过数及清理未完；不会运行接收的脚本或Windows制品。收到的原记录和本机复核记录分开保留，整版accepted保持false。

## 清理与后继

Windows原交接明确：测试私有注册表键已删净读回、相关fixture进程/Job/窗口/句柄已退出或释放；MSI超时探针只终止精确自有原句柄，退出-1的失败保留。原4个W0目录、11808文件/2,108,815,151字节installer-w1构建树及2份pycache仍在，`cleanupComplete=false`。自动审批拒绝了对应精确删除，理由`blocked by policy`；命令未执行且未换机制绕过，本机不将归档或本批成功冒充已清理。Mac本轮没有新运行夹具/临时账户，保留正常工程编译缓存。

下一步由主线接共享Windows终端确认/撤销/恢复和卸载helper，以及官方生命周期生产消费者；Windows独占文件仍由原Windows线持有。本机具备可做的共享工作，当前不要求Windows重跑这批组件或W2。等实际需要平台原生执行时按用户要求停下给简短提示词；最终W2仍须固定集成候选及真实摘要。其余根未移中断/创建取消、一般客户端图路径/完整进程集合与刷新、NAME、复杂代理、官方交付verifier及README/固定包等IMP-34原完整范围保持，未发布。
