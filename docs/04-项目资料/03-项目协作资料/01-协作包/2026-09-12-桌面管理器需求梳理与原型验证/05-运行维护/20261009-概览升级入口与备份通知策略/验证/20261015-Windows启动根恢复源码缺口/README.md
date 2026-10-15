# Windows 启动数据根恢复的源码接线缺口

2026-10-15，Asia/Shanghai。源码审阅发现`WIN-0110-STARTUP-ROOT-RECOVERY`，状态为open-source-wiring-gap，尚未修复或做普通账户真机复现。它属于原IMP-20第4节“不可写时明确提示选择自定义根”的实现范围，不是新增验收要求。

审阅固定`feature/0.1.10-windows-query-etw@3c51367f1be89abf66f69805d29b2a660a570a2c`的7份Git原文件，逐项核验与431阶段候选对应产品文件相同。证据只保存blob/SHA和调用位置，不在文档分支复制生产源码。

## 已确认的接线事实

- `src/lib.rs`的setup先选择安装目录默认根，再执行`bootstrap::resolve_locked_with_boundary`；错误通过`?`离开setup，发生在`SharedDataRootAnchor`等业务状态注册前。
- bootstrap先获取根锁再初始化。`modules/instance/mod.rs`创建manager-state或打开锁文件失败时返回文件系统错误。
- 当前这个调用点没有启动根恢复分支。设置中的`switch_data_root`需要已初始化的`SharedDataRootAnchor`，因此设置切根不能替代失败启动时的恢复入口。
- 锁定Tauri为2.12.1。官方对应标签源码确认：用户setup错误映射为Setup error，事件循环Ready时调用setup，返回错误即panic。项目的`build().expect`不是该setup错误处理器。初次只看项目代码时的错误路径推断已更正并保留初稿。

相关路径、准确行号及Git身份见`.adg/evidence/OCX-0110-20261015-WIN-STARTUP-RECOVERY-REVIEW/source-review.json`，框架来源/摘要/定位见`framework-source-review.json`。官方框架源码仅在内存核验，没有复制完整第三方源码，也不将标签源文件摘要混同锁文件的crate包校验和。

## 影响及下一步

重点场景是全新Windows安装、无旧AppData根，普通用户不能在安装目录下创建或打开data/manager-state。当前接线不具备原契约要求的明确根选择恢复。这是已确认的源码缺口；本轮没有观察实际Windows报错界面、退出码或权限拒绝，不能把它写成真机故障复现。

下一步先实现并测试业务服务初始化之前的显式恢复路径：给出可操作提示和用户明确选择，验证并持久化合法选择后才启动正常服务；取消/失败保留原根和绑定。损坏/过新结构、实例锁冲突和未决迁移不得混作写权限问题，不能以静默AppData回退、管理员常驻或放宽ACL绕过。

修复后按新源码/制品验证普通权限拒绝写入、选择根后的启动与重启持久、普通/管理员同配置根，以及取消/无效目标保持原数据；不重复已确认的窗口操作或安装器指定文件保留。独立普通测试账户仍待用户确认，前台仍需新开始信号。

## 证据与保护

10项归档清单/verify-review通过只证明本轮源码审阅的来源和边界，不是10项产品测试通过。复用保护采集方法的初稿场景标签误保留runner文字，原快照另存后以准确“本机只读源码审阅”标签重新采集；无产品或系统动作变化。

18:16:08独立保护快照native0、代码3c干净，日常EXE、旧隔离四摘要、原TASK3878bytes/SHA bc7965b4…保持。没有新CI、库回放、ETW、产品/安装器启动或账户/ACL更改。此前MSI/NSIS安装器文件保留子集仍有效，本发现不撤销它，也不将它扩大为整个storage/installer门禁通过。

原bd2/e6失败窗口ETW和根因、Mac性能failed、全部原预算/真实周期/八项门禁继续开放，不关闭、合并、签名、晋升或发布。
