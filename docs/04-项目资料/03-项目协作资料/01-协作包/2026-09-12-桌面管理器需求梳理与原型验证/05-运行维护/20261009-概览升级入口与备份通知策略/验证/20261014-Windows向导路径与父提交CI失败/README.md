# Windows 向导路径与父提交 CI 失败

2026-10-14。Mac仅后台。代码feature/0.1.10-integration-candidate@51a35effb3bea7b8f49a73e508e45152881790f7，父f545cb33。设置安装向导之前仅按POSIX的“以/开头”判断，Windows来源DTO中的合法C/F盘及UNC前缀无法下一步；归属提示仅startsWith，D:\data-sibling或macOS的data-other会错误标为数据根内。该问题由本轮真实组件挂载复现，不将源码线索当Windows真机结果。

## 修复与范围

installPath.ts按app_about平台事实识别完整盘符或UNC服务器/共享路径，拒绝相对/盘符相对/根相对/根目录/设备命名空间及含./..或控制字符的路径；macOS保留绝对路径与原系统保护目录拒绝。平台事实未就绪时不猜Windows，事实到达后响应式重新校验、保留用户原文本。输入路径不转换或改写，明确确认后仍原样发给install_runtime。

数据根内外提示改为根身份+逐目录组件比较，Windows处理分隔符/大小写，不把同名前缀的相邻目录或另一UNC共享标为根内；不满足语法时不猜归属。该提示是词法比较，不声称canonical文件系统/链接/网络共享实际可写已验证。

Windows系统保护根依赖真实宿主SystemRoot/ProgramFiles/ProgramData等，前端不猜固定C或D盘；后端validate_install_target的系统保护/链接/权限/原子安装与脚本确认不变。测试模拟原native拒绝，验证原请求一次、allowScripts=false、失败就地显示且不自动重试；它不替代后端或普通令牌实机验收。Tauri全目录在aae/f545/51a原字节相同，未改ACL/系统目录规则、存储根/schema/查询预算或许可门禁。

## 回归证据

首次20项对父f545组件14通过/6失败，新实现相关3文件42通过。扩充确认/后端拒绝与平台迟到后，最终同一测试文件6384字节/SHA78efe2309f831cc7fa0a5ff4b7b27f84ed1d4f576b7e7faf9c2afc67387f0120，对父生产SettingsRoute实际14通过/8失败；最终实现前端99文件/533通过，新增22项均包含在全套。基线红测试finally精确恢复最终生产字节，提交blob/测试SHA再核验。

覆盖C/F盘、含空格、混合斜杠/大小写、UNC、两平台同前缀相邻目录、相对或非普通组件、macOS保护目录、平台事实迟到、用户确认后的路径原样及native拒绝反馈。类型检查/生产构建/diff通过；4条动态导入和jsdom canvas警告保留。此次仅SettingsRoute、纯UI路径helper与挂载回归，未重跑本地Rust/原生/性能或系统选择器，未执行真实安装/迁移/第三方脚本。

## 父提交的失败继续保留

f545cb33 PR CI38533779767已completed/failure，Mac35组/900通过/6忽略，Windows库527通过/3失败/4忽略，后续组未运行。失败为official_version_source三个success/nonzero/timeout夹具；日志显示Job assign/resume在12–26ms内返回，约2.02–2.03秒返回Timeout，查询结束时独立control Node、父/子脚本和ownership-ready标记全缺失。fixture最终无法建立预期句柄，不能据此证明产品进程树收尾通过。完整原ZIP/CRC/摘要、失败上下文和实际PR checkout封存；不重跑覆盖它，不降原2秒/4秒清理/并发/所有权断言。

独立control未加入查询Job，因此还需Windows固定源原始进程/调度/Node启动/标记I/O证据；后端从aae到f545/51a完全相同只证明来源没变，不足以认定环境波动、Defender或UI补丁导致。父push38533772616最新job观察为回归成功但release build仍进行中；这不解除PR失败。新51a push38535387208/PR38535398225最后仍in_progress，两frontend job已success，后端未完整终态，匹配run/job原件单独保存。GitHub connector工作流列表仅PR首屏，push状态通过明确run_id查询，不推断遗漏即不存在。

## 分支、制品与后续

PR4已由原有owner CLI账户更新title/body并返回核验draft/open/head51a/merged_at=null；connector可读取但写返回integration403，不扩大权限，CLI最初EOF后先读回确认未改再按已授权title/body更新。没有改base、reviewer、state或执行合并/重跑。

最新源码51a包含Windows路径及父f545真实进度修复；旧已验证候选38524086177/artifact11875775415只归aae，不含它们。保留Windows固定aae当前验收，不操作其唯一实例；新SHA须完整CI/失败处置后才准备新候选/单实例协调复验，当前不派新包。门禁清单已更新最新源码/旧包对应，main仍eae、文档docs/governance-main，五共享文件一致、两个未跟踪协作探针保留。

原macOSCPU+1.817629pp/RSS+117.008965MiB失败、冷热每版各≥20、启动p95≤100ms且≤5%、10minCPU≤0.5pp/RSS≤10MiB、主线程≤50ms、有效缓存100切页/10重启零自动网络/查询进程、真实24h/6h周期及全部双平台/安装保留/更新备份事件矩阵继续开放。损坏保护关联不能从版本号猜造或强制解除；没有FZ50许可不执行脚本。

原TASK3878字节/revision2/in_progress/pending、空验收保持；RCP总体failed、goalactive。Mac原生前先协调Windows实例0，无真实数据迁移、日常0.1.7/0.1.9替换、稳定核心晋升、自动合并、签名、公开发布或完成证书。
