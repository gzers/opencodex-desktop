# 待确认的产品范围：后台生命周期结果与有界停止

## 已验证事实

1. 第21轮sourceab5a9a3d8：272份证据全符，26过/1失败，accepted:false。A真实管理器默认自动55581，原预算内running/healthy；原版实际Codex配置指向该端口。B固定冲突无自己的记录，但管理器无诊断。清理manager-stop不能在预算内完成；最终独立严格账户/进程/ACL清理通过。
2. 同环境原版直接CLI对照42477952450/source6fe6f16f1：A14.870秒真正ready，自动56349；另H固定占用该端口的start6767ms返回code1、原文“Proxy already running (PID unknown, port 56349). Use 'ocx stop' first.”，A记录不变、B无记录。原CLI stop10006ms仍未退出、A记录仍存在。原ordinary执行器完整代际进程/账户/Profile/ACL清理通过。此对照自身不是产品验收，accepted:false。
3. 当前ControlledProcessRunner在spawn后对Start/Restart立即返回Started，后台退出receiver丢弃，stdout/stderr均null；commands::process_action为同步Tauri command，内部阻塞等待runner。直接返回Started并写start confirmed不能代表启动成功；长stop还占用UI线程。
4. 安装套件MSI默认哨兵与同用户后续custom清理夹具冲突已仅在test/修复：卸载保留hash先证明，随后只移除完整树相符的本次新建sentinel和空父目录；额外内容/变化拒绝；原矩阵和实际文件系统行为检查通过。不得把产品问题混作测试时序修掉。

## 建议产品修复边界（尚未实施apps改动）

- commands::process_action改为async，经现有阻塞任务机制调度；写入与runtime-install reservation由实际worker持有到终态。窗口关闭/IPC观察者超时不能释放尚在运行的动作，其他只读界面保持响应。
- 保留真实spawn与退出代际、原退出码和已有状态源；启动只代表已提交，真正running仍由既有HMAC+ready身份观察证明。后台启动退出非0必须回写有界固定错误类别/退出码并通知当前作用域消费者，端口占用不能再变成start confirmed。原始输出只在有界内存分类，日志不泄露完整argv/Key/凭据。
- Stop/Restart针对当前显式H与当前managed程序，在完整激活验证、受保护运行记录、真实进程归属及已认证官方管理会话后走官方有界停止/重启接口；官方仍拥有配置/客户端还原与停止语义，不自己kill代理、不按端口杀进程、不删运行记录假停，不跳过默认官方teardown。无可证明身份保持拒绝；10秒内未证明服务已退出仍失败/未知。完整CLI留作实际诊断/不支持场景，不用其超时声称成功。
- 启动30秒、停止10秒、状态读取5秒、真实双SID、固定冲突、不同默认端口、六秒重叠、B停A存活及精确账户/进程清理全部不变。不关闭客户端集成，不改服务只活着即可ready，不新增旧路径扫描或迁移。
- 不修改维护运行包默认行为或重新降低其所有权/安全检查。若发现必须改维护包或者官方接口无法在保留语义的预算内完成，先记录具体原因，不扩大本授权或放宽验收。

## 必须验证

- 实际快速非0/延迟失败/长期运行启动、固定冲突原码/分类与无误判、取消/重启/worker存活与lease、错误输出限额/脱敏。
- 同H真实身份/篡改/记录切换/外来PID/端口/无认证/超时/半途断开均拒绝；官方成功停止后的进程代际、record及socket真实退出，含默认客户端清理结果，不用tasklist成功推断。
- UI在pending stop期间仍可观察/关闭/重开，动作状态不丢失。停止失败时现有配置与运行身份保全；同源Mac和真实Windows定向回归，strict Clippy。
- 一轮完整27项sweep依旧accepted:false。全表后整批修复，再正式同源候选安装/两用户/五格保留/签名相邻升级回退，最终发布合同不减。

## 当前状态

仅提案和测试/装配已落盘；apps未实施以上产品变更。用户最新限定只改test/.github，之前授权路径前缀、初始化/查询、认证状态读取以及names.2自动端口/精确版本；本提案触及实际Start/Stop/Restart结果与后台编排，需额外明确采用。
