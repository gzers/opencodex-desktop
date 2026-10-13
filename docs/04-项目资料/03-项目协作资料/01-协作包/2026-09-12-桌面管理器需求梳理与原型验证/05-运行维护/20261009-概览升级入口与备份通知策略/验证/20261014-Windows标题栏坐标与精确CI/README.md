# Windows 标题栏坐标与精确 CI

2026-10-14（UTC+8），接续 Windows 可独立完成的后台验证。诊断工具固定 `feature/0.1.10-caption-diagnostics@5a36cda04018fb367c2f77e3a2a1bdc22d991e03`，实际应用仍为 `feature/0.1.10-update-center-focus@be7395acfce0ec1e36b83300b576db408f8c35bb`。没有输入、安装、切换制品或启动第二原生实例。

证据目录：`.adg/evidence/OCX-0110-20261014-WIN-CAPTION-POINT/`。`verify-evidence.py` 无参数重验冻结清单；不对既有目录重新 `--build`。

## CI 与制品

精确 5a CI `38431616359` 为 completed/success：release-tools、windows-smoke-evidence、backend、frontend、backend-windows 五项成功，main-only build skipped。两份 CI 测试证据 ZIP 已下载至忽略的 runtime，摘要及 ZIP CRC 均独立核对：

| 制品 | 字节数 | SHA256 |
| --- | ---: | --- |
| windows-process-token-evidence | 825 | 89ffa78f88fbd3532b24f1da8d41932a185eba9e8080c894569e855b63ed42e7 |
| windows-startup-evidence | 9218509 | 56dd8f16ff4409e68d448f44565375dca6d8034353fefc2b2fd1c5f4bf889b72 |

第二包初次下载 TLS EOF，已保留第一次完成的包，只补缺失包并复核；未关闭 TLS 验证或向存储域转发 GitHub 凭据。七份选定 CI JSON 按原字节归档，无二进制执行。

CI startup 为 `mode=any`、runner 应用令牌 elevated=true/type1、EXE SHA3eefaf62…；它不是本机普通/管理员矩阵，也不是可安装候选。诊断分支基于 main/eae，只新增工具和 CI；不含 be 的焦点修复，不能据此替换本机候选或用本机 be 证明 5a 产品运行验收。

## Windows 实际只读观察

00:50:34，同一 PID10904 / HWND6685560 / creation FILETIME134363770515757733 / EXE SHA3a26098e…，普通真实令牌仍 false/type3。正常窗口 DPI192、foreground=false、DWM backdrop=1，物理窗口范围 `[2007,355,4393,1946]`。完整工具在本机运行成功；16 项 fake-backend 便携契约均通过。

按钮左四分位/中心/右四分位的父窗口 WM_NCHITTEST：

| 按钮 | 父窗口命中 |
| --- | --- |
| 最小化 | 2 / 2 / 8 |
| 最大化或还原 | 8 / 9 / 9 |
| 关闭 | 20 / 20 / 20 |

九个坐标的 WindowFromPoint/GA_ROOT 均不属于目标窗口或 PID；目标当前处于其他窗口遮挡之下，无关窗口身份未落盘。父窗口命中正确不能证明鼠标点击送达。此观察不能解释此前最大化还原失败，不能据此修产品、放宽断言或计任何真实点击通过。

前后日常 EXE、候选 EXE、隔离 preferences/data-root/backup-policy/schedule 摘要一致；本机进程普查仅一个隔离应用。auto-check=false 保持，没有新的自动24h测试启动。

## 下一前台与依赖

人的新开始信号后，先确认连接/解锁和最新真实 HWND，再逐动作采完整截图与状态。最大化后重跑只读工具，核实坐标归属后单次还原点击；立即读最大化标记及窗口边界。失败先保留点击坐标和完整前后帧，再使用系统菜单恢复；后续最小化、Snap、托盘、退出分别取证。

普通/管理员与物理100/125/150%DPI需要前台窗口；安装卸载及官方0.1.9配对仍需真实安全安装隔离/KnownFolder Profile，不能与日常版共用MSI注册。FZ50脚本许可仍缺。最终源30分钟状态错误退避、真实beta/周期原始计数及完整性能预算保留；Mac新修复须另核对应SHA候选。八项原门禁、历史性能failed与TASK/IMP状态均未提升，不关闭或发布。
