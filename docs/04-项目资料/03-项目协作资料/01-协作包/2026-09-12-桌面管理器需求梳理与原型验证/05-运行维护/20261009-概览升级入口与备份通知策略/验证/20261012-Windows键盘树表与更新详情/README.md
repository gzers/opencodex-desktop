# Windows 真实键盘树表与缓存更新详情

2026-10-12，Asia/Shanghai。关联IMP-20/TASK-182。原根唯一afff490候选PID29936，普通真实令牌、DPI192/DPR2。本轮没有安装或运行第二应用实例，未打开安装脚本。原任务原件仍缺失，不创建正式RCP或关闭门禁。

## 树表实际输入

证据归 `.adg/evidence/OCX-0110-20261012-WIN-TREE-KEYBOARD/`。使用Windows Computer Use对生产BackupFiles输入，随后用只读DOM核对实际document.activeElement及aria-expanded；没有程序调用focus、模拟键盘事件或替代业务结果。

| 真实输入 | 生产状态 |
|---|---|
| 点击月份展开按钮 | 焦点`收起 10`，露出manual-preferences/restore-protection |
| Right | 焦点移至`展开 manual-preferences` |
| Enter | 分组展开，焦点保留`收起 manual-preferences`，可见行5→16 |
| Right | 焦点移至原固定备份`展开 bk_20261012044452_a921f503` |
| Space | 备份目录展开，可见manifest与preferences.json，行数18 |
| Right/Down | 分别进入backup-manifest.json和preferences.json |
| Left | 从preferences.json回到父备份目录 |
| Home | 回到`收起 backups` |
| 点击外部刷新 | 实际aria-busy true→false；根/年/月/分组/备份五级展开保持，焦点留在刷新按钮 |

刷新观察器只监听真实用户click及真实aria-busy变化，没有自行调用refresh；记录成功完成后disconnect。上述只覆盖这个现有生产无复选框备份树，不能扩展成全部可选checkbox/mixed/disabled、全部DPI/画质/管理员或失败重试矩阵。

保留先前无效尝试：计时观测产生路由唤醒时切走页面，`after-native-right.json`报树未挂载，伴随Node工具断言；不算产品失败或键盘通过。后续`stable-*`序列在唤醒循环结束后重新建立真实焦点，逐项验证。一次AX目标已在屏幕外，工具拒绝点击，重新观察/滚动后才操作；不使用过期坐标。截图有异步更新延迟，最终判定使用动作后稳定DOM与后续截图，不以旧帧单独证明。

## 共用更新详情

证据归 `.adg/evidence/OCX-0110-20261012-WIN-UPDATE-DETAILS/`。从设置“版本升级”用实际按钮打开管理器详情，回到全部更新，再查看面板详情；保存AX和真实原生截图。

管理器显示current0.1.10、无可用候选、已是最新，缺说明如实提示；默认备份勾选，安装按钮disabled。面板保留已取得远端2.83.0，本地未知/查询失败明确呈现，备份并更新按钮disabled；不把缓存远端版本冒充本地安装，说明未提供如实展示，官方网页出口可见（未点击外发）。

本轮只是缓存查看，不主动发check/download/install，不执行管理器或面板升级。Escape关闭可见更新面板并返回入口焦点的原生截图保留；原始事后`modalCount`只是全部role=dialog节点计数，不据此推断隐藏/可见弹窗数量。真实长说明内滚动、跨路由在途进度、下载验签与重启回读继续待验。

## 实际30min观测仍在运行

同期计时观测使用独立明确验证已安装的真实timer recorder，保持原Tauri桥/实际performance.now/计时延迟；旧内部floor未重置。16:00:52观察到错误分支重新安排完整30min，恢复原调度文件；200个route/online唤醒只保留一个有效timer，尚未提前执行。真实最小化使activeTimerCount=0，恢复后=1且最早名义截止仍为anchor+1800000ms。

以上是中间样本，尚未将30min门禁记为通过；最终依据monitor实际终态、fire时间及cleanup另行回写。树表中断与后续稳定序列明确分开，未把仪器干扰解释为产品缺陷。

原预算、既有性能failed、完整门禁继续保持；脚本二次确认和原TASK仍待用户补齐，未替换日常版/真实数据、不关闭或发布。
