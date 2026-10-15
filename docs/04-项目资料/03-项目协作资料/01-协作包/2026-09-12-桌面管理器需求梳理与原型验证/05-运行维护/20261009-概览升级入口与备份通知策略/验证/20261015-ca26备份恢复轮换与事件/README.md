# ca26 Windows 备份恢复、轮换与本地事件实测

2026-10-15，Asia/Shanghai。本段记录已完成的有界前台子集，完整 0.1.10 门禁仍未通过。原 TASK-182 的八项 required_checks、revision 2 / in_progress / pending 及原字节保持，不产生完成证书。

## 候选与执行范围

源码 `ca26e771b0808b8b1b58c7ffadf5d1bad46189ff`，精确候选 run `38663871103` 已 success；MSI 只读提取 EXE SHA256：

`f43e81a9d96f25a29a937a54d2c779ce2459c3ecb30a73b131c57729f70cb929`

沿用[候选准入证据](../20261015-ca26候选后台准备与回放差异/README.md)，本轮没有修改产品、重新打包或在宿主执行安装器。新的隔离根为代码 worktree 内 `test/runtime/foreground-backup-ca26-20261015/data`；偏好、备份、通知和 WebView profile 均在该测试边界。

用户真实信号“开始前台验证”于 09:43:50.723 记录，30 分钟窗口截止 10:13:50.723。输入仅使用 Sky；loopback 配置仅明确隔离 WebView profile，没有通过 CDP 执行业务或注入输入。自动面板、管理器自动检查、启动清理、随 Codex 启动均关闭，关窗保留关闭。

| 顺序 | 产品身份 | 实际令牌 | 正常退出 |
| --- | --- | --- | --- |
| 首实例 | PID 18460，创建 09:45:25.4876884，主 HWND 5637288 | elevated=false / elevationType=3 | 10:02:05.0375261，持有句柄观察 exitCode=0 |
| 同根重启 | PID 66488，创建 10:02:33.1716359，主 HWND 6165676 | elevated=false / elevationType=3 | 10:04:59.4693475，持有句柄观察 exitCode=0 |

首实例退出后 10:02:32.097 先证实本机 native=0，再顺序启动第二实例。两实例同 EXE、同源码、同隔离根，未并存或强杀。10:05:29.919 最后保护检查 native=0；前台提前结束，之后仅离线校验和文档回写。

首屏实际单系统标题栏、无应用菜单，DWM backdrop=2；这是本实例的局部状态。remoteSessionMetric=1 / nativeDpi=192，不当物理 200% 证据。按用户“不用管 dpi”的指示未改物理缩放，物理 DPI 门禁仍待验。

## 实际备份与异值恢复

真实 UI 生成手动备份 `bk_20261015014652_9e631596`，payload 1374 bytes / SHA `25a41960e9033165e15052d927816914cbe266c67f97a1830240eff8ee547eab`。通过真实 UI 固定，再将主题 dark 改为 light；light 偏好为 1375 bytes / SHA `07fcccd25b7e7dcc6ee6c752092e22659460929d770eef8c28f84bc4eb00bb0d`。

恢复上述手动备份后，active preferences 恢复 dark，逐字节等于原 dark payload。恢复前保护 `bk_20261015015101_2a4ddd2a` 保存真实 light 字节，management v2 / restore-protection / related 为原手动备份 / transaction_state=committed。原用户 pinned=true 独立保留，未把事务保护与用户固定混同。

同根重启后，通过 UI 将原手动备份取消固定再重新固定；两步均落盘核验，取消固定没有立即删除。最终策略为 manual，原备份仍 pinned=true。

## 保留与轮换边界

通过真实 UI 另生成 12 份手动备份，每次可见记录数递增 1；加原备份与恢复前保护共 14 份。全部新鲜时，预览显示没有可清理项，即超过 10 份也受 30 天条件保留。

之后仅在本隔离根建立明确的 manifest 年龄夹具，原件及前后摘要保留；只改变 `created_at`，不改 payload、ID、stored_at、目录或系统时钟：

| 场景 | 人工夹具与真实 UI 动作 | 落盘结果 |
| --- | --- | --- |
| 手动轮换 | 原固定备份合成 90 天；`bk_20261015015159_1cc59ec0` 合成 61 天，`bk_20261015015200_df084d8f` 合成 60 天；UI 预览后确认清理 | 预览 2 份，恰删除后两份；固定 90 天项保留，剩 12 份 |
| 保存自动策略 | 将 `bk_20261015015201_a254f99c` 合成 60 天，此时仍在普通最近 10 份内；UI 保存 automatic | 保存策略本身不删除，前后备份记录完全相同 |
| 自动轮换触发 | UI 成功生成新备份 `bk_20261015015618_1393e76a` | 仅删除上述掉出最近 10 份的 60 天项；新备份存在，固定项保留，仍 12 份 |

最后通过真实 UI 保存回 manual。合成年龄只证明保留规则及触发边界，**不证明经过真实 30 天，也不证明 stable 24h / beta 6h 周期或状态错误 30 分钟退避**。

## 损坏拒绝与精确事件解除

在隔离备份 `bk_20261015015618_1393e76a` 的 payload 中替换一个内部 JSON 空白 LF 为 space，1374 bytes 与 JSON 语义不变，SHA 变为 `cbf910baaeda1ded82aef83b20588030d5feb850c7b3d4fe5012a22ddddf9197`；manifest 未改。原始与损坏字节分别保留。

真实 UI 尝试恢复被拒绝，active preferences 与尝试前逐字节相同；页面显示“操作未完成”及完整性检查提示。失败前保护 `bk_20261015015856_334223b7` 为 v2 / restore-protection / related 为该目标 / active / pinned=false，保留失败保护。

失败通知的精确 identity 为：

