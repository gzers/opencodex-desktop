---
id: REL-OPENCODEX-DESKTOP-18
object_kind: release
state: verified
title: 0.2.15 配置传输、Agent 与旧配置迁移发布
summary: WP-82～WP-84 及共同清理已实施；PR #23/#24 合并，双平台 CI 及安装器验收、三个更新包验签通过，v0.2.15已公开并晋升stable/Latest。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/23
  - https://github.com/gzers/opencodex-desktop/pull/24
  - https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.15
  - 2e687a60a9acebb3749f6b5abc1c2bdd243b5b8b
---

# 0.2.15 配置传输、Agent 与旧配置迁移发布

2026-10-27，Asia/Shanghai。用户已授权文档、开发、测试至发布；范围与过程见 [IMP-24](IMP-OPENCODEX-DESKTOP-24.md) 与 [WP-82～WP-84](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.14-bugs.md)。

## 交付

文件导出和 WebDAV 使用当前 config.json；导入采用只形成内存候选并保留真实文件基准和未冲突编辑，确认保存才写所选授权与原字段。Agent 目录采用当前文件身份并刷新；模版任务独立使用模版库，范围、精确版本、人工锁、防重和回执保留。

迁移有效布局与身份，保护历史、候选、凭据和锁；只清已证明废弃且归档读回正确的终态记录，移除旧普通写者。异常在现有铃铛固定提示，正常成功/忙锁不添加任务；没有新增状态行、迁移待核对或额外保存流程，沿用 0.2.0 总原型。

## 源码与发布事实

| 项目 | 结果 |
| --- | --- |
| PR | [#23](https://github.com/gzers/opencodex-desktop/pull/23) / [#24](https://github.com/gzers/opencodex-desktop/pull/24)，均合并 |
| 最终 head | 750c9f35e0c9b0e57c694ecef1b5b90367c316f7 |
| main / tag | 2e687a60a9acebb3749f6b5abc1c2bdd243b5b8b / v0.2.15 |
| PR CI | [40457220770](https://github.com/gzers/opencodex-desktop/actions/runs/40457220770)，五个必需 job 通过；main-only build 在 PR 下按配置跳过 |
| 双平台构建 | [40460616784](https://github.com/gzers/opencodex-desktop/actions/runs/40460616784)，macOS arm64 / Windows x64 与 MSI/NSIS 安装器检查成功 |
| 公开 | [40462386552](https://github.com/gzers/opencodex-desktop/actions/runs/40462386552)，保留前 Latest 后公开 |
| stable | [40462525375](https://github.com/gzers/opencodex-desktop/actions/runs/40462525375)，Latest=v0.2.15 |
| 匿名检查 | 非草稿、非预发布，latest/download/latest.json version=0.2.15 |

[GitHub Release](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.15)。初次未公开构建 40455658537 在完整性复核发现异常通知缺失后取消，未产生发布资产；未公开候选 tag 撤回后才重建，旧 stable 版本 tag 未移动。原 Windows 锁失败和重启 fixture 时序失败/原样复验保留在 IMP 与证据中。

## 验收

- 发布门禁 21/21，0 失败、0 门禁跳过；前端 903 通过，10 项既有条件跳过。
- 完整后端 41 套件 / 1261 通过，14 项显式忽略；fmt、clippy 通过。迁移 macOS 22 项 / Windows 21 项，平台专属链接用例不冒计为通过。
- 最终 Chromium / WebKit 实际 Rust 文件链各 20 项；Windows 最终 EXE、NSIS、MSI 均验证鼠标拖排、Key 显隐、文件采用不写文件、确认保存及重开。原型 311 项和布局/通知配方对照通过。
- 回环 HTTPS WebDAV 用生产 reqwest/TLS 与隔离 CA 完成条件往返；Agent 生成请求参数由受控官方路由适配器核验。模版任务完整事务与损坏无关旧副本隔离检查通过。
- 迁移故障 4 项实际投递/原件保护，以及事件目录 41 项、持久化重开去重通过。通知使用固定文案/分类，不包含旧配置内容、Key、绝对路径或自由错误。
- 三个更新包在公开与 stable 流程分别验签；下载的macOS 归档独立用既有公钥验证并核对包内 0.2.15。

## 资产摘要

| 资产 | 字节 | SHA-256 |
| --- | --- | --- |
| latest.json | 5402 | sha256:9bc44e28874ca28777514499fdf11798aa55b2da20bd9fb2db5dee0ef5d6d67b |
| OpenCodeX.Desktop_0.2.15_aarch64.dmg | 6470096 | sha256:1092f904ab41cd292074e0005d3d9d886d44488ab8cb6d3f9a455b89ef0c4de5 |
| OpenCodeX.Desktop_0.2.15_x64-setup.exe | 4858809 | sha256:390c19d1ee0604e864275e281fa2df8275f4a12a3ef6061d8d957042a19d58a0 |
| OpenCodeX.Desktop_0.2.15_x64-setup.exe.sig | 452 | sha256:614a6725a1cf24987b0fa7b34cbf15baaf74598ec0145b511754c0bbf59404a8 |
| OpenCodeX.Desktop_0.2.15_x64_en-US.msi | 6807552 | sha256:a55e6f15ffb7ce69adfd1a9512392d40019ce679726e3dd6efacc150412ca88c |
| OpenCodeX.Desktop_0.2.15_x64_en-US.msi.sig | 452 | sha256:58fc016d42a9b923618a5d08c40d090135633056b237222f71becd4320e8d9b1 |
| OpenCodeX.Desktop_aarch64.app.tar.gz | 6579592 | sha256:786defab71efdedf7c2c9ac3a20ca15673c80abce103235f6fcab76c19b37393 |
| OpenCodeX.Desktop_aarch64.app.tar.gz.sig | 440 | sha256:5ccd3b5c2cf3ff2c5e6c70fa096410d9069fabc4da7efb2a4fdc26e99a2f9052 |

## 兼容、回退与边界

不要求清空配置。当前文件事实优先，未知未编辑字段、原始型号名、有效凭据与必要历史保留。回退前备份当前文件和迁移归档，不能删除用户文件或将旧副本自动反写为当前渠道；stable 回退沿既有显式通道流程。

没有替换日常安装或写日常配置，没有向付费模型生成、访问用户 WebDAV或物理重启活跃 Codex；隔离文件、回环服务与原生制品各自验收，不互称。安装包仍为未配置 OS 代码签名 / Apple 公证的测试分发，更新签名已单独核验。全部供应商、网络、登录和设备组合不在本轮证明范围内。
