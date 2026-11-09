# 新 Desktop startup_output 诊断单次回交

2026-11-09，Asia/Shanghai。执行合同为 user1-output-diagnostic-handoff-20261109.md，实际读取文档提交 ab66d173bcc058abc5dc700dab67637c1a6e588a。先构建新源码 Desktop，再仅执行一次真实 user1 Start；诊断 accepted:false，Start 仍失败，正常 Stop 未到达。历史失败批次均保留。

## 新构建身份

代码沿 codex/0.2.23-integration@a4abf17aa7aa2b9f677c20aa3d30cda59deed5b4 原样执行。本轮没有代码或判据改动，文档和证据沿 docs/governance-main 回交。显式派发 windows-candidate.yml 的 purpose=core-build，实际 run.head_sha 与完整源码 SHA 一致，run 42619237040 / attempt 1 完整成功。仅 core-build / prepare 执行，其余工作流分支全部 skipped；步骤 02 和 25 是构建程序及观察工具，没有执行 core-sweep。

- 新 artifact 13676565494：windows023-core-program-a4abf17aa7aa2b9f677c20aa3d30cda59deed5b4。ZIP 304982014 字节，SHA256 23bae4812098639bd54693d43eae5f5ddb6450c647a12536f6d7244c03365a7d，与 API digest 及大小一致。
- core-build.json 全部 9 个成员逐一核验，未发现清单外文件；appsTree 为 3bd507805a74d345169a3c5da4eaab085d18200a，policy SHA256 为 7c6e8adc23295138c06736304573056e2ea1f414683a17ccf2b2b0b63d19ba67。
- 新 Desktop SHA256 为 2a99d344745bb8bc59ade92133a70c91144ef3d8bcb7cc313a590f7d3ecc00df，已确认不同于旧产物。执行器显式传入 -ExpectedSource a4abf17aa7aa2b9f677c20aa3d30cda59deed5b4 和 -ExpectedBuildRun 42619237040；诊断 Node v26.8.1 也从新程序完整运行包提取。
- 运行包维持 2.97.0-desktop023.lifecycle.2，15413257 字节，SHA256 c02f4710496775d8f4dfe271bd90d6afdf7dbf960f6e696637271ab9f37bc70b。完整程序树摘要 9025e847fc0d29e99b2ad7b310000d490357f79105608464f02edf2a408d4313，清理前后不变。

构建派发、实际 run、artifact、编译日志、源摘要和成员核验全部保存在 user1/build-preparation/；构建日志 artifact 为 13676130793，ZIP 35145 字节，SHA256 2e6b14198de0eeb6c2f62f932bf48cd39c4b392f3428595e72a74fd5243b45e6。构建包及原 ZIP 本机保存在 D:/ocxu1-output-build-20261109-204221。

## 环境和四项原证据

现有本地工作站 EZIO-MINI，实际 OS Caption 为 Windows Server 2025 Datacenter，build26100 / UBR33438，非临时 Actions VM。RDP 会话 2，43 次采样均 Active，测试区间无连接/断开事件。控制器 20:53:33—21:00:35 运行，实际 elevated:true / type2。新建的唯一普通账户为 ocxaccu19d9256085，SID S-1-5-21-587408064-856152790-2583328096-1011，真实 Profile C:/Users/ocxaccu19d9256085；应用 PID17312，实际 elevated:false / type1，原生窗口可见，sandbox:false。

