# Windows最终补验提示词

继续gzers/opencodex-desktop本批集成验收。先fetch并读取代码origin/codex/post021-sync-fixes（bd7c0fe8ccecfa39e0706c885dc233f3854dab74，产品源234405094；新增改名备份/恢复日志断言适配，须使用该最终探针），文档origin/docs/governance-main的IMP-30、DMD-POST021及0.2.21-bugs.md第8.7节。由集成分支另开codex/post021-windows-final，避免用原W分支替代L/L2；文档证据另开docs/post021-windows-final，仅写独有证据/新TASK回执，原冻结任务不重绑。

补三类实际缺验：
1. 同最终源回归Windows材质、首次范围/同步方向/ETag有界确认/地址/版本只读观察、名称唯一/无运行标识/无私网前置、重启和面板。新Mac面板空URL修复cf412c336/234405094涉及共享origin复用：补首次导航未完成即离开重开、服务地址变化、旧视图关闭失败/重试。保留代次/超时/浮层遮挡与配置读回。
2. 用物理Windows桌面和MSVC候选正式构建补材质滚动/静止/后台性能；沿用原预算和基线，记录OS/WebView2/GPU/DPI/远程状态。原RDP Server基线与候选均失败，不能改预算或当作通过；没有合适环境就报告具体缺验。
3. 用正式Codex打包桌面默认发现/退出/启动adapter完成独立根正常重启及配置读回；OwnedClient.exe壳只作旧证据，不替代本项。只能操作能证明独立配置和进程归属的实例；无法隔离即回报，不关闭日常Codex。

不发布、不合并main、不跑真实WebDAV、不改日常配置/Key；使用合成数据与自有loopback。提交推送代码/文档贡献、回交SHA/原生截图/检查与未完；最终由本机统一冻结版本、验证原包安装升级/签名更新和发布。
