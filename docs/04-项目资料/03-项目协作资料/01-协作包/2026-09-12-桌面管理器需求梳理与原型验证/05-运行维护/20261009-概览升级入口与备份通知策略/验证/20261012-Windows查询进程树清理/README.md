# Windows 查询进程树清理

2026-10-12，关联IMP-20/TASK-182。实现已按代码归属集成推送至 `main@4852820b89f7f9df7d54bc623e94f3f55d884023`；文档/原始证据归本分支 `.adg/evidence/OCX-0110-20261012-WIN-QUERY-PROCESS-TREE/`。正式TASK原件仍缺，不创建RCP/完成证书。

## 实际问题与修复

原生产npm元数据查询在Unix失败时终止查询进程组，但Windows只kill/wait直接子进程。隔离夹具证明，分离辅助进程在输出超限、非零退出和根进程正常完成后仍可存活；持有stdout的后代还可能让Tokio运行时在2秒超时之后继续等待。

修复位于 `apps/desktop/tauri/src/infrastructure/windows_query_job.rs` 和 `modules/about/remote.rs`。Windows查询先建立匿名、不继承的Job并设置KILL_ON_JOB_CLOSE；根进程以CREATE_NO_WINDOW|CREATE_SUSPENDED启动，实际归入Job并核验唯一暂停主线程/所有者及根存活后才resume。归属/线程核验失败拒绝继续，不恢复未归属根。失败主动TerminateJobObject，再清理/等待根；成功或作用域退出关闭Job，清理它所拥有的辅助进程。未枚举杀死同名进程，不把桌面代理或其他控制进程归入Job。

总deadline从构造查询开始计时并扣除准备耗时，原生产30s与16KiB输出上限不变。Unix进程组清理、固化npm来源和缓存/环境约束保持。只增加既有windows-sys特性，锁定版本/依赖摘要未改变；所查0.61.2官方crate字节摘要与Cargo.lock相符。

## 同源红 / 绿对照

生产Rust函数的Windows测试位于 `modules/about/remote_windows_tests.rs`，通过明确的测试Node绝对路径执行本地`.cmd`/Node夹具，没有包生命周期脚本、远端响应注入或第二桌面实例。

测试在查询完成前持有工作/后代进程句柄并校验Node镜像与创建时间；后代为分离辅助进程，独立控制进程必须存活。夹具不自行30秒退出；8秒看门狗只用于红轮次收尾，一旦介入就失败。生产修复不能靠看门狗取得绿结果；2秒测试deadline后还需在4秒内完成查询清理。

最终红分支 `2218b8c94187efbb5139e33e312676d7beffafb2` 与修复 `4852820` 的测试blob均为 `3dc48b4251cd8465f933bd59387e77dcfc8f339b`，差异仅为生产Job实现、接入和所需特性。

| 场景 | 同机旧实现 | 同机修复 |
|---|---|---|
| 非零退出 | 后代仍运行，失败 | 通过，约152.9ms |
| 17,000字节输出 | 后代仍运行，失败 | 通过，约141.3ms |
| 正常元数据返回但后代仍在 | 后代仍运行，失败 | 通过，约124.5ms |
| 后代持有stdout / 2秒超时 | 约8.112s，需外部夹具看门狗，失败 | 约2.003s，原生产Job收尾，通过 |
| 独立控制进程 | 最终夹具断言存活 | 存活，未误杀 |

本机最终红0/4、exit101、总16.47s；修复4/4、exit0、总2.49s。两份测试EXE来自对应精确CI制品，ZIP摘要核对GitHub API，EXE摘要核对CI生成的测试回执。没有把CI测试EXE冒充运行中的桌面EXE。

## CI与历史失败

最终feature CI38238993959 completed/success；Windows完整34组598 passed / 0 failed / 7 ignored，其中主库502 passed / 0 failed / 4 ignored。最终红对照CI38238999022 completed/failure，主库498 passed / 4 failed / 4 ignored。忽略不计通过；feature的main专属build被skipped，不当作构建成功。

早期1f02红轮次包含后置进程身份读取/自然退出干扰，995红夹具已改为持有句柄，仍受30秒自然退出影响；全部日志保留，未将其余仪器失败统一称产品泄漏。超限红缺陷另以995精确测试EXE在本机复现exit101。修复初版f2de精确CI及同机4/4另有记录，最终结论仍基于相同最终测试blob。后续显式终止/看门狗/分离辅助进程夹具改进未下调生产门禁。

main已无冲突快进并推送，代码工作树干净。main精确CI38242543274与候选38242585130已启动，终态/安装制品仍待单独最终核验。候选workflow只打包并在CI隔离环境验证，不发布、不在本机安装。

## 实机来源边界

本机24h观测仍为旧afff EXE/PID38924、监督器44780、监测18944（exec93170），源码checkout变化不替换其运行进程。没有发它的手动查询、改截止或重启它。本修复尚未在新桌面候选生产WebView中回验，不据旧实例/旧周期证据替代新SHA的门禁。

68个原始文件集合/摘要及最终同源红绿断言冻结后重验通过，只认证此实际生产函数进程夹具场景。主机完整安全/取消/真实npm查询与新候选、管理员完整UI/其余DPI、安装更新卸载、全事件、同机0.1.9配对性能和真实周期等仍开放；原TASK/脚本确认、原预算/既有性能failed保持，IMP in_progress，不关闭或发布。
