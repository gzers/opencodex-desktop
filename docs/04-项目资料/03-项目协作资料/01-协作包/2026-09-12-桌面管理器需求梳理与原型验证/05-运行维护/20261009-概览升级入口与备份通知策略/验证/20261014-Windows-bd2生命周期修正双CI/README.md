# Windows bd2 采集器生命周期修正双 CI

2026-10-14（UTC+8），自动目标接续只核验后台可做的同SHA CI、实际检出和原日志；没有将续行消息当作管理员录制、前台或Mac原生交接批准。

collector `abc3a9140fc6f1b5d2aa6e739299af260f0f5d94` 的 push CI38561752521、PR CI38561759502均completed/success，五必需job成功/main-only build skipped。每轮Mac/Windows各35组，分别900/627passed、6/7ignored；Windows最终EXE首屏冒烟成功，忽略项不计通过。

push实际checkoutabc3a91，PR实际16c781ea3e4abee9e7a7bf89efe38f3a24b187a9；两个tree均0aa5440ba89a4c31c6239b2fbe372e81a269f1b3。实际PR合成不当独立head编译制品，分别保存身份；源树一致本身不证明所有二进制相同。

完整两ZIP大小/SHA256/CRC、安全条目、四后端checkout/test原件封存到 `.adg/evidence/OCX-0110-20261014-WIN-BD2-LIFECYCLE-CI/`。PR#5正文只补实际新SHA终态及局限，读回仍open/draft/未合并/无auto_merge，head/base/title不变。初次读取TLS失败发生在写入前；后续只读重试后发一次body补充，不重复不确定写入、重派CI或创建候选。

## 这些证据没有覆盖什么

CI运行原Rust/UI/首屏回归；没有调用新增C#实时collector，也没有执行本机53纯scope/生命周期/native-stop检查。因此新CI绿不证明实际内核启用、事件家族过滤、调度/FileIO覆盖、事件loss/排空、真实ControlTrace stop或失败场景复现。53纯检查仍只归[前轮生命周期修正](../20261014-Windows-bd2采集器生命周期修正/README.md)，不挪成CI范围。

失败PR38537166560和MacCPU/RSSfailed继续开放。管理员单次录制入口仍固定abc新SHA，位于 `.adg/evidence/OCX-0110-20261014-WIN-BD2-COLLECTOR-LIFECYCLE/record-request.ps1`；明确批准和真实管理员令牌尚未收到，预定录制输出不存在，不能再以多份绿日志代替真正失败取证。

## 现有执行依赖

1. bd2失败时间窗原跟踪：批准限定内存筛选和仅持久化自有进程范围的管理员一次录制。
2. 最终源原生/更新备份树表事件：人的新前台开始窗口、稳定桌面及可验最终候选。
3. 安装升级卸载保留/官方019配对：实际安全安装注册与KnownFolder/Profile隔离，另目录不能消除MSI冲突。
4. 面板真实生命周期：FZ50脚本明确许可仍缺。
5. 原性能/30min错误退避/真实周期矩阵：最终源、原控制条件及直接原请求/进程观测；不缩预算或把旧schedule当完整通过。
6. Mac原生：Windows实例0后的明确交接，本轮未发跨线程消息或交接。

本机desktop/官方Node0、auto-check=false，日常EXE/隔离preferences/data-root/backup-policy/schedule原摘要仍匹配。原门禁/TASKrevision2/in_progress/pending/空验收、预算/性能failed及日常真实数据约束保持，未合并、签名、发布、晋升或关闭。当前依赖清单不是缩小目标或新增TASK，不把“准备可执行”写成“原目标完成”。
