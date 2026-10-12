# Windows 最终eae备份命令回环

2026-10-12 23:07（UTC+8）。最终 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc` 精确提取EXE，唯一普通PID37276，同一隔离根。证据归 `.adg/evidence/OCX-0110-20261012-WIN-EAE-BACKUP-ROUNDTRIP/`。

RDP session2重查为Disconnected；本轮没有可靠鼠标/键盘输入，**不是树表或完整恢复UI验收**。在实际生产WebView中调用原生命令，随后原生DTO、生产store和独立文件摘要核对，未替代任何命令结果。

## 实际不同值恢复

当前周期偏好为system主题、stable/auto-check true，备份策略manual。真实创建 `bk_20261012150743_7dc2689a`，没有自动删除备份；设置pin=true，真实保存theme=light，再从该snapshot恢复system。恢复DTO与原DTO完全一致，没有finally兜底保存冒充恢复成功。

恢复前后的该备份pin均true，新恢复事务备份 `bk_20261012150743_83813305` 的protected=false、transaction_state=committed，related_backup_id指向本次原快照。refreshRequired/protectionReconciliationPending均false。最后撤回本轮临时pin并加载生产store，原用户pin仍保留、store无错误、theme system、策略manual；未清除旧异常或伪造通知解除。

原偏好和snapshot载荷SHA均为 `674ad619b46fc27a273eb37da775660e3b44263fe3485e4ade98287f5fbda8fa`，恢复后活动文件相同。保护载荷是实际light值，SHA `05ab9eb4112e30716b033b9d6e53301b4ce3de24512b455602f8e2fbf0aab38f`，与原快照不同；独立读回两payload的主题和自动检查开关。

两份列表DTO的integrityVerified=false是未主动全清单验签的列表投影，不能据此宣称整个备份库存验签成功；本次真实restore的完整性准入、payload/manifest摘要与字节已单独核验。现存通知的resolved维度及时间前后一致，不以新候选成功解除旧世代异常。

## 周期保持

不同值保存、恢复及清理后三个实际读回阶段均为stable/auto-check true。完整周期文件前后SHA均 `6e51fe41559c287c6036ef495456f1d1d5200710f5f925b3e9e7084621c3df20`，保留新eae截止2026-10-13 22:55:39/40（UTC+8）。未调用手动更新查询、修改deadline/时钟或重启产品。

独立持有实际应用/监测句柄再核验PID37276/3920存活，仅一个桌面应用。日常EXE摘要43e06bc3…保持，真实数据未迁移，没有包生命周期脚本或安装器执行。不据文件未变推断原始网络次数，也不认为该回环已证明24h终态或性能。

11个原始文件全集合/哈希及不同主题、pin/保护、原字节与真实周期保持断言冻结并重验通过。只通过本次最终源原生命令场景；全恢复UI/树表/轮换安全失败矩阵、管理员完整UI/其余实际DPI、安装升级卸载、真实更新、全事件、配对性能与真实周期继续开放。原TASK/脚本确认/测试环境缺口、原预算与既有性能failed保持，IMP in_progress，不降低门禁、不关闭或发布。
