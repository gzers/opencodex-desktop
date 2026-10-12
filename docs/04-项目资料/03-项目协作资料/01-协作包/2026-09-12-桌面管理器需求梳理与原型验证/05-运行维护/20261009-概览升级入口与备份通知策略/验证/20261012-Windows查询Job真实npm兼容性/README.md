# Windows 查询Job真实npm兼容性

2026-10-12，代码来源 `main@4852820`，原件归 `.adg/evidence/OCX-0110-20261012-WIN-QUERY-LIVE-NPM/`。只执行已核验CI Rust测试EXE里的一个显式ignored live查询函数，不启动新桌面应用，不发当前24h桌面实例的检查。

实际测试为 `modules::about::remote::tests::live_remote_query_returns_version_and_integrity`。生产函数受控发现隔离HOME中的npm.cmd，执行真实npm10.9.0 CLI与原字节Node，查询官方 `@bitkyc08/opencodex@latest` 的version/dist.integrity；此测试不输出远端版本原文，只断言真实解析的tag=latest、版本非空/数字开头，不以缓存响应或注入结果通过。

测试进程环境先清空，只列入Windows系统目录、任务HOME/AppData/TEMP/PATH和显式sandbox根；缓存由测试临时目录产生。外部npm CLI文件只读，复制的Node SHA与来源完全一致；不继承凭据或npm_config环境、不读取真实用户HOME配置、不执行包生命周期脚本。该测试使用生产新Job的启动/归属/清理逻辑，不替代新候选UI查询。

20:07:03至20:07:06（UTC+8）实际运行1/1通过、exit0，测试2.48s/外层2.548s。独立核查测试根下Node进程为0、测试HOME无npmrc；仍只有原桌面PID38924，周期监测18944实际句柄存活、完整原周期调度未改变。

保留两个启动前工具错误：首次误用PowerShell只读HOME变量，第二次创建已存在的证据目录未允许重用；均在测试进程启动前停止，修正任务专用变量/目录处理后才产生实际结果，不算产品失败、不抹去原件。

9个原始文件集合与摘要、实际成功及原周期保持断言冻结并重验通过。只通过一个实际生产Rust查询函数兼容性样本，不认证新桌面WebView、完整网络/取消/安全矩阵、安装、同机性能或真实周期。TASK/脚本二次确认及剩余门禁保持，原预算/既有failed不变，IMP in_progress，不关闭或发布。
