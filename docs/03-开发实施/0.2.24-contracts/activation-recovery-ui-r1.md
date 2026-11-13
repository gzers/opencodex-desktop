# 0.2.24 中断版本切换的恢复入口

revision: 1；2026-11-11。共享归属路径修正 mainaff0d9952，产品恢复入口 main063af3a96。本合同补 program-activation-r1/versioned-install-r1 的 IPC 和页面消费者；进度归 IMP-34。

## 入口与用途

设置 → 安装配置 → 程序安装与卸载 →「检查安装恢复」。进入只读观察当前I/runtime/.selection-pending.json，不创建程序锁、不改变source记录或推进原操作。没有pending明确显示没有待恢复的版本切换；记录或目标异常时显示实际拒绝，不猜是未安装或修复成功。

预览显示原版本/目标版本、程序位置及配置保护目标。选择「继续完成安装」或「回退到原版本」，原来未安装时回退文案为「撤销本次版本选择」。确认前重新核验服务已停止并完整保护实际HOME当前配置；恢复过程不改配置内容、不自动启动服务。关闭只退出观察，执行状态属于持久store，路由重挂载能查看同一请求，不生成第二次恢复请求。

## 实际接口

| 命令 | 输入 | 输出/事实 |
| --- | --- | --- |
| runtime_activation_preview | 无路径或操作输入 | Option<RecoveryPreview>：attemptId/from/to/programRoot/configPath/confirmationDigest |
| runtime_activation_recover | attemptId、confirmationDigest、decision（complete/rollback），拒绝未知字段 | RecoveryOutcome：同一attemptId/decision及读回的selectedVersion；不因“无pending”直接推断先前成功 |

preview的摘要绑定I/D/H、严格原pending字节和入口/current/runtime.json三个当前值。即使另一写者把同一事务推进到合法后像，旧确认仍需重新观察。当前配置在执行时备份保护，不接收页面传入的原生配置或密钥。预览不需要先停止，但执行只接受真实Stopped，Owned/Unknown均拒绝且不主动强杀。

recover持有全局写入准入、Desktop安装租约、I程序锁和当前source锁，排除进行中的管理器安装/待重启；拒绝未完成的旧保护事务。配置保护失败零恢复写入；服务再次出现、配置或备份变化、其它目标包含外部值均保留原pending与原件。后端回执必须匹配用户选择的原/目标版本，前端再核对操作ID/决定/版本；丢失或错误回执只显示待核对，不声称成功或隐式重试。

## 归属路径接线

RuntimeExecutableProvider::observed_program_identity 用于已准入程序的路径观察。版本化RuntimeHandle复用缓存及current/入口新鲜度检查，不在高频路径额外哈希全部Node/Bun。SystemServiceOwnerProofProvider的进程匹配以该身份派生package_root/node/bun，核对实际程序根，拒绝flat路径或选择漂移；既有PID代次、用户、HOME、记录、监听、官方管理器和完整启停前程序准入不放松。Windows W0生产生命周期集成和真实预算验收不由本次路径测试替代。

## 验证与未完

.adg/evidence/0.2.24/activation-recovery-ui-r1/：30项后端定向测试（命令4、事务6、新安装4、选择7、归属7、消费者2）；显式官方包和真实服务case各1项不在本轮运行，保留其ignored标记。90项前端测试及类型/格式/严格Clippy通过；实际共享组件以浏览器内存IPC夹具验证1024×740、580×480预览与回退结果，未调用原生接口。首轮监听沙箱拒绝及双Pinia视觉夹具故障均记录原因，产品断言未放宽。

这是中断版本选择的恢复，不是对任意损坏镜像的重新安装。新产品设置入口已避开旧flat恢复器；同版本镜像修复及原来用户要求的完整诊断/恢复场景仍在P24-01，不能因此删除范围或关闭整项。原型对照和固定原生候选、CLI/卸载、Windows W1/W2及名称剩余项仍待完成。本轮不运行日常配置、业务同步或模型请求，不改官方2.99或发布。