```json
{
  "object": "backup",
  "object_id": "1cde8588-7b83-4161-8772-f6b52d8ad625",
  "action": "restore",
  "phase": "execution",
  "channel": "local",
  "candidate": "792edcd4-00f7-46df-9b74-ee2c3bed584b"
}
```

随后 UI 成功生成另一份备份 `bk_20261015015922_b3f9f2c5`，上述恢复失败仍 unresolved。恢复目标的原始 payload 后，再由 UI 恢复同一目标成功：同 notification_id、同 identity 的记录于 10:00:30.524001 resolved=true，read=false / deleted=false；无关 runtime-not-found 警告继续 unresolved。

本次成功的恢复前保护 `bk_20261015020030_5ac0e0d6` 为 committed；旧失败保护仍 active，未被新尝试自动解除。这符合本次观察到的失败保留边界，不声称旧失败事务已清理或完成受控修复。事件结果只覆盖 Local restore，不能替代管理器 stable / beta 通道隔离或全领域事件验收。

## 生产树表与重启

保存了 29 份 Sky 状态及对应截图，包括初始原生首屏、空态外层、默认 root/year 展开而 month 收起、month/action/backup 展开及 manifest/preferences 文件叶节点。真实空格收起、Enter 展开，以及刷新后展开状态保留均有前后状态；深色三列、箭头/文件夹/文字与叶节点截图可复核。

本轮没有做像素坐标量化，也没有完整验证刷新后焦点保留、全部主题/密度/缩放或原型复选框。生产 BackupFiles 没有业务复选框，不能将其占位当复选框门禁通过。浅色主题切换截图只证明异值恢复步骤，不扩大为浅色完整树表验收。

同根重启前后，15 份备份的全部文件大小和 SHA 一致，dark 偏好字节一致，启动没有清理备份。恢复失败通知的已解除、未读、未删状态也持久保留；最终仍 15 份（12 manual-preferences、3 restore-protection），旧失败保护保持 active。

## 方法错误与离线复核

证据目录：`.adg/evidence/OCX-0110-20261015-WIN-BACKUP-PREFLIGHT/`。原 17 项预检清单及旧方法不改，本段用 `foreground-manifest.json` / `foreground-verification.json` 独立冻结。

三项方法问题保留原件，不当产品失败：

1. 启动前原授权解析将 PowerShell 的 UTC DateTime 强转无时区字符串，误判过期，实际未启动、native=0、runtime 不存在。新增 live 方法保留类型及 UTC instant，真实授权起止没有改动。后补 6 正 / 4 负离线类型回归，覆盖真实 JSON DateTime、Z/偏移字符串、DateTimeOffset、UTC/local DateTime、过期、无时区及未来时间；没有延长窗口或重启应用。
2. 损坏夹具首轮错误要求 payload 末尾 LF，而原字节没有末尾 LF；写 payload 前即失败，只建空证据目录。原方法与说明保留，改为内部 JSON 空白后才执行上述有界夹具。
3. 离线 verifier 首轮要求最早原生首屏已有 document_text，但该原件只有 AX tree；保留方法/错误说明，仅对这一个已知初始状态豁免 document_text，所有内容状态继续要求。失败页面匹配实际通用提示，具体损坏拒绝由磁盘状态和事件共同证明。

`foreground-manifest.json` 冻结 582 项原件/快照/方法，SHA256 `df76b8c63e76d5a70f9e893d35d88805e4ef9c5495e7535f75b0e9b9b49189d8`。独立复核 18 份快照、193 次有效 payload 摘要检查及两个预期损坏快照；193 是跨快照的文件复核次数，不是 193 次恢复场景。新旧 verifier 均通过：

```powershell
python -X utf8 .adg/evidence/OCX-0110-20261015-WIN-BACKUP-PREFLIGHT/verify-preparation.py
python -X utf8 .adg/evidence/OCX-0110-20261015-WIN-BACKUP-PREFLIGHT/verify-foreground-backup.py
```

仅运行以上离线复核方法即可，不要重跑启动、年龄或损坏夹具脚本。

## 保护与剩余门禁

10:05:29 日常 EXE SHA `43e06bc3bd6e7f633bf3928e7f58b2ad9dbd9b2a8573ea197251c4bbd4401e67`、旧隔离 preferences/data-root/policy/schedule 四摘要均保持；TASK 原 3878 bytes / SHA `bc7965b44343391c0de831b2e49c630bcd93a4569e7396652b03cb6930f8ff93` 保持。用户原未提交删除不纳入本次提交。

本轮无新安装/升级/卸载、第三方生命周期脚本、更新下载或自动周期。管理员活动材质/完整窗口、托盘退出/Snap/物理 DPI、默认安装 data 与自定义根和卸载保留、真实更新详情/进度/重启、完整备份失败矩阵与树表、全领域事件/通道、状态错误真实 30 分钟退避、stable24h/beta6h 与 sleep/restart、官方 0.1.9 同机配对性能仍开放。官方 0.1.9 Known Folder/profile 隔离和安装安全环境仍缺，Mac 原生尚未正式交接。

原 bd2/e6 查询失败与失败时间窗 ETW 缺口、Mac CPU +1.817629pp / RSS +117.008965MiB 失败及原预算保持。性能仍要求每版冷/热各至少 20 样本、interactive startup p95 增量同时不超过 100ms 与 5%、每版 600s CPU 增量不超过 0.5pp / RSS 不超过 10MiB、main 不超过 50ms，以及 freshcache 100 routes / 10 restarts 零自动请求/query。真实周期不额外要求同一进程连续 24 小时。

本段不更新任何 required_check 为 passed，不关闭 TASK/IMP，不合并、签名、晋升或发布。前台结束后保持无测试实例，后续原生操作等待新的用户前台窗口。
