# lifecycle.3 单用户启动成功／正常停止失败回交

2026-11-09，Asia/Shanghai。按 user1-record-path-fix-handoff-20261109.md 执行，实际读取文档提交765a900bcc719628c3135039e15b8c997f262b7e。新源码和lifecycle.3完整Desktop先构建，再仅调用一次真实user1 Start；启动在原预算内通过，立即正常Stop返回lifecycle_stop_unknown。整体accepted:false，正常停止及客户端还原未确认。没有追加Start/Stop，没有放宽判据。

## 新源码与新制品

- 代码codex/0.2.23-integration@d3b34a234e32b63365672052772c2c72c719f97c；本机未改产品、维护包或测试脚本。
- windows-candidate.yml显式purpose=core-build，run42625073747/attempt1完整成功，实际head_sha等于上述源码。仅core-build / prepare执行，其余分支均skipped；构建步骤02/25只准备程序和观察工具，没有运行core-sweep。两次只读run查询出现传输EOF，随后连接器查询成功；仅派发一次构建。
- 新artifact13678273054，名称windows023-core-program-d3b34a234e32b63365672052772c2c72c719f97c。ZIP304980630字节，SHA256 7746177f4284e4e96a2417b2e741e6a2227903f495347dd7fb72f426a1305d26，与API大小/digest一致。下载显式使用http://127.0.0.1:3060代理，未修改全局配置。
- core-build.json全部9个成员及清单完整性通过，appsTree=3bd507805a74d345169a3c5da4eaab085d18200a，policy SHA256=37c4e79d231e56ea8f793ed89ac57d413d6dcfa74d491e61cdf346d8eb03e472。新Desktop SHA256=45959bedc843cc7a689f2fdee0888363cda9b9290d37411c36f2ecae31f50854，确认不同于上一轮Desktop；没有复用旧包。
- 运行包2.97.0-desktop023.lifecycle.3，15413397字节，SHA256 ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739，与Git交付原归档、policy和实际程序内成员一致。程序树f3f0e24efc2f1059f28ef38c44cc8627ade8d2ad41f5beb2b6f3201d2b2ea3b6清理前后不变。
- 执行器显式传-ExpectedSource上述完整SHA、-ExpectedBuildRun 42625073747。诊断Node v26.8.1从新制品提取。构建日志artifact13678741700，ZIP35038字节，SHA256 5e797b3a8aaf09566c322eae115954bcf50c077a3e4e92c6d7b9b10836610fdf。原来源和编译日志在user1/build-preparation/。

## 环境与四项证据

现有本地工作站EZIO-MINI，Windows Server 2025 Datacenter/build26100/UBR33438，非临时Actions VM。RDP会话2，40次采样全Active，测试期间无会话连接/断开事件。控制器21:43:16—21:49:55运行，实际elevated:true/type2。唯一临时普通账户ocxaccu16eb88db7a，SID S-1-5-21-587408064-856152790-2583328096-1012，Profile C:/Users/ocxaccu16eb88db7a；应用PID40852，实际elevated:false/type1，原生窗口可见，sandbox:false。

| 回交项 | 本轮实际结果 | 原证据 |
|---|---|---|
| ① Start原回执／数量／唯一性／PID端口 | Start仅1次，原回执1条、成功1条，唯一成功回执。服务PID14788/端口53668与受保护记录、健康挑战、原生SID/镜像/创建时间及127.0.0.1监听一致；实际健康版本为lifecycle.3。全部动作回执共2条，另一条是Stop失败，不能混算为第二次Start。 | user1/action-001.raw.json、action-001.json、start-runtime-step.json、runtime-record-native.json、owned-health-observations.json |
| ② Start trace／startup_output／首错 | 原启动预算30000ms，记录15072ms通过；app.log第7行21:46:21.193确认启动。实际空客户端目录由服务写入配置并绑定所选端口，原断言通过。该成功启动日志没有startup_output字段，不补造或借用旧批次输出。首个产品失败发生在随后Stop，非启动。 | user1/start-runtime-step.json、desktop-logs/app.log:7、user1-result.json、event-order.json |
| ③ 正常Stop trace与真实退出码 | 启动确认事件后约0.019ms进入正常Stop阶段，经原绑定/健康前置核验，约210.484ms后调用Stop。仅Stop1次；app.log第8行21:46:22.076返回lifecycle_stop_unknown，产品动作187ms，公开调用约236.048ms，原预算10000ms不变，非10秒超时。服务原句柄清理前未退出，正常服务退出码null；没有独立Stop CLI，不能填CLI退出码。诊断driver PID88748原句柄exit1。客户端还原断言尚未执行到，未确认；后续精确终止不算正常Stop。 | user1/action-002.raw.json、action-002.json、user1-result.json.trace、desktop-logs/app.log:8、original-exits-before-cleanup.json、driver-exit.json、start-stop-observation-summary.json |
| ④ 账户/Profile/目录清理 | accountRemoved/profileRemoved/userDirectoryRemoved/desktopAclRestored/programUnchanged均true，identityUnproven:false，residualProcesses:[]。独立只读核验账户和CIM Profile均0、目录不存在、控制器/原服务PID不存在、原端口无监听。 | controller-finished.json、user1/post-cleanup-check.json |

Start回执原文：

```json
{"lifecycleState":"running","result":"started","canStart":false,"canStop":true,"canRestart":true}
```

Stop回执原文：

```json
{"lifecycleState":"running","result":null,"canStart":false,"canStop":true,"canRestart":true,"failure":{"code":"lifecycle_stop_unknown","message":"官方停止或客户端还原结果未确认；未强制终止或宣称已停止"}}
```

Stop失败后的21:46:31.655，原生观察仍见服务PID14788监听53668，归属本次SID，记录的DACL仅授予该SID权限。清理前保留句柄中，服务Bun14788、启动Node94284、cmd92456均未退出，退出码为空。后续仅按保留身份精确清理，不能把清理后的进程消失改写为正常停止成功。详细全镜像/SID/创建时间见原退出证据；bootstrap exit0只代表采集器执行完毕。

停止失败发生在原停止成功断言之前，因此没有生成成功的stop-runtime-step.json；采集器four-part-report.json中的stop.traceFile为null，并不表示没有Stop调用。真实失败trace在user1-result.json原事件数组、action-002及app.log，补充摘要只引用这些已发生事件，无重放。

## 结论与接续

本轮证明lifecycle.3新程序在该真实普通账户环境中已通过原启动、记录发布、自动端口、健康认证和客户端实际配置写入断言。原路径拒绝阻断没有再次出现；这是一轮有界证据，不能扩展成全部场景或版本验收。正常Stop及还原仍失败，现有回执无法区分具体是停止执行还是还原确认环节，根因保持未知。

启动30秒／停止10秒／状态5秒和原归属、健康、实际还原、退出、清理判据保持。未core-sweep、双SID、安装器、全量、真实业务同步或发布；未单修第18项，本轮清理成功不关闭其历史失败。下一步由Mac核对本次Stop原证据后确定修复位置，本轮没有自动追加任何产品调用。

采集器显式传本轮artifact ID、新ZIP、document-ref 765a900bcc719628c3135039e15b8c997f262b7e。environment.json的documentReadAtExecution为本次实际合同，documentHandoff为采集器保留的历史来源，不冒充当前文档提交。原输出位于D:/ocxu1-20261109-214306；新构建ZIP/封存程序在D:/ocxu1-lifecycle3-build-20261109-212335作为复现输入保留。账户/Profile/应用私有安装均清理。evidence-manifest.json绑定本目录大小和SHA256，历史批次不覆盖。
