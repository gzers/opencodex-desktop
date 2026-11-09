---
id: "REL-OPENCODEX-DESKTOP-26"
object_kind: "release"
state: "in_progress"
title: "0.2.23 配置同步、用户隔离与模型模版发布"
summary: "Windows回交已接收；按用户目标模式授权进行Mac复验、正式制品和通道验证，尚未发布。"
---
# 0.2.23 发布收口

2026-11-10，Asia/Shanghai。用户在主对话明确授权目标模式继续至发版；正式范围沿IMP-31/32/33及原验收条件。当前未生成通过证书、未宣称整版通过。证据根为.adg/evidence/0.2.23/release-20261110/。

## 来源与收件

Windows代码4d783585645206c52b49bce8656475d26940c1d7、文档e67ddd5d28601989bd27846a91b4356aedd7b83e已fetch并快进接收。已按14份原manifest核对696条原文件记录，含失败轮及后续通过轮，不覆盖历史。关键通过范围：普通user1启停与实际客户端还原、双普通SID重叠及Stop peer保持、真实控制台与RDP不同SID冲突/启停隔离、NSIS和隔离MSI安装保留与卸载。范围限制沿[Windows交接](0.2.23-contracts/windows-to-mac-release-handoff-20261110.md)，调试制品不充正式发行物。

用户已将Windows本机Profile1018/旧缓存清理与cross-session-01旧Stop未知原因转为该端并行跟进，不再由本线单独设为Mac发版前置。原失败、未完成和作用范围保留；若本次正式候选有新的启停、归属或还原失败仍据实处理。

维护包固定2.97.0-desktop023.lifecycle.3，15413397字节，SHA256 ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739。Desktop HOME等价比较修复保持签名tuple与Job摘要原字节，原安全/就绪/退出/还原判据不变。

## 发布准备与未完

be078cda0修正实际发布清单缺字段：构建先比较独立算出的版本/归档/大小/摘要/维护范围，再附加tag策略来源，满足既有发布器全部字段核对；新归档仍逐字保持ec14。3个清单契约反例、29个发布器测试、Mac7项认证状态/Stop回归通过。08a3c65ac只对齐README维护包版本，代码与文档副本一致；不改变产品。最终CI42662755407固定be078cda0正在执行，后继README源码差异不影响被验产品树。

下一步骤：最终CI与受影响Mac检查→合入main并补准确CHANGELOG日期/维护包版本→固定v0.2.23源码生成草稿正式制品→校验旧公钥签名和平台资源、实际原包安装/相邻升级及手动回退→晋升stable/Latest并匿名读回。MSI从旧机器级切换新用户级按已采用卸载重装合同，不能冒称无缝OTA。安装目录及数据目录按新版本规则，不实现兼容/自动迁移。

正式资产ID、摘要、签名、最终源、构建/发布run与公开读回待实际执行后回填；不使用构建成功代替原包/升级/还原验证，不以局部通过关闭版本。日常配置/安装、真实业务同步和推理均不在本次验证范围。


### 最终CI首错与有界补齐

CI42662755407前端未获得macOS ARM runner，15分钟后由平台取消（无步骤执行）；请求单job重派因原run仍运行被GitHub403拒绝，未重开整轮。本机相同前端树typecheck、216文件1603测试、build、真实生产入口检查全部通过，原取消与本机证据分开保存。

Mac后端完成严格编译并通过2203项后，旧tests/process_action.rs仍期待Started后Starting且不能Stop/Restart，唯一断言失败。该期待与此前edb908已采用终态契约冲突；f8a9a761b仅修Unix集成测试为Running/允许Stop/Restart，产品字节不改。本机补失败目标和其后未执行22目标，64项过、5条件忽略，doctest过；不重跑已过前段，原失败log保留。Windows原轮仍在执行。

另Mac完整lifecycle.3离线平台归档237753388字节，SHA256 eb8b0176a9fb9a2e3c282cd307a3110d1b54df5b5107d31a2e64ceb32ad2dd03，两个规范归档一致，11328条目、原生依赖/只读CLI通过；只读运行包不冒最终签名Desktop/正常服务验收。证据位于release-20261110/local-frontend、mac-platform和mac-ci-tail，原包留外置临时复现输入。


### 最终源码合入及私有发行草稿准备（2026-11-10）

Windows CI42662755407完整后端回归与TokenUser私有对象所有者检查success，原生凭据及余项仍在原job129318307513执行，未重开。Mac旧测试补齐与前端本机替代已完成，原CI失败/环境取消不改写。为与剩余检查并行，PR72按精确head f8a9a761b合并为089501b26；main补CHANGELOG日期/lifecycle.3/ec14与Windows路径修复说明，最终源e3728109e93498bfbd6e993d49904d57dfd75315，轻量tag v0.2.23已推送。产品源码与4d7835856完全一致，改变仅发布来源清单和测试/说明。

