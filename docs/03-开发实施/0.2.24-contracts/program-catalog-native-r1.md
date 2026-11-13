# 0.2.24 明确版本登记与Mac占用证明

revision: 1；2026-11-11。代码 main0954ec6f0（登记）、main2ab15a9e0（占用），承接program-removal-core-r1。实际进度归IMP-34。本批还没有替换卸载窗口和完整官方清理编排。

## 版本集合来源

首次固定供应和新安装候选经过源/镜像核验、版本目录原子发布之后，调用installer-only program_catalog::register。记录位置I/runtime/image-admissions/<version>-<imageSha256>.json，绑定实际I、CurrentSelection及原官方归档摘要；创建式持久写入，重复只接受同义原记录，未知内容不覆盖。

登记发生在发布后的入口版本探针之前。候选随后失败时，其已发布镜像仍有明确原登记，继续重试不会被迫从任意清单猜归属。未能完成登记的异常仍如实保留未确认镜像，不启动或自动接管。当前版本只读启动不扫描目录补登记，不兼容/迁移旧开发镜像。

program_catalog::inspect先读严格、有界、单链接登记，再重新核验每个镜像及原来源，之后比对实际versions子目录集合；未知行/文件名、重复版本、未登记目录、被改镜像或缺失镜像全部拒绝。revision仅读取并摘要登记原字节，用于已经冻结的卸载在部分代码被删除之后继续绑定原来源，不因此证明剩余文件正确。

program_removal::prepare_registered使用上述集合生成原卸载范围，并将catalog revision写入原计划。执行前、执行阶段和显式继续都复核登记未变；存活代码仍由原文件冻结清单核对。卸载后登记原件保留为证据，不拿它自动重装或推断未删对象不存在。

## Mac程序不在用证明

SystemServiceOwnerProofProvider::require_program_unused先核对同一启动上下文及当前H的真实Stopped证明，再以OS原生进程枚举核对同UID所有相关程序。I是Mac用户专属程序根，不能把这一结论跨用为Windows共享I的全SID证明；Windows调用明确返回未接入。

Mac的program_processes用内核UID过滤列表，先读取执行路径再筛名称，发现I/runtime中的重命名程序仍作为占用。Node/Bun等其它位置的解释器也读取原argv，可识别它们在运行本I中的脚本或其它HOME；未知程序路径/权限/参数不被静默跳过。当前H无记录不能替代所有版本不在用。

proc_pidpath可能对已移动/删除的存活程序路径返回ENOENT。此时读取有界、zeroize的KERN_PROCARGS2内核执行头（不使用argv0或环境冒充image），用其明确原执行路径分类；PID owner/创建时间在路径/参数读取前后复核。无法读取仍存活对象时拒绝；只在原生证明PID消失或zombie时认为已退出，不把一次路径错误当退出。更新候选切换在Mac也消费该证明，避免只核对当前H后丢失其它实例。

配置/Key文件不在此次进程观察写集，原argv/env仅为既有HOME归属和参数证明读取；不日志打印其它进程参数。开发中曾加入一条过宽诊断，留证前将该行替换为明确omission标记，不留相关argv；失败断言/其余原输出保持，未伪造未删减原件。

## 验证及后续

.adg/evidence/0.2.24/program-catalog-native-r1/：72项最终回归（catalog2/removal7/activation7/supply33/install4/owner10/command9）、格式及全目标严格Clippy通过。native测试启动复制的临时sleep并仅对该隔离副本ad-hoc签名，证明程序更名/其它HOME使用仍阻断、unlink后存活不当缺席、退出回收后不再占用。最初复制系统二进制受macOS签名校验直接SIGKILL，测试夹具先证明稳定存活后才删除文件，不改产品断言。

另固定官方2.99真实离线供应/目录核验、登记集合及重开1项通过，耗时约86秒，不执行Node/服务，不作为30/5/10秒原生预算验收。网络只开放已有原生回环监听测试，其它出站拒绝。原失败及实际二进制/归档摘要保留，合成/真实资源临时目录清理，无日常配置改动/业务同步/模型请求。

下一步由卸载产品编排冻结此登记及真实unused事实，并绑定终端撤销/选择范围/官方清理/继续入口与明确重装。跨用户Windows原语、CLI/PATH和安装器仍按TEAM独占回交，不以Mac枚举通过代替W0/W1或最终W2。
