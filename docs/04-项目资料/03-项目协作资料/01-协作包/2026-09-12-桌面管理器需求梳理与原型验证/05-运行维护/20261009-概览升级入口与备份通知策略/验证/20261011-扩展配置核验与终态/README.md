# 扩展配置核验与终态

源码：feature/0.1.10-maintenance / 2064085ead3dbdc55105607e9008a760c23f1ef4。证据：.adg/evidence/OCX-0110-20261011-EXTENSION-CONFIG。

## 核验与隔离

ToggleClient / SetSourceDir / SetSyncMethod 及独立 toggle 入口共用配置终态。根 / HOME / 配置路径 / 锁路径安全预检通过并取得合作锁后才开始；锁保持至配置文件最终精确回读及终态回调结束，完整字节一致才成功。除明确 NotFound 外的读取错误不当作首装默认。读取 / 保护备份 / 写入 / 回读失败均报告失败，准入前拒绝不虚构执行终态；通知失败不覆盖领域结果。

成功只认证管理器扩展配置。路径仍可能投射 Skills，但本轮没有增加资产核验或跨文件回滚保证，配置成功不能解除客户端资产失败。ExtensionConfigSave / ExtensionConfiguration / Save / local 使用用户触发 extension-config-save-failed / succeeded。候选绑定完整命令与无损 HOME，仅摘要 / 不透明身份持久化；同根世代 / 动作 / 通道 / 候选成功才解除捕获的失败。偏好关闭仍解除已有异常，候选轮换不复用旧身份。

## 验证

最终 Rust 34 组 / 857 通过 / 0 失败 / 5 忽略，包含扩展集成 39/39、注册表 33/33；fmt / Clippy all-targets -D warnings / diff 检查通过。新增测试覆盖三类配置命令腐败文件拒绝、显式夹具修复、持久异常重载重试、保护备份拒绝、越界配置 / 锁预拒绝、资产命令无配置回执、命令 / HOME / 通道 / 偏好隔离及终态持锁。

首次 focused 缺少 SupportedEvent、第二次误用私有模块、首次 Clippy needless borrow 失败原件保留；第三次 focused 和首次全量是中间源码结果。只有 final-* 绑定最终补丁。最终回读没有单独注入投射后破坏，不宣称该分支实验覆盖。真实临时文件与持久 NotificationPublisher 不替代原生 AppHandle 投递。

注册表 8 triggers / 39 jobs / 80 events（78 active / 2 planned）/ 27 policies / 86 emissions / 8 scheduling / 8 feedback / 12 paths / 4 cleanup。无新计时器 / 路径，未改 UI / 未重跑原生。

## 剩余范围

十个现有资产命令需独立核验后注册终态：LinkSkill、UnlinkSkill、UninstallSkill、WriteMcp、RemoveMcp、ImportSkillArchive、RestoreSkill、AddMcp、EditMcp、ResyncSkills。它们是现有产品入口，不能统一退为未来 0.2.0；generic Migration / Backup / Sync 保持 planned 0.2.0。

损坏保护受控修复、Windows / 安装器、真实更新 / 重启、配对 CPU / RSS / p95 和真实 stable24h / beta6h 仍待验。TASK-182 / IMP-20 保持 in_progress，不回写稳定核心、不关闭或发布。
