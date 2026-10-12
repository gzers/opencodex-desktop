# Windows 查询修复集成CI与制品

2026-10-12，源码 `main@4852820b89f7f9df7d54bc623e94f3f55d884023`。前页同源进程树红/绿证明不重复替代本页精确main/制品核验。原件归 `.adg/evidence/OCX-0110-20261012-WIN-QUERY-INTEGRATED/`。

## 同SHA真实终态

main CI38242543274于19:48:41（UTC+8）completed/success，六个job全部success，包含Windows回归/最终EXE烟测及macOS bundle；不存在用feature的skipped build冒充main构建。候选38242585130于19:46:52 completed/success，head_branch=main、head_sha相同，package job success。

两项命名制品下载摘要均与已认证GitHub API一致：

| 制品 | Artifact ID | 原始ZIP SHA256 |
|---|---:|---|
| 0.1.10 Windows x64候选 | 11744123025 | 1e4881bc70aef09097dffbb2b49ab22a27751b6a941e79146537c08090025b53 |
| windows-candidate-evidence | 11744028167 | e7d1f74d860b5224fe048653c9d05fe674c03920ed9248109e1df1486f059913 |

只提取验证过的压缩包内容，路径限制在任务software根；没有执行其中安装器或桌面应用。

## 安装包与EXE身份

MSI SHA256 `c64812da634a3fe8b5010cccfad4502a16cff031206699bf742f8d5634151b39`；NSIS SHA256 `67ab186d4efb72ca99596c36100d9f3f5c6bf7de27a7673dd07c591f06c66e1c`。两个artifact内installer-verification回执完全一致，CI fresh install、first-screen、uninstall结果pass。

以WindowsInstaller只读数据库/CAB提取MSI，得到EXE SHA256 `7526561ccfd6f79b5948fc27e16d232258449f13583b93ab9907a2a8c207f875`，与CI实际安装EXE及预期marker摘要一致，产品版本0.1.10。普通构建EXE、MSI marker EXE及NSIS marker EXE具有不同摘要，逐项核验，未混用。

CI/MSI、NSIS首屏均读取真实应用令牌elevated=true/type1、mode=any、applicationModeMatched=false。只代表CI runner的fresh smoke；不抵本机普通/管理员矩阵、实际默认data/custom根、升级卸载保留或OTA。候选没有代码签名，releaseAccepted=false。

下载候选的Cargo.lock原字节SHA为 `f7be175dc8a55c139781d3076b1eb79bb0af38ecb5664a77aaa8f2a87af9d8ab`，与本地提交锁一致。MSI仍和日常版共享UpgradeCode `{30FC0B19-5C02-5849-A234-D548E7AFC0DE}`，不同INSTALLDIR不构成安全安装隔离；没有在本机运行安装器。

新提取EXE留在代码工作树 `test/software/candidate485/`，未启动。仍保留原周期afff PID38924/监督器44780、监测PID18944（exec93170），独立持有句柄核验存活、实际应用数1；main checkout485与运行afff身份明确分离。19:59:45时调度文件仍等于真实24h初始化基线，stable/auto-check=true，日常EXE SHA仍43e06bc3…，原数据未迁移。

## 验证范围与未闭合门禁

34个冻结文件及六job/包/EXE/锁/共享文件/保留实例断言无参数重验通过。本轮只闭合同SHA CI、制品和只读身份，不提前通过新候选本机原生门禁。全分支共享五文件blob一致；代码main干净，文档分支保留既有无关删除未提交。

原TASK-182仍缺，FZ50脚本二次确认未收到。新SHA原生复验须等待当前真实周期生命周期允许顺序替换；管理员完整交互/其余实际DPI、安装更新卸载保留、自定义根成功、更新说明与进度/安全矩阵、全事件、官方0.1.9同机配对性能及stable/beta/panel真实周期仍未整体闭合。原性能预算及既有failed保持，IMP in_progress，不颁发RCP/证书、不降低门禁、不替换日常版、不关闭或发布。
