---
id: REL-OPENCODEX-DESKTOP-18
object_kind: release
state: candidate
title: 0.2.15 配置传输、Agent 与旧配置迁移发布
summary: WP-82～WP-84 及共同迁移清理已实施，本地门禁通过，PR23 和双平台 CI 通过，已合并 main 并打 v0.2.15 tag；制品验收在途，尚未发布 stable。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/23
---

# 0.2.15 配置传输、Agent 与旧配置迁移发布

2026-10-27，Asia/Shanghai。用户已授权文档、开发、测试、发布；范围见 [IMP-24](IMP-OPENCODEX-DESKTOP-24.md) 与 [问题清单](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.14-bugs.md)。

本地 21 项门禁、903 前端和修正后 1254 后端测试通过；Chromium / WebKit 实际文件导入到保存重开各 20 项通过，HTTPS 回环 WebDAV/Agent 官方路由受控适配分别记录。迁移 22 项与 17 项消费者处置记录完成，旧新增渠道模块及命令清除，必要历史恢复、候选、凭据、任务回执保留。界面沿唯一总原型，无新状态行/弹窗/核对流程。

PR #23 已合并，head aa567e7a，main/tag 6a8e06c6ec87951f823ce50e179026bf089ac1be / v0.2.15；PR CI 40452748873 五个必需 job 通过。前一 CI 的 Windows 锁读取问题已修正；独立重启 fixture 保留首次 marker 时序失败，原样重跑 5 场景/40 项通过。Release 40455658537 双平台出包和安装器检查在途，更新签名、公开/stable、资产摘要在完成后回写，本记录目前不作为已发布证明。

验证仅使用隔离文件、固定凭据、回环服务与 mock 原型；未写用户日常配置、未付费生成、未强行重启活跃 Codex。OS 安装包继续未签名测试分发，更新签名分开核验。回退前备份当前配置及迁移归档；不得删除用户文件或自动将旧副本当当前渠道，若需回退 stable 应沿既有显式通道流程。
