# 协调尾部原包提前失败与进程读取竞态修复 r1

revision: 1；2026-11-13；IMP-34 P24-04；失败原包源码main0c988fc89；修复main4c2a826aa、回执留存main382cf593fd24e1c957690bfaec7e4f0545ad8e1b。accepted=false，非固定候选；承接 [镜像路径优化](image-path-admission-r1.md)。

## 原包结果与限制

固定干净main0c988fc89、24关键源与release库宿主dfac55863e94751d4a7aa278dd299dc7a133df2bc62004d014df97ca22888a35，构建6m44s。开跑前假设要验证镜像优化及Stop协调收尾，唯一一轮运行却在首次wait-os-absence提前失败：Start7436ms、同连接验签Status229ms通过；procargs读取返回EINVAL，manager Stop失败duration1863ms，测试随后断言退出。独立Head为StopPrepared5，未取得正常退出、root恢复或第二轮结果，不能把早错当性能通过。

现有测试在断言后才写Stop回执，故本轮summary没有stop／stopElapsedMs，不能从日志编造DTO或具体读前后BSD状态。仅取得两段Start期间协调tail（完整来源456.862／455.266ms），不是Stop收尾耗时。本轮不重复执行，也不声称历史EINVAL已经证明“进程缺席”；它只暴露了代码在读取出错时跳过后置身份复核的路径。

失败证据 [.adg/evidence/0.2.24/client-tail-native-r1](../../../.adg/evidence/0.2.24/client-tail-native-r1/manifest.json) 保留原构建、原日志、初始假设、缺回执限制、40原文件和独立verifier。Original／Ready／PreStop／root provenance链与5events离线核验，未生成Stopped／restore／retirement。观察到的PID+birth与自有exe已缺席、导出主端口独立内核快照缺席，但这些不补造exit0证明。精确新3协调文件留字节并核first-seen identity／exclusive后清理，globalL原身份／保护／bytes保持释放；40原文件逐字归档后清Profile／程序／exe，全部材料再逐字归档后清临时证据根。官方2.99及原资源不变，沙箱实证拒绝日常HOME／外网，未强杀、重试失败轮或业务请求。

## 修复

Mac PlatformHost.process以私有process_snapshot夹住image／argv读取，无论fields成功还是失败都做后置原生BSD观察。只有bsd确切n=0且ESRCH才产生None；fields的EINVAL／ESRCH本身不代表缺席。后置PID、UID、birth变化拒绝，同代仍live保留原读错，后置观测失败保留错误；初始／后置同代zombie返回typed recovery_process_exiting。非Stop查询仍拒绝该未知状态。

仅已有正常Stop等待循环将typed exiting映成Unknown，沿原remaining与50ms循环继续等待，没有延长原deadline、提前产生Stopped或跳过不可读活进程。fullscan、manager／record复核、ports及所捕获listener集合、原代次ExitWatch的exit0和后续endcheck均保持。快照观测缺席不是正常Stop证书。另将exact Stop DTO及elapsed在相关测试断言前持久保存，原断言不变，避免下一次又丢首错回执。

## 验证

两项新测试覆盖10种before／after／fields分支，且以私有管道协作使真实自有/bin/sh子进程正常exit0并reap后才读参数：该读失败，后置BSD明确None，共两次身份观察，未发送信号、未当官方Stop权限。活进程读错、UID/PID/birth变化、observer未知、初始／后置zombie均保持各自拒绝或未确认结果。

owner模块20项（含新2项）、命令4项、原退出证明10项、服务／launcher配对6项，共40项不同受影响测试通过；owner／命令各1项原有忽略保持。最终严格all-targets Clippy、fmt及独立include测试文件格式、diff检查通过。初次测试模块path解析失败、随后Clippy items-after-test-module失败均保留，分别改private cfgtest include／模块位置，未公开unchecked seam或禁用lint。最终新2项再编译执行确认。actual diff只读补审无新增阻断，不冒称审阅者运行测试。

修复证据 [.adg/evidence/0.2.24/client-process-snapshot-r1](../../../.adg/evidence/0.2.24/client-process-snapshot-r1/manifest.json) 含测试命令／压缩原日志、失败说明、三源与测试二进制摘要、审阅边界及unit临时根逐字归档清理。本次修复后尚未运行新官方服务轮，不把确定性子进程测试代替原包完整生命周期；后继应固定本源码／新release宿主，一轮取得原30/5/10与tail分段。此前RootAbsent18但Stop11345ms超限的失败也保持。

根中断、动作可启动资格、缺Ready、一般客户端图路径／完整进程集合与刷新、NAME、复杂代理、WindowsW1W2、官方交付验证及固定候选等IMP34完整范围不缩减。未改日常配置／Shell／已安装app／官方包、未真实业务同步或模型请求，未发布，目标active。
