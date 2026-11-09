# 第13项：等价HOME修复后单用户启停通过

2026-11-09，Asia/Shanghai。本机普通user1的启动、立即正常Stop、实际客户端还原、原服务exit0及账户/Profile清理均通过，已停止继续测试。整版验收和发布未完成。原采集器/driver的accepted:false是固定诊断标记，原JSON不改写；本批有界通过结论见user1/runtime-success-summary.json。另有三个自建编译/下载缓存目录删除遭自动审批拒绝，见末节，不能称全部磁盘临时产物已清空。

## 原因与修复前后证据

上一轮ocxu1-20261109-214306在d3b34a234/lifecycle.3下启动已过，正常Stop187ms报lifecycle_stop_unknown，服务仍活着。用户随后授权Windows直接原因分析、最小修复及定向验证，无需每轮交Mac中转；先写H1/H2假设再实施，见user1/analysis/hypothesis-001.json、hypothesis-002.json及[本机接续合同](../../../../../docs/03-开发实施/0.2.23-contracts/item13-local-stop-home-fix-20261109.md)。

实际Desktop HOME是扩展本地盘符路径；同一封存包Bun1.4.4的realpathSync.native返回同目录普通DOS路径。生命周期生产端以该普通路径发布签名HOME/Job，旧Desktop消费端key只替换斜杠/小写，保留扩展前缀，在Binding.verify_proof及native Witness.live把同一HOME判为不同，尚未发官方POST便拒绝。

真实Bun只读路径夹具及直接提取产品Binding/单元测试的隔离Rust小crate复现：旧实现2过/1失败，最小修复后3过/0失败，原字段篡改/签名反例保持。小crate排除native模块，不替代原生Witness。首次夹具rmSync目录清理报EISDIR，原输出保留，仅改为rmdirSync后确认夹具目录消失；一次offline缺依赖原日志也保留，通过指定代理下载固定依赖后取得真实红绿结果。没有服务盲重跑。

修复61080a1bc644e5abb4ef45794344a7185542b74a仅在两处HOME比较使用已锁定dunce1.0.5安全简化，不重写路径、不改签名tuple原字节或Job摘要，SID/PID/创建时间、owner/DACL、Job成员/限制、租约锁、原生exit0及客户端还原校验保持。机械还原两处比较及helper可恢复旧生产段全部原字节，见user1/analysis/fix-scope-verification.json。回归继续拒绝不同目录、设备/扩展UNC、驱动器相对、尾随点/空格及修改签名HOME。运行包及双平台policy未改。

fdf5d4db551f71cbf493dc97b428f8b9a9364a7a只给core-build加入两组定向测试，成功才封存；提交跳过自动全量CI。完整Windows产品测试绑定3/3、认证状态/Stop边界9/9通过，其余只编译未执行。原日志在user1/build-preparation/build-logs/evidence/。

## 新制品和真实环境

- 代码codex/0.2.23-integration@fdf5d4db551f71cbf493dc97b428f8b9a9364a7a；运行前读取文档e98d1252bc1c5897134dcca64aba9c9a49fe7c43。
- 显式purpose=core-build/run42631183420/attempt1完整成功，head_sha匹配，仅core-build / prepare执行。新artifact13681803354，windows023-core-program-fdf5d4db551f71cbf493dc97b428f8b9a9364a7a，ZIP304980804字节，SHA256 a44664fb38780898d6ce1907f8c277f61c996d1afd008d9ec1935cf9ed17824a。通过http://127.0.0.1:3060四范围下载，合并后整包大小/摘要与API一致。
- core-build.json全部9成员和清单完整性通过；appsTree=a4acf4ad2c0fb25420471d50a7d8e11897362068；policy SHA256=37c4e79d231e56ea8f793ed89ac57d413d6dcfa74d491e61cdf346d8eb03e472。新Desktop SHA256=6bf9606b094ee9154c77a0800a1960f3ed493e9bba44a2172cc12da894a57779。
- 运行包仍2.97.0-desktop023.lifecycle.3/15413397字节/SHA256 ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739。程序树969a002c00681303ea739604380273fc3b4fa4a25a48dfd8833f94c9c336840c清理前后不变。诊断Node v26.8.1来自新制品。构建日志artifact13681293733/36466字节/SHA256 bf8128fa896b39ea19489b312ccfd392f983ae10b83dc841d42b76ce34f1f473。
- 本地EZIO-MINI，Windows Server2025 Datacenter/build26100/UBR33438，非临时Actions VM；RDP会话2，39采样全Active、无断线事件。控制器22:34:36—22:41:03，elevated:true/type2。唯一普通账户ocxaccu154d6c1104，SID S-1-5-21-587408064-856152790-2583328096-1013，Profile C:/Users/ocxaccu154d6c1104；Desktop PID69944，elevated:false/type1、原生窗口可见、sandbox:false。

