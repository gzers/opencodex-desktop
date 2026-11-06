# L4 R2 MSVC 原生凭据失败回交

2026-11-07，Asia/Shanghai。补充前批 r2-33d2c65b-20261107-progress 的CI进行中快照；保留原快照，不回写其历史状态。

**已取得的真实结果**

- Draft PR #71，head=400c4e7f4a765b18e4406ba889d13fe5021ccb9b，merge ref=74583c49f1a137d05b28c78d0bbf01a272ab45d9。CI run=42249148982；backend-windows job=127999232404 已 completed/failure。
- Windows runner Rust 1.99.1，host=x86_64-pc-windows-msvc。日志记录成功编译 provider_auth_native 测试目标（6m06s），无需GNU EXE副本或manifest适配；这是本次实际MSVC编译证据，但不是“受影响库测试全部通过”或正式Desktop/NSIS/MSI制品。
- 原生测试 native_protection_reference_resolution_rebind_and_missing_key_refusal：0 passed / 1 failed / 0 ignored。失败发生在tests/provider_auth_native.rs:141的provider_import::stage，错误如下。

~~~text
ModelError {
  code: "storage",
  message: "atomic write failed: set temporary permissions: private object belongs to another user"
}
~~~

- Windows普通后端回归、clippy及后续release EXE/安装候选步骤被先前fmt失败跳过；always隔离凭据步骤实际编译并执行后失败，不能把这两种失败混成“MSVC编译失败”。同一步后续channel_rename/native_secure_store未执行。
- backend（macOS）job127999232030同样因cargo fmt失败；Windows smoke和release-tools已成功。前端及总run是否终态以最终workflow-jobs状态另记录，不用中途快照冒充总体完成。

后继终态核对：run42249148982已completed/failure；frontend job127999232280为failure，27个测试文件失败/177通过（总204），147项测试失败/1231通过（总1378）。汇总build job128006012666被跳过。完整Windows日志与ci-terminal-verification.json归同一批终态证据；ci-frontend-failure-excerpt.log仅为连接器所读原日志的筛选摘录，保留原job id定位，不冒充完整前端日志。未重跑任何job、未删失败记录或降低门禁。

**定位与L0最小修正需求**

本版platform::set_mode已经接严格current-user owner验证；atomic_write仍通过OpenOptions create_new/create_mode创建临时文件，而Windows create_mode是no-op。native provider用例在隔离临时根暂存合成凭据时，到达protect_current_user_path的owner!=TokenUser拒绝路径。

直接证据只有“临时对象owner不等于当前用户SID”这一错误。提权runner默认TokenOwner可能为Administrators，是合理的待验证解释，日志目前没有输出actual owner SID和TokenOwner，不能将其写成已证根因。请L0在同源runner合成夹具中记录：process TokenUser、TokenOwner、token elevation、parent owner、刚create_new的临时文件owner，以及stage前后路径；不打印合成秘密内容，不更改日常Profile。

共享创建/原子写边界归L0，本线不扩大写集修改atomic_write、platform、data_root或测试文件来掩盖失败。建议修正准则：

1. 对新建、明确归本操作所有的文件/目录，在创建handle时使用显式当前用户owner及私有DACL（SECURITY_ATTRIBUTES），读回核对；保持只作用于本次新建对象。
2. 对已存在、其它owner/未知身份根继续拒绝。不允许简单接受BUILTIN\\Administrators/SYSTEM、自动take ownership、关闭owner check或把mode=600降为no-op来通过测试。
3. 同一规则应覆盖用户根/分区的create_dir和原子临时文件，不只修改provider_auth_native的TempDir用例或添加ignore。L4此前已回交“提权新建owner需统一策略”的缺口，本轮真实MSVC结果证实此缺口需要处理。
4. 当前提供的helper只保护既有owned对象，不负责编造来源证明。若L0需要新增私有创建helper，由L0给出共享调用点和最小函数签名，L4可在R2既有Windows helper范围实现并验原生行为；共享接线仍L0单写。
5. fmt失败的10个公共文件清单见前批local-format-summary.json；由对应owner修正后公布后继SHA。不绕过fmt/clippy，不以重新选择旧基线替代当前目标。

**证据可取得性与残余环境**

ci-backend-windows.log为连接器下载的完整解码日志，保存原时间戳/失败/CRLF。msvc-failure-excerpt.log仅定位摘要，不能代替原始日志。actions artifacts为windows-startup-evidence（id13520071162、252261 bytes、sha256:56bf723146de242498b40bb6024e9915d2d28de539792cb8c0675665b8fa1055）与windows-process-token-evidence（id13517894778、6911 bytes、sha256:0bcb5b163cfe2d985b55a01a43c71bd7fbdadec618c0535e8729d4d4135ba96d）。名字中的startup不代表正式运行包；完整exe构建/归档步骤已跳过。

本机无法解析GitHub日志/ZIP重定向域和连接器临时文件域，ZIP未下载、未逐字节核对或提取；记录Actions id/digest供L0获取。没有为下载修改DNS/代理或日常配置。成功smoke原始解码日志已在前批Git证据中。

第二账户受控通道尚未提供；两SID GUI/IPC/DPAPI交叉拒绝、同SID跨会话/提权、整机端口及进程归属仍未验。Windows CI自己的Profile-only临时账户不是这些验收的替代。最终离线runtime/NSIS/MSI制品仍等L0实际可取位置及hash。

贡献代码400c4e7f4已推送、当前工作区干净，本轮不改生产文件。之前被审批拦截的test TEMP删除没有重试；日常配置/账户/服务不变，不真实同步/推理或发布。L4保持未完原生验收；本失败不是关闭任务的依据。
