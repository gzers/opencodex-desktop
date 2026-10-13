# Windows 全库隔离对照与探针阶段诊断

2026-10-13继续后台诊断焦点分支的Windows CI失败。原run38349316806/d975b23失败与31份证据保持；本轮原件归`.adg/evidence/OCX-0110-20261013-WIN-FOCUS-CI-FULL-LIB/`，没有接受原CI或修复制品。

## 原失败run EXE的全库对照

沿用SHA e5aaed4c…的原CI测试EXE及已验证的Node24.21.0，在独立HOME/AppData/temp和白名单环境执行两组完整509项主库样本，没有过滤测试或启用ignored。桌面仍只有原eae普通实例。

| 本机实际样本 | 主库结果 | 原CI失败的三个版本场景 |
|---|---|---|
| 默认线程，宿主20逻辑处理器，13:35:58起 | 502 passed / 3 failed / 4 ignored；11.19s | 三项均通过 |
| 显式4线程，13:41:43起 | 502 passed / 3 failed / 4 ignored；15.10s | 三项均通过 |

本机三项失败是`commands::workspace::tests::local_documents_are_frozen`及两个`modules::sync::config::tests`兼容配置测试，完整失败日志和exit101保留。测试EXE内嵌构建时的`D:\a\opencodex-desktop\opencodex-desktop`文档路径，本机该路径的LICENSE不存在；两个配置夹具写日常服务名，而本轮显式沙箱的验证要求沙箱服务名。相关源码blob未改。没有创建CI绝对路径、关闭沙箱、读取日常凭据、改断言或删测试来获取绿灯。

两个全库样本的7个版本/元数据进程树场景均通过，但不抵完整CI工作区和runner环境；4线程不是已核实的CI默认线程数。计数虽同原CI，失败身份完全不同，不能合并、覆盖失败或宣称原故障只是runner噪声。

## 有界阶段诊断

已提交推送`feature/0.1.10-update-center-focus@be7395acfce0ec1e36b83300b576db408f8c35bb`：仅测试期输出版本探针runtime-ready、spawned、assigned-and-resumed、waited-and-terminated的耗时；夹具原5秒就绪断言增加父/子PID文件及ownership-ready存在性；CI读取逻辑处理器、Node路径、temp及两个Rust测试设置。

原2秒生产探针预算、4秒清理断言、生产清理逻辑、fixture脚本、完整cargo测试命令、CI并发与ignored均不改。源码检查确认四个诊断块受`cfg(all(test, windows))`约束，去掉这些块后官方版本来源与父提交逐字一致；夹具模块本来即仅Windows测试编译。这个检查不是本机Rust编译或release制品验收。当时仅检查PATH和常规Cargo/Rustup目录，没有找到工具；PowerShell步骤语法及Git空白检查通过，Rust编译/格式/Clippy/全部回归当时等待真实CI。后续已在既有隔离软件目录找到Rustfmt并对两份改动文件只读检查通过，见[同SHA候选与只读提取](../20261013-Windows焦点修复同SHA候选与只读提取/README.md)；此前原始观察保持，不据此宣称本机完整Cargo构建通过。

新push自动触发的精确run38352525092在13:41:02观察in_progress；该历史快照冻结，不请求旧run重跑求绿。后续结果须另存原件，再审阅同SHA完整CI及制品，原因当前仍未隔离，特性未集成main。

## 保持与范围

原eae桌面37276、监督器24300、监测3920的真实保留句柄/创建时间，以及唯一实例保持；偏好、data-root、备份策略和调度字节、日常EXE摘要不变。监测快照178份、0观察失败，原截止仍2026-10-13 22:55:39/40（北京时间）。后台没有窗口输入、重启、手动检查、安装或改时钟/定时器。

首次生命周期外层包装在成功的PowerShell助手后错误比较了未设置的LASTEXITCODE；实际JSON完整、stderr空。解析确认三个真实句柄后，下一次独立读回改用PowerShell成功状态，没有为包装错误重跑测试。源检查器首次匹配时误假设模块声明的cfg顺序/可见性，修正为已提交的真实声明后通过；产品源码及测试未为检查器改变。

21份原件全集合及字节摘要冻结重验通过，仅说明归档完整。TASK182原合同缺失、FZ50显式脚本许可、安全安装/独立Profile、全部未完原生矩阵、0.1.9同机配对性能、真实周期与历史failed保持。中午最后前台输入仍12:49:55；22:50–23:10窗口须用户再次发开始信号。IMP保持in_progress，无正式RCP/关闭证书，不降低门禁、不关闭或发布。
