# 工具完整CI与正式基线构建版本补证

工具 `feature/0.1.10-windows-working-set-pages@3a2ff08a` 的首次push CI38910385568/attempt1已整轮success。五个必需作业原日志的实际checkout均为同一3a2ff08a，main-only build正常skip。前端99文件535项、Mac后端35组923项/6ignored、Windows后端35组648项/7ignored及最终EXE冒烟通过。这是诊断工具分支的CI，不将它混称b4产品候选的CI或宿主真机门禁。

新终态发生在晚期GPU快照之后，按新增证据记录。上一目录保留的 `method-admission.json` 在核验时整轮仍运行、`wholeCiAdmitted:false` 的事实不改；不能倒称完整CI早于21:51的只读快照。

为核查“原生图形框架版本升级导致RSS增长”的假设，只读取正式0.1.9原始Release37871899116/attempt1的Windows作业原日志。实际checkout为de6862d4，制品API的MSI asset623593980/digest b33dd0c8与既有正式基线链相同，创建时刻在该Windows作业内。API名称为 `OpenCodeX.Desktop_0.1.9_x64_en-US.msi`，实际上传日志名称为 `OpenCodeX Desktop_0.1.9_x64_en-US.msi`；显式保留两种拼写，不声称逐字相同，也不依据名称猜源码。

原始Compiling行与b4候选38893610415的原始package日志逐项对照，七个核心包实际编译版本相同：

| 包 | 正式019与b4实际编译版本 |
| --- | --- |
| tauri、tauri-runtime、tauri-runtime-wry | 2.12.1 |
| tao | 0.37.1 |
| wry | 0.57.0 |
| webview2-com、webview2-com-sys | 0.39.1 |

不是从下载行或未提交的旧Cargo.lock推断；旧锁文件未恢复。原生updater另有2.13.2→2.13.1差异，前端API也有源码锁文件变化，不能称所有依赖相同。相同核心版本不证明features、编译器、二进制、启动时序或实际WebView子进程映像相同。019子进程映像当时未采集，不能给已退出历史PID补映像身份。此证据收窄版本升级假设，未取得具体GL分配栈或解除RSS失败。

两次离线解析错误分别是ANSI转义拆开Compiling词元，以及误要求API文件名原样出现在上传日志；错误控制器/观察和原始字节均保留，修订只重新解析既有下载，不重取或重跑CI。21份运行原件副本、先前冻结证据摘要及独立复算见本目录；`verify.py`只读核全部来源、计数、版本与清单。

22:10:21只读读回原候选PID76372仍存活，原配置目录ID/创建时刻、测量克隆、HKCU指针及TASK摘要保持。原配置尚未恢复，原两次退出超时和退出码未知保持。原RSS+20.040155MiB超过+10MiB、b4/823及Mac历史失败、bd2实际失败窗口缺失、严格恢复与全部其余预算/周期/八门禁均不变。DPI专项取消；TASK revision2/in_progress/pending，不合并、关闭、签名、晋升或发布。
