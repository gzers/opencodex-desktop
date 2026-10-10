# 卸载核验与限定范围终态事件

代码 feature/0.1.10-maintenance@7d5b2e9d581a66ff18545a81d996b4edcd1a6aec，已推送；未合入 main，版本仍 0.1.9。本记录是 macOS arm64 源码回归，不是原生卸载、Windows 安装器或发布验收。

卸载请求先校验范围及明确确认，再取得租约或停止代理。由持有写者准入和运行变更租约的工作线程负责终态投递，IPC 观察者取消不提前释放所有权。工作线程范围内的保护核对、停止失败与卸载步骤错误均投递失败；准入前的取消 / 无确认 / 更新占用 / 写者冻结拒绝仍只即时反馈，未强行向冻结数据根写通知。

成功须至少一项步骤实际成功、无失败步骤、残留证据非空且全部 cleared。命令 exit 0、全部跳过、unknown 或空证据均不算成功。后端历史、通知和 UI 共用此口径；未核验时不显示 100% 成功。选择保留的 HOME 数据不属于包体残留目标，真实 fixture 的包体 / 入口删除并保留 config.json 已通过。完整卸载的 Windows 原生残留矩阵仍待验。

新增 runtime-uninstall-failed / succeeded，用户触发、runtime / uninstall / execution / local；成功只解除同根世代及相同候选的对应失败，不解除安装失败，不自动已读。候选散列来自请求选项与已解析来源 / 版本 / 路径 / 计划对象；通知不存这些原始路径，不声明包内容指纹身份。来源版本在移除前捕获，不能用移除后默认版本写历史。跨重启去重、错误候选 / 动作隔离、未核验成功拒绝与周期触发拒绝已通过。

生产注册表现为 8 triggers / 28 jobs / 56 events（54 active / 2 planned）/ 15 policies / 62 emissions / 8 scheduling / 7 feedback / 12 paths / 4 cleanup。未增加自动卸载或定时器；原型独立事件表不受此计数替换。

- cargo test：33 组，756 通过 / 0 失败 / 4 忽略。
- cargo fmt --all --check 与 Clippy all-targets -D warnings：通过。
- UI 93 文件 / 438 项通过，类型检查与生产构建通过；卸载定向 8 项通过，不累加到总数。
- JSDOM 两条 canvas getContext 未实现提示保留在原始日志，不作为原生渲染证据。此前定向日志另存，不冒充最后全量源码证明。

命令、退出码、身份与原始日志见 [.adg verification.json](../../../../../../../../../.adg/evidence/OCX-0110-20261010-UNINSTALL-VERIFICATION/verification.json)，摘要见同目录 SHA256SUMS。准入前错误注册、全部修复动作、损坏保护回执处理、Windows / macOS 原生安装更新卸载、配对性能与真实周期继续待验。TASK-182 / IMP-20 保持 in_progress，没有完成证书、稳定核心回写或公开发布。
