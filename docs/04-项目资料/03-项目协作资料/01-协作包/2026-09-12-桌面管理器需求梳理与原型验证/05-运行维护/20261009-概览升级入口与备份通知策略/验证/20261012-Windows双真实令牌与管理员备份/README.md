# Windows 双真实令牌与管理员备份

2026-10-12，Windows Server 2025 Datacenter build 26100，同一物理机、同一账户session2、同一隔离根、精确 `main@afff490` EXE。证据归 `.adg/evidence/OCX-0110-20261012-WIN-ADMIN-TOKEN/`。

## 普通 / 管理员真实令牌

原普通PID41328、监督器44628的应用令牌实际为elevated=false/type3。经已有监督器受控结束，独立确认原应用/监督器退出、候选数0后，以标准Windows RunAs启动同一单实例监督器，未自动化认证、修改安全设置或使用脚本安装权限。

管理员PID42372、监督器44052的应用令牌为elevated=true/type2，创建UTC09:38:42.790，源码日志匹配。监督器对持有的进程句柄读取令牌；另起独立管理员只读助手再次打开同一应用进程句柄核验EXE、创建时间和令牌，结果一致。第二助手只是诊断进程，不是第二应用实例。

普通权限读取管理员进程路径为null，既有身份门禁拒绝metrics观察；保留该失败，没有放松校验。提升的独立只读助手读取已选主HWND381618848，所有者PID42372、DPI192、visible=true、foreground=false、DWM attr1；原生DTO为inactive/solid，DPR2。Computer Use抓帧FrameArrived超时，重新选择后的AX仅窗口/标题栏，不据此操作或通过管理员完整UI/Mica矩阵。

原普通Get-VM权限不足已在前页保留；本轮管理员只读Get-VM实际成功返回0台VM，补充当前Hyper-V清单空的事实，未创建或改动VM。安全安装测试环境仍未取得。

## 管理员原生命令备份回环

真实创建备份`bk_20261012094808_1ef0160b`并固定；保存theme=light后，以该备份真实恢复system。原生DTO完整一致、原始偏好SHA前后均8498f7cf…；没有使用finally兜底保存来冒充恢复。pin跨恢复保持，新恢复保护protected=false，refreshRequired/protectionReconciliationPending均false。最后撤回本轮临时pin并重载生产store，原用户pin保留、manual策略/自动检查关闭。

这是实际管理员进程执行的原生命令验证，不冒充真实鼠标/标题栏、安装目录data或安装器权限验证。

## 返回普通实例

受控结束管理员会话，确认应用/监督器退出、候选数0后才启动同根同EXE普通PID38924、监督器44780（exec session58413）。新令牌false/type3、源码匹配、偏好SHA及日常0.1.9 EXE摘要不变。两次切换没有重叠应用，不计受控清理为正常退出验收。

23个原始文件清单和令牌/顺序/回环断言冻结并重验通过。管理员完整交互/前台Mica、其余实际DPI、安装更新卸载、全事件、同机配对性能、真实周期仍开放；原TASK/脚本二次确认、原预算/既有性能failed保持，IMP in_progress，不关闭或发布。
