# r6原内容上下文与typed Stop恢复入口

源码ed89291540e1d02c886c66020ebbbdc49c2cebeb/tree3b5ba8d93c964d8e00fc93b12b385a2717616a5e，承接c259886cf/1228486fe；fetch实际main fac8573e5、治理578230717，TEAM16/IMP142/r6不变。本轮仅Windows允许集。

| 运行 | profile | raw退出 | 过/败/忽略 | ms |
| --- | --- | --- | --- | --- |
| library01 | debug | 101 | None/None/None | 61006 |
| library02 | debug | 101 | 45/1/0 | 230755 |
| library03 | debug | 0 | 46/0/0 | 165194 |
| library04 | debug | 0 | 47/0/0 | 179018 |
| product01 | debug | 0 | —/—/— | 144616 |

首library01 E0603/raw101为production guard模块可见性，无新EXE/未测试；按父模块重导出精确类型修复。静态发现外层模板把预期捕获替换写成1[USER]，在测试前以caps[1]闭包纠正，原期望保持。library02编译后45过1败：测试bundle root包含../../..，被原native路径检查正确拒绝；仅canonicalize实际测试根，不改生产路径断言。library03修正后46过；最终补source pin真实写/替换/错hash负例及旧unqualified preview测试限定后library04 47过。所有错误/实际执行source/参数/raw/stdout/stderr/EXE摘要保留。

新增实际原上下文factory：原Source/Owner/未resume/Start动作成立时capture，保管准确original根进程代次、native创建FILETIME转UTC毫秒、Profile/H、原routes/config native pin及4固定官方审计源码准确hash/ID/all8；Stop只从原Owner获取原Stop/Restart子窗action，需actual NormalExit重核原Job全家族，PID+creation去歧义，不接受caller PID/home/time数组/JSON/boolean/Clone。notBefore来自原native根创建时间毫秒floor、notAfter仅在实际停止族重核后取得；不是新的mutation deadline，窗口/未知拒绝保持。

固定HOME audit投影只在已验证identity-redactor域准入：实际固定Bun115MB/原sha只调用原官方redactUserPath(path).slice(0,256)纯库，对8支持路径逐字相同，含C:/、反斜线、自定义根、中文用户名、Unicode长s前缀不误折叠；4敏感/token/截断路径明确拒绝。源码4份逐字对固定bundle和内tgz，源码位置/hashes原件单列，未改官方。合成source pin对write/atomic replace原码拒绝且bytes不变、wronghash拒绝，三次检查actual original source pin，不启动服务/模型/网络或写官方/日常配置。

ProductionStop和NativeContentAssessment均为私有非serde/nonClone原对象边界。先实际Source/Owner/NormalExit/SupervisorAbsent/StopAccepted/current/listener/action/routes重核，再从canonical唯一Ready/PreStop/Stopped选择实际3帧；原before+3帧按原sharedpolicy在完整BOTH-tree before/after屏障内评估（全部original/custody/历史work/native archives/current帧）。scope/series/原deadline绑定assessment先装core，之后才capture_retirement转移current frame。旧未评估preview现仅cfg(test)存在，不能让生产提前转移当前帧。

新CaptureSlot.restore_stopped_existing仅内部恢复/history入口：原Ready tuple和Stop绝对end相等、original NativeLease新取得并observed Stopped后，qualified batch current退休/original返回、整树确认、显式release/两稳定范围证明、原件不可变history按guard前后actual typed条件检查。没有把组计数或Policy DATA当mutation许可。返回内部restored_history标记而非ClosedEligibility；source资格/receipt/Closed完整canonical记录、handle所有权闭环、实际process/supervisor完成history、真实Status5/dispatcher和继承接管尚待。

本轮真实production Source/Owner/Ready/NormalExit/Stopped/qualified restore factories仅编译未执行；不能把47组件回归、Bun纯库投影或旧GUI空broker证据当实际原包生产闭环/原10秒Stop证明。原typed上下文不可伪造测试，不以假Owner/正常码制造正例。实际ordinary desktop固定官方生产H-S-D/后继/Status和guard反复完整来源核验的预算还需实证。公开Start无条件拒绝；官方config/auth/Grok与动态目录非artifact变化仍未知拒绝，first-use/ownerreceipt/原缺席Codex安全退根/普通MSI/CAP-trust/历史清理保持未完。不旧raw5/空GUI/相同1314/W2/双SID/发布。

本轮244场景完整私有ZIP核验，3完成纯库/合成pin场景核fresh准确路径/全文件hash/目录membership清理；241失败/pending及81个D根保留。2544份canonical/快照/history原字节另归raw-scene-records。ZIP非NTFS身份/安全克隆，历史拒绝删除不重试，overallCleanupComplete=false。完整继承状态与首错/制品/source归final-status。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77794816 | 4713b35b729030529e3c01f8aa758593657152e50fe7565265d624545b232085 |
| ocxd | 5398528 | 533403ac239e9a6ceb94918634b5e4021fa935a00a0db9ec7443ffe72f3f9c25 |
| ocx | 306176 | 6627a82aae71364528a6fa814fe5070a8b0590bdeb8259a618aba4636164f97c |
