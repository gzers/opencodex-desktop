# Windows 最终eae切根准入诊断

2026-10-12，唯一最终 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc` 普通应用PID37276。原件归 `.adg/evidence/OCX-0110-20261012-WIN-EAE-ROOT-ADMISSION-DIAGNOSIS/`。本轮只读当前来源/根配置并请求实际状态刷新投影，**没有调用切根或迁移命令，不算自定义根成功**。

## 当前原生状态

23:27:26（UTC+8）原生返回同时为kind=unresolved、matrix.runtime=not_found、source=live，pid/port为空、health=unknown。before/after根配置完全一致，runtimeActive=true仅表示当前绑定与实际活动路径相同，不能据此声明代理已运行或可安全切根。

状态命令成功返回DTO，不代表它内部的collector.refresh()成功；这不是两个可互换的布尔值。本轮未把source标签Live或投影NotFound重写为准入验证成功。

## 精确源码的准入链

核对eae源的Git blob/SHA及函数行号，引用真实实现位置，不复制源码到文档分支：

- `infrastructure/status_source.rs` 的真实来源在未解析到入口时返回CollectError::Unreachable。
- `modules/status/mod.rs` 在该采集错误且source未解析时把矩阵投影为NotFound，但refresh结果仍保留Err。
- `commands/mod.rs::status_snapshot_from_collector` 忽略refresh错误，以当前矩阵/facts和Live来源标签组装DTO；公有状态读回没有暴露本次成功flag。
- `commands/data_root.rs::change_binding` 单独检查本次collector.refresh().is_ok()及idle状态；verified=false由binding_runtime_blocked返回RuntimeUnverified，先于绑定提交。

因此，当前未解析的运行来源并未建立原合同要求的“本次成功验证且空闲”的切根前提。此结论是当前原生来源与精确实现的**准入推断**，不是本轮新切根命令返回值。此前0a837源确有真实runtime_unverified拒绝原件，本轮不把它重新标成eae命令结果。

不放松fresh/verified要求，不把缺失入口自动改成已验证停止，不注入collector成功结果。后续成功切根仍须建立真实运行来源/当前采集前提，并按原合同验证绑定与重启读回；第三方包脚本二次确认未收到，不为取得该前提执行它。

## 文件及生命周期边界

只读诊断前后数据根绑定文件、偏好、整个最终源周期文件SHA分别一致；没有改通道、自动检查true、deadline/timer/时钟或发手动更新。精确应用/监测PID37276/3920的实际句柄与创建时间再次核验，仍一个应用，日常EXE SHA43e06bc3…保持。

7个原始文件集合/哈希与当前DTO、五处源码契约、根/偏好/周期不变断言冻结并重验通过。诊断校验通过仅证明前提与证据可追溯，**切根未尝试、未通过**。不将状态投影诊断作为安装、全根矩阵、性能或24h周期证据。

原TASK/脚本/安全基线与安装环境、完整原生/更新/事件/性能/真实周期等门禁继续开放；原预算和既有failed保持，IMP in_progress，不降低门禁、不提前关闭或发布。
