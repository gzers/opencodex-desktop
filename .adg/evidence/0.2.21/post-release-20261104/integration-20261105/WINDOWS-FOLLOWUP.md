# Windows最终补验提示词

## 68c034362回交后的定向补验

原始证据已接收，审计见windows-final-code-review/evidence-audit.json。继续已有Windows贡献分支，先合入最新集成源f9def3300；不重新创建同名分支，不以旧探针超时报告作最终通过。

1. 使用用户已确认可用的本地Windows控制台完成物理桌面性能/MSVC补验；当前回交仍为RDP。记录Git/apps树、MSVC与SDK、EXE构建commit、复制前后SHA及实际启动日志；不能只从旧feature目录取EXE后声明同源。
2. 正式Codex默认adapter采用一次性Windows测试账号/独立登录会话，先证明目标进程owner和祖先链都属于可丢弃实例，并确认官方adapter的当前用户过滤。从独立PowerShell/测试管理器执行，不能停止日常Codex。listProcesses=null需记录原始exec异常/超时，不能当无实例；不能证明隔离则继续只读停点，不伪造默认adapter返回值或以OwnedClient替代。
3. 用独立expected-first-scope.json重新验证首次本地+云端独有渠道/模版完整集合、已有范围及后续新增不扩选；旧结果0渠道17模版不证明完整并集。账户/Skill/MCP按本轮夹具另补对应证据。
4. 完整模型UI/节点弹窗/保存取消与同步流程重跑至终态，使用已修名称/版本探针。补已知/未知保存时间、无快照、失败状态及HTTP请求/业务文件不变证据；来件只有失败版本状态，缺记录重启截图来自人工showSaved。
5. 材质性能保留原预算和同场景前后矩阵，记录物理/RDP、GPU、DPI、WebView2和制品身份；旧RDP高档两项仅作观察。
6. 原RCP的MSVC/物理性能/正式重启checks与正文矛盾，已被隔离。原件保留，新回执或合法修订逐项填真实结果，不把缺验统一标passed，不生成完成证书。回交原始证据与明确未完，由本机统一收口。

用户已确认有本地Windows环境，继续由Windows对话补验（2026-11-05）。本机工具未连接Windows宿主，尚未实际发往该对话；请粘贴以下内容。

继续gzers/opencodex-desktop本批集成验收。先fetch并读取代码origin/codex/post021-sync-fixes（f9def3300640f0871a93b6bbcec55de646e00567，产品源234405094；已集成faf8dd926补验驱动，保留严格名称/版本合同及不可变根准入，须使用该最终探针），文档origin/docs/governance-main的IMP-30、DMD-POST021及0.2.21-bugs.md第8.7节。由集成分支另开codex/post021-windows-final，避免用原W分支替代L/L2；文档证据另开docs/post021-windows-final，仅写独有证据/新TASK回执，原冻结任务不重绑。

补三类实际缺验：
1. 同最终源回归Windows材质、首次范围/同步方向/ETag有界确认/地址/版本只读观察、名称唯一/无运行标识/无私网前置、重启和面板。新Mac面板空URL修复cf412c336/234405094涉及共享origin复用：补首次导航未完成即离开重开、服务地址变化、旧视图关闭失败/重试。保留代次/超时/浮层遮挡与配置读回。
2. 用物理Windows桌面和MSVC候选正式构建补材质滚动/静止/后台性能；沿用原预算和基线，记录OS/WebView2/GPU/DPI/远程状态。原RDP Server基线与候选均失败，不能改预算或当作通过；没有合适环境就报告具体缺验。
3. 用正式Codex打包桌面默认发现/退出/启动adapter完成独立根正常重启及配置读回；OwnedClient.exe壳只作旧证据，不替代本项。只能操作能证明独立配置和进程归属的实例；无法隔离即回报，不关闭日常Codex。

不发布、不合并main、不跑真实WebDAV、不改日常配置/Key；使用合成数据与自有loopback。提交推送代码/文档贡献、回交SHA/原生截图/检查与未完；最终由本机统一冻结版本、验证原包安装升级/签名更新和发布。


补验注意：新增native-regression.mjs的restart-missing直接调用showSaved，仅能证明人工通知场景下的具体错误/重试，不能当实际保存或正式客户端adapter证据；首次范围脚本当前any/some断言也不能单独证明完整目录首次全选，需要报告精确预期对象集合及读回。原门禁继续，不据代码分支推送视为补验通过。


首次范围驱动已加固：sync-setup前在独立root放expected-first-scope.json，格式{"channelIds":["夹具本地渠道ID","夹具云端独有渠道ID"],"templateIds":["夹具本地模版ID","夹具云端独有模版ID"]}。必须由已知本地/云端夹具生成，不能从待验scope反推；起始scope/initialized文件必须不存在。新驱动逐一比较完整并集、拒少选/多选/重复，并保留读回及期望输入。此驱动覆盖渠道/模版；账户/Skill/MCP仍须对应证据。61协议测试通过，实机执行结果待交回。
