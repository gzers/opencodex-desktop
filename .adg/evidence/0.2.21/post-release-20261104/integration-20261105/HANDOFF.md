# 最新构建候选与CI（2026-11-05）

已取得MSVC原候选run41904417657/artifact13362926947并核验ZIP/EXE哈希、PE架构、构建87d5137和请求head31a28bfb4源码绑定。Windows首轮实际EXE冒烟通过；整轮后续在安装恢复后的WebDAV旧configured断言失败。c99071793仅修剩余ready消费者并纳入CI，71协议测试通过，完整Rust及ui/src与候选源保持一致。Node22及后续普通账号矩阵尚未执行，整轮不是绿；RDP性能、正式MSIX隔离重启和发行安装升级等门仍未齐。最新提示词固定该候选下载入口，产品未重建冒充新源，EXE未在本机执行。下面保留历史阶段事实，不覆盖本段。

# 最新接续状态（2026-11-05）

用户恢复目标并接受RDP、预算不变。代码31a28bfb4已完成本机面板恢复/加载来源修复、Vue Proxy探针标量投影、Windows驱动合并及MSVC候选留存。产品apps树ab09d800b27ce13e17c46ce68f9ae38f7d4fc21a与Mac原生6a3c7ef2d一致；63协议、1291前端、Rust面板12/Clippy/构建及Mac原生15通过，Rust全库1217通过/0失败/4原条件忽略。Windows新源切端口竞态未验证，RDP性能三项失败保留，正式默认重启仍缺。新回交54932fe8a已版本化保存，corrected RCP4将缺验及隔离失败真实标明；保留旧矛盾原件。报告披露误停日常服务PID30196，缺前后字节，不能宣称零副作用；本机未执行补偿操作。最新事实见IMP-30及两份audit/validation summary；下面为历史阶段交接，不覆盖本段。

# 双线集成交接（未发布）

Windows最终原始回交68c034362已接收，46原始blob/40manifest项SHA一致。详细采用边界见windows-final-code-review/evidence-audit.json。仍为RDP/GNU，正式默认adapter未重启；首次范围0渠道/17模版只能证明部分选择，主模型流程以旧WebDAV断言超时结束。Rust17个Node环境失败均被18定向重跑覆盖，不记为全workspace单次通过。来件RCP总体failed，却把MSVC/物理性能/正式重启checks误标passed，与正文冲突；原件隔离到received-runtime，不采用权威回执、不关闭AC。EXE启动脚本从另一checkout复制，缺原始内嵌commit及哈希拷贝链，运行身份待补。四个新TASK仍in_progress，无新CERT。

代码codex/post021-sync-fixes / c71a8cf1eef28db5de9ba82b5b599bdc1336678f，PR #69草稿。产品源23440509452fc1241324a8ba02ebdd6e52822c18；后续HEAD仅增加两份smoke脚本，apps树0eec77398bbd1cbe49f88dd3bd5e8046266e14c1一致。文档采用仍以DMD-POST021与IMP-30为准，未来原型脏文件未纳入。

Windows代码4b0c357b1已合入；文档911d51514的328独有路径字节导入，原manifest177+137项SHA一致，六TASK/六RCP保留失败及未完。材质恢复的根因与原生截图保留，不把性能失败改成通过。

本机结果详见validation-summary.json。集成前后端1289/1215及定向11、Clippy和构建通过。Mac六材质场景、渠道12、最终同步27与面板15检查通过；同步仅隔离HTTPS GET，业务状态摘要保持；面板为官方2.84原包真实子WKWebView，无脚本替换响应。返回面板410/353ms是复用返回观察，不是冷启动或首像素。Mac管理器通知中心会随外部侧栏点击收起，跨路由带通知的假设不成立；该原生覆盖未宣称通过，Windows既有/最终补验仍负责相应场景。

保留失败：100ms Rust超时测试初次并发请求未到服务，单项和低并发完整复验通过，未放宽deadline；亮低初次无WebArea；测试server stdout放在日常home禁读范围导致Node启动失败；缩写build SHA拒绝；通知入口/默认路由假设失败。最关键的实际产品失败是Mac首次导航前重开读取nil URL崩溃，cf412c336/234405094已修并补原生面板。另一个加载失败来自测试沙箱屏蔽系统mDNSResponder Unix socket：最小WK程序NSPOSIXErrorDomain1，同规则仅放行该socket即成功；最终产品也已独立通过，不用最小程序代验。所有尝试独立UUID根且保留原始报告。

发布门仍未齐：Windows最终同源CI/原生回归、RDP未通过的材质性能（需要物理桌面和MSVC候选包补验）、正式打包Codex桌面默认重启adapter、新版本冻结和原包安装升级/签名/更新通道/回退。没有发行版号变更、main合并、tag/发布、真实WebDAV或日常配置/Codex重启。当前TASK/DMD/IMP仍in_progress，无整版证书。Windows接续用WINDOWS-FOLLOWUP.md；用户已确认有本地Windows环境，采用继续交Windows对话补验；当前宿主工具仅列本机对话，未连接Windows宿主，未实际发送。提示词交由用户粘贴，不据此宣称Windows已开工。


最新交付代码f9def3300640f0871a93b6bbcec55de646e00567仍为同产品apps树，仅校正改名保存的恢复日志/写入前备份探针。35测试通过；Windows实际重跑待证。Windows最终交接以WINDOWS-FOLLOWUP.md新HEAD为准，旧CI失败保留。


接续CI显示名称旧合同失败：bd7c0fe8c实际原生已过剩余A保存及备份、排序/转移/对齐，再于021改名raw.name旧值断言失败。3aa789a6e仅适配name-only的持久化名称，整组59测试通过；产品apps树仍不变。原生重跑、后续未执行检查、Mac runner补跑和Windows实机仍未齐，详见ci-display-name-probe/analysis.json。旧source身份保留在原始证据，最新交付以WINDOWS-FOLLOWUP.md为准。


最终探针接续HEAD9850dbbae：原生新失败为持久化显示名称旧断言，3aa789a6e已修；继续逐项审计发现尚未执行的WebDAV旧探针与本轮启动只读检测冲突，9850dbbae按采用合同预先适配。新build必须有真实失败观察/修订/时间，只有config_sync_observe可额外准入，仍拒上传/下载/应用。历史独立运行ID界面保持旧预期。完整60测试通过，Windows实际重跑仍待，不把源码审计预测写成已复现。产品apps树与234405094不变。


补验注意：新增native-regression.mjs的restart-missing直接调用showSaved，仅能证明人工通知场景下的具体错误/重试，不能当实际保存或正式客户端adapter证据；首次范围脚本当前any/some断言也不能单独证明完整目录首次全选，需要报告精确预期对象集合及读回。原门禁继续，不据代码分支推送视为补验通过。


首次范围驱动已加固：sync-setup前在独立root放expected-first-scope.json，格式{"channelIds":["夹具本地渠道ID","夹具云端独有渠道ID"],"templateIds":["夹具本地模版ID","夹具云端独有模版ID"]}。必须由已知本地/云端夹具生成，不能从待验scope反推；起始scope/initialized文件必须不存在。新驱动逐一比较完整并集、拒少选/多选/重复，并保留读回及期望输入。此驱动覆盖渠道/模版；账户/Skill/MCP仍须对应证据。61协议测试通过，实机执行结果待交回。
