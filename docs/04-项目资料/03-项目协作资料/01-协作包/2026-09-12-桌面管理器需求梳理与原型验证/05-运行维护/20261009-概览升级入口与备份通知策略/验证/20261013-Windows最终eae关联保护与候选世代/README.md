# Windows 最终eae关联保护与候选世代

2026-10-13 00:20:37起（UTC+8），在精确 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc` 的唯一普通应用PID37276上执行真实生产WebView原生命令；RDP断开，不作为鼠标/键盘或完整UI验收。原件归 `.adg/evidence/OCX-0110-20261013-WIN-EAE-RESTORE-PROTECTION/`。

## 清理前全候选校验与失败关联保护

原manual/N10/D30策略、原29条库存及system/stable/auto-check true保持。真实新建两份备份，只将这两个新ID的manifest.created_at设为2026-09-01；较新的正常夹具先列入清理候选，较旧夹具的载荷临时翻转一个字节。没有修改活动偏好、原库存或系统时钟。

| 实际操作 | 独立结果 |
|---|---|
| 原生预览 | 候选恰为较新正常记录、较旧损坏记录，顺序确定 |
| 实际清理这份预览 | 在损坏的第二候选因SHA-256 mismatch拒绝；第一正常候选也未删除，前后库存完全一致 |
| 实际恢复损坏记录 | size/SHA-256 mismatch拒绝，活动偏好原字节保持 |
| 拒绝后的保护与清理预览 | 新restore-protection为active、pinned=false；关联源protected=true/referenced-by-protection，新预览仅含正常夹具 |
| 尝试取消失败保护记录固定状态 | 真实拒绝，不能用unpin解除事务保护；库存不变 |
| 再执行保护前旧预览 | inventory changed拒绝，源与保护记录均保留 |

失败保护为 `bk_20261012162037_682ae822`，关联源为 `bk_20261012162037_85e92c53`。metadata、载荷摘要、关联ID和原生DTO相互核对；没有直接写保护状态或删除文件取得结果。临时损坏仅作用于本轮新备份载荷，随后以原字节修复；finally恢复两份原manifest日期及载荷，独立SHA核对通过。

## 同一恢复动作的候选世代

在修复载荷后，真实成功恢复另一份正常候选 `bk_20261012162037_7f2f82c5`，再真实成功恢复原损坏候选。两次均refreshRequired=false/protectionReconciliationPending=false，当前偏好与原DTO、原文件SHA相同，新的各自恢复保护committed/protected=false；首次失败事务的保护始终active，未被另一事务释放。

| Restore/local/execution阶段 | 指纹与候选UUID |
|---|---|
| 首次失败 | 原记录指纹2f1ce264…，UUID f1a9a33e… |
| 另一候选成功 | 另一指纹d3c97153…，UUID 7e89ecf4… |
| 返回原记录成功 | 指纹回到2f1ce264…，UUID a03c6895…；没有复用旧UUID |

指纹独立从实际备份ID重算SHA匹配。原生通知及原始notifications.json中的新失败身份仍绑定首次UUID，两个后续成功后均resolved=false/resolvedAt=null，全部原有通知解除维度和时间保持。原始event-scope的根object_id不变，manager/panel远端分区也未旋转。

这是“同分区候选改变会旋转，返回旧摘要不能恢复旧UUID”的实际世代隔离，不是同候选连续重试失败。本轮不应要求后来新世代成功解除旧世代异常；此前最终eae连续同候选成功解除的独立证据仍有效。全量动作/阶段/通道及在途安装隔离矩阵仍待验，未因本场景将全部旧异常清空。

## 来源、周期和证据边界

原所有备份文件逐字节保持；最终只新增本轮两份完整原备份及三份真实恢复保护。原用户pin及原事务保护DTO保持；当前失败保护仍有意义，按原门禁保留。偏好、根绑定、策略和update-schedule原字节始终相同，调度SHA6e51fe41…，真实stable/面板截止仍2026-10-13 22:55:39/40（UTC+8）。

应用37276、监督器24300、监测3920的实际句柄、创建时间和路径保持，只有一个桌面实例；精确候选EXE SHA f5e4a2ee…、日常EXE SHA43e06bc3…保持。没有手动更新检查、切根/通道、安装脚本、时钟/timer修改、周期或应用重启。生产store加载system且error=false，原生桥未替换。

首轮校验器错误地要求store.error为null，实际原件为布尔false；精确源码定义error:boolean并在成功load后置false。保留首错日志与原件，改为严格`is False`后校验，不接受任意假值，不重跑原生场景或降低产品断言。

53个原始文件的全集合、字节摘要及全候选校验、关联保护、三UUID/旧世代不误解除、原件/周期保持断言冻结并重验通过。只闭合上述局部命令场景；完整备份安全/UI、全事件通道/在途矩阵、物理DPI、安装更新/卸载、同机配对性能和真实周期仍未闭合。TASK182原件、安装脚本二次授权及隔离Profile环境缺口，原预算及既有failed保持；IMP in_progress，无正式RCP/证书，不降低门禁、不关闭或发布。