| 回交项 | 本轮结果 | 原证据 |
|---|---|---|
| ① 启动回执、数量、唯一性、PID/端口 | 实际 Desktop process_action Start 仅1次，原回执1条，成功0条，无唯一成功回执。未观察到受保护 runtime-port.json，无服务 PID/端口可核对；不能用 cmd/Node 启动链 PID 替代。 | user1/action-001.raw.json、action-001.json、user1-result.json、owned-health-observations.json |
| ② 启动 trace 与首个失败 | app.log 第7行，20:56:48.565，lifecycle_start_exit / exit_code=1；exit_observed_ms=12276，duration_ms=12430。后续原30秒健康确认在30011ms结束，不能覆盖此前实际退出。固定 lifetime_admission stage/kind/win32 未观察到。 | user1/desktop-logs/app.log:7、user1/user1-result.json、event-order.json、four-part-report.json |
| ③ Stop trace 与真实退出码 | Start 未成功，正常 Stop 未执行，无 Stop 回执/trace/退出码。清理前保留原句柄证实 cmd PID111040 exit1、启动 Node PID65464 exit1、诊断 driver PID82132 exit1。本批未保留 Bun 原身份/退出码，不补造。bootstrap exit0仅代表采集结束。 | user1/original-exits-before-cleanup.json、driver-exit.json、child-finished.json、controller-finished.json |
| ④ 账户/Profile清理 | accountRemoved/profileRemoved/userDirectoryRemoved/desktopAclRestored/programUnchanged 均true；identityUnproven:false，residualProcesses:[]。独立只读复核账户和 CIM Profile 均0、原用户目录不存在、控制器已退出。 | controller-finished.json、user1/post-cleanup-check.json |

启动回执原文：

```json
{"lifecycleState":"starting_failed","result":null,"canStart":true,"canStop":false,"canRestart":false,"failure":{"code":"lifecycle_start_exit","message":"服务启动进程已退出，请查看诊断日志后重试。"}}
```

## startup_output 摘要

| 流 | bytes | eof | read_failed | kind | hint | site |
|---|---:|---|---|---|---|---|
| stdout | 384 | true | false | unobserved | unobserved | unobserved |
| stderr | 2516 | true | false | error | unobserved | src/lib/windows-runtime-private-record.ts:10:39 |

原日志字段为：

```text
exit_code=1；exit_observed_ms=12276；startup_output=stdout[bytes=384,eof=true,read_failed=false,kind=unobserved,hint=unobserved,site=unobserved] stderr[bytes=2516,eof=true,read_failed=false,kind=error,hint=unobserved,site=src/lib/windows-runtime-private-record.ts:10:39]; duration_ms=12430
```

两流均已读到 EOF 且未报告读取失败；这次不能把未知具体原因解释为输出仍未 drain 完。kind=error 是宽泛类别，hint 仍未知；没有具体原生归属准入 stage/kind/win32，不能推断准入已通过或确定哪项检查被拒。

仅只读核对本次 maintained-runtime.tgz 内的允许来源 suffix：第10行是私有运行记录模块多个检查共用的拒绝抛出位置。此位置不是唯一调用分支；目前无法区分触发它的身份、原生调用或记录校验条件，不能直接归因为历史 token-pointer 缺陷。该映射是对封存源码的读取结论，不是新增运行调用或原始堆栈。摘要及源码成员摘要见 user1/startup-output-summary.json；未导出原始启动错误文本、符号、堆栈、密钥或认证头。

## 回交与边界

原启动30秒／停止10秒／状态5秒及身份、健康、ACL、Job、租约、退出和还原判据保持。Start失败后没有补充Stop或再次Start；没有core-sweep、双SID、安装器、全量、真实业务同步或发布，没有单独返修第18项。本轮清理成功不能关闭历史第18项，未执行的正常停止/还原也不计通过。

采集器按新合同传入 --artifact-id 13676565494、--artifact-zip 指向本次新ZIP、--document-ref ab66d173bcc058abc5dc700dab67637c1a6e588a。environment.json 的 documentReadAtExecution 是实际读取合同；原 documentHandoff 字段仍为采集器保留的历史合同来源，不冒充当前文档提交。

本机 D:/ocxu1-20261109-205318 保留原诊断输出，构建输入目录保留新ZIP和程序作为复现输入；账户/Profile及私有安装已清理。evidence-manifest.json 绑定本目录文件大小与 SHA256，原批次不覆盖。Mac 接续应依据这次已读完的输出摘要和公共拒绝位置，决定下一步检查或诊断方案；本批没有自动追加验证或修改产品/维护包。
