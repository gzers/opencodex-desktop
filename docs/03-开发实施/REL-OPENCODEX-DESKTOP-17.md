---
id: REL-OPENCODEX-DESKTOP-17
object_kind: release
state: verified
title: 0.2.14 文件读写、连接测速与原型通知统一发布
summary: WP-75～WP-81 已实施；PR22 合并 main 56c11220b，v0.2.14 双平台出包、安装器实测及三个更新包验签通过，已公开并晋升 stable/Latest；保留活跃宿主物理重启未验边界。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/22
  - https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.14
  - 56c11220b77b69de948f933e3b4c7fb93dae8256
---

# 0.2.14 文件读写、连接测速与原型通知统一发布

2026-10-27，Asia/Shanghai。用户授权文档、开发、测试、真实测速与发布，范围沿 [IMP-23](IMP-OPENCODEX-DESKTOP-23.md) 和 [WP-75～WP-81](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.13-bugs.md)。

## 交付

当前 config.json 是渠道事实源；内存编辑经已有保存预览确认后备份并原子写回，旧管理器草稿和计划不作为普通保存资格。已有 Key 在编辑器掩码加载、眼睛查看，Key 单独修改也可离线保存。清空输入保留原值，明确清空按钮才删除。只更新修改过的字段并保留其它渠道与未知配置。

测速和获取模型共用协议路径补齐及有限同源回退。保存后结果只出现在现有通知宿主，立即重启走官方独立接口；稍后即结束。通知入口、材质、字号和按钮沿 0.2.0 主原型，删除底部保存卡、额外收起入口和自创清理卡。历史恢复基准与普通文件基准隔离，模版建议只合并模版库，复制 Key 只在内存等待确认保存。

## 源码与发布事实

| 项目 | 结果 |
| --- | --- |
| PR | [#22](https://github.com/gzers/opencodex-desktop/pull/22)，已合并 |
| head | 47102d6473bb840fa7da08844212a6948dc5d586 |
| main / tag | 56c11220b77b69de948f933e3b4c7fb93dae8256 / v0.2.14 |
| 最终 PR CI | [40438355951](https://github.com/gzers/opencodex-desktop/actions/runs/40438355951)，五个必需 job 成功；PR 下 main-only build 按配置跳过 |
| 双平台 Release | [40440643137](https://github.com/gzers/opencodex-desktop/actions/runs/40440643137)，macOS arm64 / Windows x64 均成功，MSI / NSIS 安装器检查通过 |
| 公开 | [40442295245](https://github.com/gzers/opencodex-desktop/actions/runs/40442295245)，保留前 Latest 后公开成功 |
| stable | [40442363496](https://github.com/gzers/opencodex-desktop/actions/runs/40442363496)，成功晋升；Latest=v0.2.14 |
| 公开状态 | 非草稿、非预发布；匿名 latest/download/latest.json 的 version=0.2.14 |

[GitHub Release](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.14)。CI 的临时合并引用 5a0cc999 与实际发布合并点 tree 相同，见 [来源对应](../../.adg/evidence/0.2.14/source-trees.json)。发布没有移动旧 tag，也未替换用户日常安装或重启活跃 Codex。

## 验收

- 本地发布门禁 21 / 21，通过，0 失败、0 跳过；前端 872 通过、10 条按既有条件跳过。
- Rust 41 套件、1213 通过、13 条显式条件忽略；fmt / clippy -D warnings 通过。忽略项不计入通过。
- Chromium / WebKit 实际 Rust 文件保存各 13 项、通知交互各 56 项；原型通知配方对照 20 项、原型回归 287 项通过。
- 最终 Windows WebView2 与 MSI / NSIS 安装器各自执行真实鼠标拖排、保存预览确认、目标文件读取及整页重开；现有 Key 掩码/眼睛与仅排序保存的 Key 保留均通过。此前旧脚本要求磁盘草稿重载造成的失败及修正保留在 IMP-23，未绕过失败。
- 真实后端只读清单请求：GZGG-NewApi 318ms / 29 个模型，FeiMao 892ms / 6 个模型；26 渠道隔离配置副本写入读回并通过官方校验，用户原文件未变。
- macOS 下载到的实际更新归档用原公钥独立验签通过，包内版本为 0.2.14。公开与 stable 两次工作流均下载实际资产并核验三个更新包签名和摘要。

## 更新包摘要与签名

| 资产 | SHA-256 |
| --- | --- |
| macOS app.tar.gz | 3bf95a1e2733dbf04bdb37354df5c808f4fc1c35d1e6c8296a87fdf3b1a03b5f |
| Windows MSI | 88f576aea1fb42197c4352811f5e5a52999d61120991b934c7e21db855c7c477 |
| Windows NSIS | a8ab9249bf07b3b872d35a6a8f5ecb3eea9f49eb4885b44dffdb5d58eca4236e |

完整资产、下载地址、run、签名日志和验收索引见 [release-summary](../../.adg/evidence/0.2.14/release-summary.json)、[manifest](../../.adg/evidence/0.2.14/manifest.json) 与 [RCP](../../.adg/work/0.2.14/RCP.json)。TASK 六个 required_checks 均有同名通过回执。

## 边界与回退

立即重启会结束活跃 Codex 对话，本轮没有在当前宿主强行执行；17 项官方接口受控测试证明调用与返回分类，不宣称设备重启已验。真实测速限于上述两个渠道，不能覆盖所有供应商、代理网络和凭据组合。Windows 原生/安装器检查不等于所有日常真机状态、触屏与网络组合都已覆盖。

OS 代码签名 / Apple 公证仍未配置，安装包为未签名测试分发；自更新签名与 OS 代码签名分开记录。保留精确写前备份及真实文件冲突保护。回退版本前先备份当前配置，不删除用户原文件或将历史计划重新作为当前数据；若需回退 stable，按既有发布流程显式晋升上一版本并验证更新通道。
