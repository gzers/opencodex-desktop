---
id: REL-OPENCODEX-DESKTOP-18
object_kind: release
state: candidate
title: 0.2.15 配置传输、Agent 与旧配置迁移发布
summary: WP-82～WP-84 及共同迁移清理已实施，本地门禁通过，PR23/平台 CI 在途；尚未发布 stable。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/23
---

# 0.2.15 配置传输、Agent 与旧配置迁移发布

2026-10-27，Asia/Shanghai。用户已授权文档、开发、测试、发布；范围见 [IMP-24](IMP-OPENCODEX-DESKTOP-24.md) 与 [问题清单](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.14-bugs.md)。

本地 21 项门禁、903 前端和 1253 后端测试通过；Chromium / WebKit 实际文件导入到保存重开各 20 项通过，HTTPS 回环 WebDAV/Agent 官方路由受控适配分别记录。迁移 22 项与 17 项消费者处置记录完成，旧新增渠道模块及命令清除，必要历史恢复、候选、凭据、任务回执保留。界面沿唯一总原型，无新状态行/弹窗/核对流程。

PR #23，当前 head f698f363；平台 CI 40449116192 在途。最终 main/tag、双平台安装器、更新签名、公开/stable、资产摘要在实际完成后回写，本记录目前不作为已发布证明。

验证仅使用隔离文件、固定凭据、回环服务与 mock 原型；未写用户日常配置、未付费生成、未强行重启活跃 Codex。OS 安装包继续未签名测试分发，更新签名分开核验。回退前备份当前配置及迁移归档；不得删除用户文件或自动将旧副本当当前渠道，若需回退 stable 应沿既有显式通道流程。
