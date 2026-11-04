# Windows MSVC 补验停点（2026-11-05）

本轮未完成验收。代码贡献 9bb7a169080d06393afe8df171f6b7d047e5adb3（前笔86e7b2b96）已推送 codex/post021-windows-final。基线c990717931aa37ed66ca99e42bf5cf4f470069bb，文档入口f6f169a52ac17d7cfe5f34ed4279972910a52f38。没有合并main、发布或真实同步。

## 已核对与已完成

- 使用run41913003716 / artifact13366782216原MSVC候选。ZIP、原EXE及测试复制EXE哈希一致；启动日志内嵌127543d2d0d9c2725f1a357710cd43bc69e3c44d；candidate.json的apps树与c990一致。完整CI五项适用job成功；build跳过按原条件。代码贡献仅改QA/诊断，ui/src与tauri子树均未变，非新产品候选。
- 本地71项协议回归通过。新增CDP传输和Chromium主线程/合成/GPU追踪器；原生foreground=false时在Tracing.start前实际拒绝。QA性能阶段同样在场景设置前拒绝，并在每段30秒采样前再次检查。原B=1000/60、p95≤2B、over6B=0、档位/玻璃均未改。
- 夹具离线预检通过：取两份固定SHA的历史合成原生导出，删除授权载荷并清空保存时间声明；首次云端1渠道1模板，后续各新增1，旧对象内容逐字保持。只读输入首次期望为本地1+云端1渠道、本地17+云端1模板，已在新target根、UI启动前保存。该清单没有读取待验scope；scope与initialized不存在。独立Python复核指针/文档/包摘要、ETag、集合和零授权。尚未经过产品解析器/原生UI，不能计首次全选或不扩选通过。
- 正式2.84默认Windows adapter只读发现与枚举成功，沿原10000ms超时；枚举约8021ms，返回当前用户14个日常MSIX进程。默认adapter按当前用户包选进程，CODEX_HOME不是进程归属边界。未证明一次性账号/会话，因此未调用stop/start/restart，未写正式配置。
- 本轮仅清理测试管理器PID21144。先核对独立EXE绝对路径、CIM创建时间、session2、owner，再核对实际进程句柄路径与创建时间。第一次JSON日期自动转换导致校验拒绝；第二次原生100ns与CIM微秒相差1tick导致拒绝，均在停止前。最终按CIM可表达精度核对通过才停止，记录全部保留。自有launcher随后退出，无该root残留。日常MSIX PID38560和管理器PID24140仍存活；这是只读存活观察，不是日常配置字节认证。

## 失败与缺验

当前15119会话2为Disc，原生控制此前报GetCursorPos access denied；等待用户重连并解锁。没有尝试切换/接管其他账号，没有本轮原生截图。

断连后曾误启动原性能驱动，约60秒样本终止于“performance sample lost native foreground”。invalid-disconnected-sample保留原JSON与日志；两段foreground=false、strategy.visible=false，不能拿数值作为产品通过、性能改善或根因证据。修订后两个前台准入探针均拒绝，失败JSON保留。此前高档性能失败、端口切换失败和误停日常Bun PID30196的既有原件继续保留；本轮清理通过不抹除历史越界，不声明全历史零副作用。

| 要求 | 本轮状态 | 恢复条件 |
| --- | --- | --- |
| 60444→60446、首次加载快速重开、Hide未返回即关闭浮层、正常重开/重试 | 未执行新候选实屏复测 | 15119 RDP连接并解锁 |
| 30秒静止/滚动/弹窗、后台停绘恢复、trace定因及修复前后 | 断连样本无效，根因与修复仍未完成 | 同数据/窗口/高档/原预算，保持原生前台后采样及GPU/合成追踪 |
| 首次本地+云端独有对象完整并集 | 输入已准备，原生未执行 | 启动target根，HTTPS只读夹具服务，保存合成连接后逐项核对2渠道18模板 |
| 后续新增不扩选、已有显式范围保持 | 输入已准备，未执行 | 首次通过后切later云目录并重新打开；再保存明确子集并复开，保存GET日志与配置字节 |
| 正式MSIX默认重启与读回 | 不准入，缺验 | 用户提供可证明一次性Windows账号/会话；不能用OwnedClient或日常同包实例替代 |

## 接续

恢复RDP后先重新核对会话状态、唯一原生窗口、候选SHA及实际foreground，再启动隔离perf/panel/target。不得把当前background样本或CDP截图当实屏通过。追踪器最多60秒/256MiB，仅诊断，不拥有验收通过结论。先保留失败，再在证据支持下修性能，产品改动须另建真实同源候选。

本轮TASK保持in_progress，RCP总体failed，仅候选身份与本轮清理检查passed；所有完整AC仍failed，不生成CERT。正式治理事实由集成线统一回写。本分支仅新增独有证据、TASK与回执，未覆盖旧证据或IMP/DMD。
