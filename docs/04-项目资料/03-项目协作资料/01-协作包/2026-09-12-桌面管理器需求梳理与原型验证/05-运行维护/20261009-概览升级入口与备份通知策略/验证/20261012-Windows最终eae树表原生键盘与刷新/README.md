# Windows 最终eae树表原生键盘与刷新

2026-10-12，在精确 `main@eae85322ebca3b755c034db97cbebf2ea5bf71fc` 的唯一普通应用PID37276、主HWND1051698上，实际使用Computer Use鼠标、滚轮及键盘操作BackupFiles。原件归 `.adg/evidence/OCX-0110-20261012-WIN-EAE-TREE-NATIVE/`；生产DOM只读观察，不通过脚本路由、滚动、聚焦、按键或调用刷新取得UI结果。

## 原生导航与异步刷新

实际进入设置→数据与备份，滚动后展开月份10，随后依次操作：

| 原生输入 | 独立焦点/行数回读 |
|---|---|
| Right | manual-preferences折叠项，5行 |
| Enter | 展开manual-preferences，22行 |
| Right | 原用户固定备份bk_20261012044452_a921f503，22行 |
| Space | 展开该备份，24行 |
| Right / Down | backup-manifest.json / preferences.json，24行 |
| Left / Home | 父备份 / backups根，24行 |

实际点击树外“刷新”，只读观察器记录真实click及aria-busy true→false；刷新结束后五个展开层级保持，焦点留在刷新按钮。观察器没有代替鼠标调用刷新。逐步生产document.activeElement回读支持上述焦点，AX常仅报告pane，未把AX容器推定为树项焦点。

## 当前24行对齐与真实窗口

实际24个生产行的用途首行和行内操作相对首行最大偏差为0 CSSpx，行内按钮最小命中高度34 CSSpx；本场景没有checkbox控件。只读Win32独立核验主窗口归属PID37276、实际DPI192（200%）、foreground=true、DWM backdrop attr2。应用真实令牌为普通false/type3。

这只覆盖当前BackupFiles数据、主题与200%窗口，不抵checkbox/mixed/disabled、所有密度/主题、管理员及100/125/150%物理DPI矩阵，也不把CSS像素量测当作其他物理DPI证据。

## 捕获限制及来源保持

设置和备份导航后的首帧/AX出现延迟，均重新观察后才继续，没有重复点击。刷新后的最后一次新捕获失败：`IGraphicsCaptureItemInterop.CreateForMonitor ... (0x80070057)`；重新枚举仍为原HWND，随后quser确认session2已Disconnected，当即停止进一步UI输入。

保留的native-tree-last-successful-frame.jpg及AX是此前成功刷新操作的原帧，可能滞后最终焦点，明确不是断开后的当前截图。失败消息、原生输入顺序、捕获边界和断开后只读24行/五展开层级回读全部保留。预定的共享更新详情UI本轮未执行。

偏好、根绑定与调度文件前后原字节完全相同，SHA分别674ad619…、8ae75370…、6e51fe41…。应用37276/监督器24300、周期监测3920的实际生命周期句柄仍活，仅一个应用；stable/auto-check true及2026-10-13 22:55:39/40（UTC+8）真实截止保持。未手动检查更新、安装、切通道/根、修改时钟/timer或重启周期；日常EXE摘要43e06bc3…保持。

27个原始文件的全集合、字节摘要及本场景键盘/刷新/对齐断言冻结并重验通过；archive-verification仅认证归档完整性及该局部场景。更新详情/进度、完整树表和原生矩阵、安装/卸载、全事件隔离、同机配对性能与真实周期仍待验。原TASK缺失、脚本二次确认、隔离Profile环境缺口、原预算及既有性能failed保持；IMP in_progress，不降低门禁、不关闭或发布。
