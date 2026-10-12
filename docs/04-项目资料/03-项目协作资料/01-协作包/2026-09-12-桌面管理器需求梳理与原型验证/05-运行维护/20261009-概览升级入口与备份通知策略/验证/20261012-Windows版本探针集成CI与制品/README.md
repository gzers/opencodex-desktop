# Windows 版本探针集成CI与制品

2026-10-12，源码 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc`。原件归 `.adg/evidence/OCX-0110-20261012-WIN-VERSION-PROBE-INTEGRATED/`，与前页同源版本探针红/绿函数测试分别核验。原TASK缺失，无正式RCP或完成证书。

## 精确main终态

main CI38254821701于21:44:37（UTC+8）completed/success，六个job含Windows回归/最终EXE烟测及macOS实际bundle全部success。此前Windows完成后macOS仍in_progress的原观察保留，没有提前记整条通过。候选38254885601于21:32:49 completed/success，head_branch=main、head_sha同源，package job success。

一分钟间隔的只读观察器等到两run终态，未重启CI或生产应用；当前记录与终态原回执分别保留。feature的build skipped没有用于此处main六job结论。

| 下载制品 | Artifact ID | 原始ZIP SHA256 |
|---|---:|---|
| 0.1.10 Windows x64候选 | 11750970498 | ba50e432faf254282cbabdb0925450cc2509801a43052426c8e0cb5c6412c65f |
| windows-candidate-evidence | 11749763357 | 157745120745ee50acac1f820779377b048699eb77312a4e4741b2c1fa09ee40 |

下载前检查精确run/source/branch和success；原始摘要与已认证GitHub API一致，认证只在内存。提取路径限定任务software根，没有本机执行安装器。

## 包、EXE与锁身份

MSI SHA `17f70d357f163a881a41b740614b7506a654207041bc2ac3e7ad57717b6c73f6`，NSIS SHA `586e21de7f1dcb32effb9a2fa0af3a7fd4db31e6f475fc58cb24736115ed2f22`。两个artifact中的installer-verification完全一致，CI fresh install/first-screen/uninstall pass。CI真实应用token为true/type1、mode=any、applicationModeMatched=false，不能抵本机双模式或完整原生矩阵。

以WindowsInstaller只读数据库/CAB提取MSI，EXE SHA `f5e4a2eef0ffc364a2a57b47b1463015faf3433a15ff021d544d6e161004f00c` 与CI实际安装和预期MSI marker EXE完全一致，产品版本0.1.10。普通构建/NSIS marker/安装MSI marker摘要分别记录，不混用。下载候选Cargo.lock原字节SHA `f7be175dc8a55c139781d3076b1eb79bb0af38ecb5664a77aaa8f2a87af9d8ab` 与提交锁一致。

候选未代码签名、releaseAccepted=false；MSI仍共享日常UpgradeCode `{30FC0B19-5C02-5849-A234-D548E7AFC0DE}`，不同INSTALLDIR不能隔离升级注册。没有以本机安装测试危及日常版。

新提取制品留在代码工作树 `test/software/candidateeae/`，未启动。仍是原24h观测afff桌面PID38924/监督器44780、监测18944（exec93170），独立重新打开实际句柄核验存活，应用数1。21:47时45次周期观察、零观察错误，完整周期文件与初始化基线一致；日常EXE SHA43e06bc3…保持，未迁移真实数据。

## 验证范围

37个原始文件集合/哈希及六job/精确包/EXE/锁/共享文件/保留周期断言冻结并无参数重验通过。只闭合精确main CI、候选制品与只读身份，**不是新候选本机启动或安装门禁通过**。源码与仍运行的旧afff分开，旧原生/旧周期不能自动认证新eae。

全分支共享五文件blob相同，main代码工作树干净，文档分支保留既有无关删除不暂存。原TASK-182、FZ50脚本二次确认及安全官方0.1.9 Profile环境仍缺；管理员完整UI/其余实际DPI、默认安装data/custom根/升级卸载保留、真实更新说明与进度/安全矩阵、全事件、同机配对性能和stable/beta/panel真实周期仍未整体闭合。

原预算及既有性能failed不变，IMP in_progress，不降低门禁、不颁发RCP/证书、不替换日常版、不提前关闭或发布。