## 四项原证据

| 项目 | 实际结果 | 原证据 |
|---|---|---|
| ①启动回执/数量/唯一性/PID端口 | Start1次，原回执1、成功1、唯一成功。服务PID81424/端口61934与受保护记录、HMAC健康挑战、原生SID/镜像/创建时间及127.0.0.1监听一致。 | user1/action-001.raw.json、action-001.json、start-runtime-step.json、runtime-record-native.json、owned-health-observations.json |
| ②启动trace/首错 | 原30000ms内15351ms通过。app.log第7行22:37:38.755确认启动，实际客户端配置写入/所选端口绑定断言通过。无产品首错；成功日志无startup_output，不导入旧失败摘要。 | user1/desktop-logs/app.log:7、user1-result.json.trace、start-runtime-step.json、event-order.json |
| ③正常Stop trace/真实退出码 | 确认启动后约0.017ms进入原Stop阶段，只Stop1次；原10000ms内8727ms完成，app.log第8行22:37:46.747确认Stop。记录消失/状态stopped/nativeClientRestoredReadback:true。清理前原句柄Bun81424、Node25152、cmd29052、driver95876均exit0。Stop是HTTP，不填造独立Stop CLI退出码。 | user1/action-002.raw.json、action-002.json、stop-runtime-step.json、user1-result.json.trace、original-exits-before-cleanup.json、driver-exit.json |
| ④账户/Profile/目录清理 | accountRemoved/profileRemoved/userDirectoryRemoved/desktopAclRestored/programUnchanged均true；identityUnproven:false，residualProcesses:[]。独立核验账户/CIM Profile均0、原目录/控制器/服务PID不存在、原端口无监听。编译/下载缓存留存另列。 | controller-finished.json、user1/post-cleanup-check.json、task-scratch-cleanup.json |

Start回执原文：

```json
{"lifecycleState":"running","result":"started","canStart":false,"canStop":true,"canRestart":true}
```

Stop回执原文：

```json
{"lifecycleState":"stopped","result":"stopped","canStart":true,"canStop":false,"canRestart":false}
```

原归属/权限/就绪、Job/租约、客户端实际还原、退出及30/10/5秒预算保持。此次有界通过不外推其它账户/主机/长路径或整版验收；通过后已停止测试。没有双SID、安装器、core-sweep、全量、真实业务同步、日常配置改动、独立修第18项或发布。历史第18项不因本轮账户清理成功关闭。

## 自建缓存删除阻点

自动审批拒绝删除以下三个精确目录，仅返回blocked by policy；命令未执行，没有改用其它机制绕过。它们是本轮自建编译产物/已验证ZIP分段缓存，非账户/Profile/服务残留；未观察到夹具存活进程，见user1/scratch-process-observation.json。需要手动删除且只删除这些目录：

- D:/ocx13-analysis-20261109-220054/binding-check/target
- D:/ocx13-stop-build-20261109-220455/program.zip.parts
- D:/ocx13-stop-build-20261109-220455/build-logs.zip.parts

逐目录存在状态与审批拒绝见user1/task-scratch-cleanup.json，buildScratchCleanupComplete:false。保留原ZIP、封存程序、分析源码/日志与独立证据作为复现输入，不删除原失败证据。

采集器显式传本轮artifact ID、新ZIP、document-ref e98d1252bc1c5897134dcca64aba9c9a49fe7c43；environment.json的documentReadAtExecution为真实本轮合同，documentHandoff为固定历史来源。evidence-manifest.json绑定原文件大小与SHA256。