草稿构建42672509824经workflow_dispatch固定v0.2.23启动（tag所指说明提交带skip-ci，无重复push构建）。只准备正式双平台签名制品，尚未公开；Windows CI余项、原包安装/签名/实际升级回退及最终通道读回仍是公开前门。Latest/beta仍0.2.22，不能把tag存在或草稿出包写成已发布。原PR/run源回执及待验项见release-source-freeze.json。


### 实际Mac原包与DMG装配补验

Release42672509824 attempt1在平台job创建前遇GitHub Internal server error，原HTML/correlation留证；同tag attempt2正常进入两端构建。Mac原生应用/更新包上传及deep strict seal已过，随后DMG独立目录包装因工作流漏mkdir失败。main11e3c7a7a只补该目录，不移动tag、不重编产品；下载原tar并按旧公钥验签、原摘要核对后，用原package_macos_023.py重新包装并只读挂载确认，原.app完整字节保持，替换草稿DMG。

Release439492773仍draft。Mac updater tar261709752字节/SHA25621629f68e1caa7101069f31d9965ca2f331ce06c50122b74713df4887ce0792a；新布局DMG263472679字节/SHA2563036033bd61600c6bf3b810f12eb39850d084544f058e2477e5ba637636b436f。旧DMG原件保留，不冒新布局通过。runtime_only42676002620已成功补交签名维护包，实际下载ec14归档与tag策略全字段一致，旧公钥签名验证通过；不重建桌面包。

Mac原包原生检查遇锁屏，在启动任何应用前退出并删除自有测试复制目录；已请求解锁，原生/真实OTA仍待，不能记通过。Windows正式包仍构建，CI受管安装及凭据已过，最终EXE编译中。当前未公开或晋升通道。证据mac-original/、runtime-only-build-terminal.original.json和runtime-original-signature.log。


### Windows最终CI通过，正式原包安装缺口单独补齐

CI42662755407/job129318307513已success，含完整普通账户后端、TokenUser所有者、隔离凭据、受管安装/离线重装、最终EXE、路径恢复及Node22/26环境步骤。Mac原CI旧终态断言及前端runner取消已分别用明确修正/本机证据补齐，不把原run整体改写绿色。当前正式tag产品源保持e3728109e。

Release42672509824 attempt2的Windows NSIS/MSI及更新签名上传成功，但临时MSI策略夹具要求first-attempt，因run_attempt=2在安装前拒绝。原拒绝不放宽、不改脚本伪装首次；同一签名包的新隔离四格安装run42677222722已派，不重建包。MSI/NSIS两个独立已验签包提取镜像，逐个反转Tauri标记后只存在一个全文一致原像，摘要8162012ee70c01e2118245a1bce8a25fec39654472f6ea944135e7f9b7a0076a，原仓库verifyBundleIdentity两边均通过；该摘要来源是两个签名安装器交叉证明，不谎称首轮单独上传了compiler EXE。初版提取器假定patched标记唯一而失败，纠正为原验证器候选机制，原暂存保留且未执行安装。

已从实际草稿下载并通过旧公钥验签的资产：Mac更新tar、Windows MSI/NSIS以及lifecycle.3运行包；latest.json所有所需平台/实际URL/上传状态/摘要及tag策略均核对。Release439492773仍draft；原安装四格、解锁后的Mac实际原包/OTA、Windows NSIS OTA和公开通道读回仍待。证据windows-final/保存原CI/打包首错/验签及精确原像来源。


### 正式Windows安装四格补验通过

42677222722四job均success。原件下载后独立检查每格tag/source、MSI/NSIS摘要、原像身份、actual普通TokenUser、安装首屏/卸载及数据保持回执；四个临时账户/Profile已删除，临时Server策略精确恢复，AlwaysInstallElevated未改。均未重建或替换原资产，default没有路径覆盖，custom安装及显式数据清理helper覆盖按实际报告保留。证据windows-original-installers/verified-summary.json与原件；此通过不代替NSIS真实OTA/手动回退。

当前剩余：Mac锁屏等待解锁后原包正常首装/原生检查，以及两端正式0.2.22→0.2.23签名升级/回退、最终stable/Latest匿名读回。三类更新包及维护包实际签名全过；Release仍draft，通道仍0.2.22，未提前公开。


### 锁屏停点及正式资源复用核对

再次实际读屏权限：AX/截图权限存在但locked:true，原生首装和Mac真实OTA尚未执行；目标保持未完成，未公开Release。正式签名.app内运行归档eb8b0176及完整descriptor与本机已过只读CLI/依赖检查的规范归档逐字一致，原维护包ec14与封存源一致。该资源证明仅复用离线资源验证，不替代原生界面或OTA。证据signed-resource-supplement-manifest.json。已确认无相关运行子进程后移除本轮4个精确解包/只读fixture暂存目录，保留规范归档、已签名正式资产和原日志；未触及日常配置。
