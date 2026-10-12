# Windows 合并制品主题原生回验

2026-10-12，Asia/Shanghai。关联 IMP-20 / TASK-182，接续[主题与cwd集成](../20261012-Windows主题与cwd集成/README.md)。证据归 `.adg/evidence/OCX-0110-20261012-WIN-MERGED-NATIVE/`。本次以合并源码、合并制品、真实生产WebView与文件元数据验证主题修复，不沿用父提交制品。

## 精确源码与制品

合并SHA `afff490a430a81f8c233d44928aa3681a23e91e0` 双亲为 e1a1aaa（受控npm cwd）及 cf71ea4（持久主题/历史导入修复）。八个相关文件的合并Git blob逐字等于各自已验证父提交，单独记为源码核验，不能替代合并CI。

Windows候选运行38213528684 completed/success；实际package-lock回归、NSIS/MSI打包、安装/最终首屏/卸载和资产上传均success。下载在操作步骤通过、cache post仍收尾时开始，初始与最终任务状态分开保留，未将收尾中的整体状态提前记为success。

- 候选ZIP11732300240 SHA-256 `59ea235c2a1b915f1a8b3603c8455b466243e8037e8780c82739799ad2e6e223`。
- 证据ZIP11732080359 SHA-256 `f0c49ea466a5c0eb08f71e0544dce42db6972d3e76159abf64e0c96051d27064`。
- MSI SHA-256 `1707ae1187c98b2cd9eda186f3824301e477fc68867938cfbb9a9b4dbae5dcbd`。
- 只读提取主EXE SHA-256 `bf8a39e977ce5c554923f5f826a02b8f71cc37ed00ac08c258e436f104d9a7a1` / 0.1.10，与同SHA CI安装后主EXE一致。

ZIP均与GitHub digest一致。本机仍未执行管理器MSI/NSIS安装器，EXE/CAB/ZIP保留于忽略的test/software，不提交治理分支。

main CI38213445710在14:57:36观察时，Windows最终EXE与smoke等前置job已结束，进入macOS build job的工具链安装，整条CI仍in_progress；状态以该次原JSON为准，不能把候选成功记作整条CI成功。此前未认证API读取遭限流后改用已安装Credential Manager认证重读；没有重启/重派任务，也没有保存token。

## 原容器唯一实例与原生读取

核对原e1 PID32564实际路径/创建时间/保留句柄后，专用supervisor完成stop-requested，原会话退出码0。该清理不计正常用户退出门禁，不操作日常进程。

原容器 `test/runtime/windows0110/` 与活动根data不变，启动唯一合并候选PID29936。实际应用令牌elevated=false/elevationType=3，当前启动日志匹配afff490，EXE摘要与核验制品一致。supervisor/应用身份及一次只运行一个候选分别保存。

主题局部结果：

| 检查 | 实测结果 |
|---|---|
| 磁盘当前结构 | schema2，appearance.theme=system |
| 原生DTO | theme=system，themeNeedsImport=false |
| 生产store重新load | theme保持system，error=false，缓存投影system |
| 跨制品启动及reload写入 | 文件SHA-256始终 `8498f7cf8984c74a555202027800027fa7fd1366e0e5ce51c455aaf3ba1fdb7a`；mtime始终 `2026-10-12T06:26:32.5458289Z` |
| 原用户固定备份 | `bk_20261012044452_a921f503` pin仍true |

因此此前“当前schema2显式主题仍请求历史导入”的真实DTO错误，在此合并制品中已纠正，并且启动/reload没有产生错误的额外偏好保存。原历史无theme导入、失败重试、全部浅/深/系统主题及实际物理DPI交互矩阵仍依赖各自证据，不能从这个system值局部检查扩展通过。

操作来自实际生产Tauri命令和production store，未伪造返回值；scope明确foreground=false、DPR2、RDP断开。没有把后台WebView读数当持续前台性能或完整原生GUI通过。

## 保留门禁与后续依赖

脚本二次确认仍缺失，未新启动任何安装事务、allowScripts未开启，真实运行来源仍无成功面板安装。原TASK-182仍未同步；RDP session2/15119断开。

管理员真实令牌、其余物理DPI/原生标题栏与Snap、实际管理器安装/升级/卸载保留、自定义根成功/重启/迁移、真实更新详情/进度/回读、完整备份保护与链接逃逸、全事件与在途通道、状态失败真实30min退避、同机官方0.1.9配对CPU/RSS/冷热p95/主线程及真实24h/6h观察仍未全部完成。既有性能failed、原预算和八项门禁保留；本轮无正式RCP/完成证书、稳定核心晋升、真实数据迁移、日常替换或公开发布。日常0.1.9 EXE摘要仍一致。
