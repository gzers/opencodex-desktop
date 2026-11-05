Windows正式默认重启：本地运行包第二版候选验证通过。

原发布2.84.0第一处桌面GetOwner枚举10秒超时；第一版同句柄SID/path/create查询已修复，真实桌面92676退出、新48960启动，65547读回，但第二处app-server8秒查询超时，扩展约13秒仍windows_enum_incomplete。失败与原生WebView截图均保留。

第二版将同句柄原生身份查询扩展到app-server，原窄匹配正则、stdout解析、8秒上限、目录分类、启动时间二次确认与停止逻辑保持。Administrator默认查询1039ms、扩展940ms，4候选/1app-server，完整目录fresh、1986ms。当前账号只读1061ms。真实原生MSVC管理器30804/CDP9461通过现有重试重启驱动候选默认服务56436；旧正式主进程48960及同代子进程退出，新主进程92496与6子进程启动，真实UI显示Codex已重启；策略摘要前后ca86fc相同、contextWindow65547、config.toml3334字节和测试loopback路由读回。其它账号桌面代际无变化。

候选基于发布包2.84.0的独立副本，明确标识local-runtime-source-candidate-not-published2.84.0；没有注入restart/adapter/IO，没有发布或替换原包。桌面补丁runtime-windows-owner-probe.patch与本次app-server补丁runtime-windows-appserver-owner-probe.patch按顺序应用；后者git -c core.autocrlf=false apply --check与精确字节重放通过。PS5策略正反例通过，Bun8项通过。

边界：本轮只精确停止已核对PID/创建/账号/会话/路径的Administrator隔离旧测试服务；正式MSIX退出由授权默认流程执行。模型源仅GET/v1/models的合成loopback，不运行推理/真实同步，不上传配置/auth/历史原文。15119日常Codex38560及管理器24140同代保留。本轮隔离验证无其它账号桌面变化；历史误停日常Bun的旧失败证据仍保留，不能宣称整个长期任务无副作用。

未完：把两个运行包补丁纳入上游并生成集成运行包，在同一最终提交完成跨平台检查及统一发布门；本贡献不替代AC-P21-06、不合并main、不发布。当前测试服务56436、管理器30804和正式92496保留供复核；private-backup仅留隔离测试目录。不能将磁盘/管理器读回扩大为Codex模型选择器或推理行为验证。原硬件超帧预算保持失败参考，按用户例外接收材质功能，不宣称性能修复或60fps。

代码分支codex/post021-windows-final，候选补丁与验收工具提交：8b9cca86a71827aca50da076eb32fd17479b1512。产品apps树保持a8bb18f9ff2781dbbfe67f91473a298f3cd74ae6，正式MSVC候选仍run41944607861/artifact13380187718，EXE fa9df83a55153f764673781e85167093d20602fd2e34742c69f6b2f5b37f9c68。
