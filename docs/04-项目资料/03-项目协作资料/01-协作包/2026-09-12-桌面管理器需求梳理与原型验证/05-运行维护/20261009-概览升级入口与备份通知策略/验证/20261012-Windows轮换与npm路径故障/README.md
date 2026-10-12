# Windows 轮换、最新树表复验与 npm 路径故障

2026-10-12，Asia/Shanghai。续接 [Windows 单实例局部门禁](../20261012-Windows单实例局部门禁/README.md)，关联 IMP-20 / TASK-182。原 TASK 仍未同步，本页不是正式 RCP 或版本通过证明。证据归 `.adg/evidence/OCX-0110-20261012-WIN-RETENTION/`；所有写入夹具均在唯一 `test/runtime/windows0110/` 容器中。

## 精确 4f7f334 与单实例替换

main `4f7f33435ec1a7032e468363f191ddce50330fd8` 的 CI `38205914227` 六项 job completed / success；Windows候选 `38206176747` success。候选 ZIP `11727387957` 摘要 `f92eec7bd2315f351ed973f39a3dd73b39432289cbb2628a762f14209034fe4b`，证据 ZIP `11726784455` 摘要 `2a746079826b641ab9bcdd53d28fe2098d8de4a6c07f097ad15fbf52f7180066`，下载后均与 GitHub digest 一致。

MSI摘要 `5df2185496378ae67d7e16936a8c5d2cbfafb17a60e90cd8749f4ff91bfc5385`，只读提取主EXE为0.1.10，摘要 `5e17773983bfcbdd741121210170a95922a436b4b6b9b783b12eca2062a5f6dc`，与CI安装后的EXE一致。仍未在本机运行MSI/NSIS，日常安装的共享UpgradeCode隔离限制不解除。

确认旧0a实例PID41528身份后，仅向其专用supervisor发stop.request，确认旧应用退出、supervisor完成；这是受控测试清理，不算正常应用退出通过。随后复用原容器/数据根启动4f候选PID20100，实际普通令牌false/type3、当前启动日志匹配源码，仍仅一个测试实例。原固定项跨启动仍保留。

## 真实轮换与损坏恢复

| 场景 | 结果 | 实证边界 |
|---|---|---|
| 近期超过N10 | 真实命令生成到14份手动备份，预览候选0 | 证明D30保留条件，不是最多十份 |
| 手动清理历史日期夹具 | 14份manual中13份非固定；锁内改为显式2026-09-01日期后，预览3候选；新增固定后旧预览被拒绝且零删除，取消临时固定、刷新预览后仅删除3份；最终10份普通+原固定项 | 真实清理命令，日期为合成夹具，不代表实际经过30天 |
| 自动轮换 | 保存automatic策略不改变任何记录；对第二组历史日期夹具，真实新备份成功后自动删4份，保留10份普通+原固定项，新成功记录保留、cleanupError=null | 只在成功后执行；本轮未覆盖备份失败时的全部自动轮换分支 |
| 损坏载荷恢复 | pinned偏好载荷临时翻转一字节，真实恢复被拒绝，活动偏好与pin保持；原载荷逐字恢复 | 真实digest拒绝；不是断电/所有损坏组合 |

人工日期写入和还原均持有与生产同名 `.backup-w2.lock` 的Windows独占锁。两轮仍存活的manifest原字节恢复；删除的文件由实际清理命令处理，不通过夹具脚本删除。自动测试最终保存回manual策略。损坏恢复失败产生的未决恢复前保护继续保留，不为减少记录而解除。

## 最新生产 WebView 树表

4f实际生产WebView，DPR2 / 原生DPI192；根、年份、月份三行的名称、箭头、图标、用途与操作按钮首行中心偏差均为0px，操作按钮高度34px。`4f-native-tree-alignment-visible.json` 与实际内容截图 `4f-native-backup-tree-visible.png` 已核对。截图由CDP捕获该真实WebView正文，不含原生标题栏；记录中foreground=false。

因此这轮闭合了所记录三行的原生WebView几何修复，不能据此通过前台交互、复选框全状态、键盘、全部主题/字号/密度/物理DPI矩阵。Windows `15119` RDP session2仍Disconnected，未重复用旧截图坐标尝试输入。此前“latest source只是浏览器通过”由本局部WebView证据补充，整项门禁仍开放。

## 自定义根准入与真实安装失败

通过原生命令初始化容器内 sibling `custom-root/`，只给测试目标seed有效偏好；source其他数据未复制，reference-only不宣称迁移。真实 `switch_data_root(reference_only)` 返回blocked/runtime_unverified，未保存绑定、未绕过门禁。当前来源未解析且状态刷新失败，先取得真实面板状态后再重试。

为验证真实官方更新/进度及状态来源，核对npm registry精确官方 `@bitkyc08/opencodex@2.82.0` 的tarball/integrity后，通过唯一4f GUI启动其生产 `install_runtime`，source=registry、allowScripts=false、prefix默认归测试数据根。没有全局npm安装或日常包替换。

操作05:28:55.798Z开始，05:29:00.518Z失败，原生进度监听收到2006条事件；下载包身份核验已完成，npm安装报 `Maximum call stack size exceeded`。来源仍unresolved，失败历史保留，监听已移除。完整原事件与npm日志保留；该seed安装使用既有runtime-install的阶段/日志派生percent，不能当作可信总字节百分比或官方update通道通过。真实官方升级、详情和最终回读仍未执行。

npm日志明确显示参数 `--prefix \\?\C:\...\opencodex.new-...`；同一目录、同一npm10.9.0 Arborist realpath函数无网络复现：带`\\?\`失败，普通绝对路径成功。此局部复现建立路径原因，但实际安装修复仍须重跑。

修复 `ee65f1158c4f0bb7c283ff09e24120a7884472d0` 已提交推送 `feature/0.1.10-windows-npm-path`，尚未并入main：

- 内部canonical存储、安全验证、锁及激活路径保持；只对npm的prefix、pack destination、cache、临时userconfig参数使用安全简化。
- 使用锁中已有dunce1.0.5，新增直接固定依赖；不做字符串强行删前缀，设备名/特殊尾点无法安全转换时保留原路径。
- Windows真实Node回归验证npm收到普通安全路径、目标与cache仍canonical到原目录，覆盖安装和pack，保留设备/尾点路径断言。
- 本地rustfmt通过；CI `38208548177` 和候选 `38208582603` 本时点仍运行，编译/全回归按各自精确SHA取证，不冒称修复已验收。

## 受阻项与保留状态

创建/清理Windows junction夹具的命令被工具自动审批拒绝，仅返回“blocked by policy”；未执行，也未换工具绕过。相关脚本只是准备材料，`junction-approval-rejection.json`明确gateResult=not_run，链接逃逸门禁保留。

原TASK、普通/管理员完整矩阵、其它物理DPI/标题栏/Snap、实际安装/升级/卸载保留、自定义根成功重启、更新详情/进度/回读、全事件及在途通道、30min真实错误退避、同机官方0.1.9冷/热配对与10min CPU/RSS、真实stable24h/beta6h均未完成。日期夹具不替代真实周期，后台资源不替代持续前台测量。

既有性能失败和预算不变；IMP继续in_progress，required checks未整体闭合。日常EXE摘要仍一致，真实数据未迁移/恢复/替换；无正式RCP、完成证书、核心晋升、tag或公开发布。
