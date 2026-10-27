---
id: REL-OPENCODEX-DESKTOP-19
object_kind: release
state: verified
title: 0.2.16 渠道详情独立保存与原型对齐发布
summary: WP-85～WP-88及独立复核补修已实施；双平台CI/Windows原生EXE验收通过，PR25合并并打v0.2.16，正式安装器及公开/stable、匿名manifest与三个更新包签名全部通过。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/25
  - d2c08dd7d62e2caada507f2eeb4d05804460b6a8
---

# 0.2.16 渠道详情独立保存与原型对齐发布

2026-10-27，Asia/Shanghai。用户已授权文档、开发、测试到发布；实施范围见[IMP-25](IMP-OPENCODEX-DESKTOP-25.md)与[WP-85～WP-88](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.15-bugs.md)。本版本已公开并晋升stable/Latest，实际回执如下。

## 交付范围

连接字段直接编辑，取消局部重复确认与还原。详情保存当前渠道，成功返回列表；列表保存先预览全部剩余修改。无修改保存禁用，详情状态只反映当前渠道。实际文件与已存布局为基线，局部保存不带入其它新增、删除、排序或采用关系；预算按真实候选计算。

保存后接收最新未编辑字段，保留其它渠道和晚到输入、Key、型号缓冲、导入及模版意图。布局失败不得假成功；持config→layout锁读回，进程内失败进行有保护的补偿，不覆盖外部新内容。无新增计划准入、客户端核对、页面底部结果块或第二通知入口。

## 当前已验事实

- PR #25 head 7a4e14ba25301e522decc7d012d3f4b56095510d；main/tag d2c08dd7d62e2caada507f2eeb4d05804460b6a8，源码树与验收head完全一致。
- CI40480837148五个必需job通过；build为main-only，在PR按配置跳过。Windows实际EXE通过双渠道保存、真实鼠标排序、Key、剩余预览取消与确认、重开、模版和导入往返。首次原生探针错误已纠正，未将首轮失败冒记通过。
- 完整本地门禁21/21、前端985通过（10项既有条件跳过）、后端42套件1282通过（14项既有忽略）、fmt/clippy通过。Windows专属布局替换失败、锁超时及原生凭据保护在CI明确通过。
- Chromium/WebKit生产Rust隔离文件链各45项通过；通知亮暗×三档材质及HTTP资源84项对照通过。原型固定为已批准提交6956dd1f的完整共享父目录，不纳入同时进行的其它原型专题。

## 正式发布回执

| 阶段 | 实际结果 |
| --- | --- |
| 双平台CI | [40480837148](https://github.com/gzers/opencodex-desktop/actions/runs/40480837148)，五个必需job成功 |
| 正式构建 | [40490173181](https://github.com/gzers/opencodex-desktop/actions/runs/40490173181)，macOS arm64及Windows x64成功 |
| MSI/NSIS | 两种安装器版本0.2.16，安装/首屏/原生保存/重开/卸载成功，包摘要与发布资产一致 |
| 公开 | [40500890565](https://github.com/gzers/opencodex-desktop/actions/runs/40500890565)，先保留v0.2.15 Latest，三个更新包验签通过 |
| stable | [40503451702](https://github.com/gzers/opencodex-desktop/actions/runs/40503451702)，Latest=v0.2.16，三个更新包再次验签通过 |
| 匿名检查 | Release非草稿/非预发布；latest/download/latest.json version=0.2.16，8资产HTTP200与SHA256一致 |
| 本地独立核验 | 发布下载的macOS归档、MSI、NSIS均用既有公钥验签；macOS包内版本与标识正确 |

[下载v0.2.16](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.16)。[证据清单](../../.adg/evidence/0.2.16/manifest.json)与[14场景验收矩阵](../../.adg/evidence/0.2.16/acceptance-matrix.json)记录准确范围。Windows报告中的releaseAccepted=false表示安装器脚本本身不拥有发布授权，正式接受由本REL及发布回执确认。

## 发布资产

| 资产 | 字节 | SHA-256 |
| --- | --- | --- |
| latest.json | 5302 | 13057ef83b445d53b10ae712be87a0a6bbefbeb6456ec85a52e403f485be4fc3 |
| OpenCodeX.Desktop_0.2.16_aarch64.dmg | 6480984 | 0759b5d09b08f814049d8564add032afe6ba1f4722c9b513608587ee6e1c1774 |
| OpenCodeX.Desktop_0.2.16_x64-setup.exe | 4868794 | 40dfc659dde954fd6671732e8fc3a2d75124bdf958585348b2132bb11178f286 |
| OpenCodeX.Desktop_0.2.16_x64-setup.exe.sig | 452 | f379650b0c37b90bc2481e146c38ab159573aa171c8484d5c3ddb3c07280cb7e |
| OpenCodeX.Desktop_0.2.16_x64_en-US.msi | 6815744 | f3bbf281c9e018eb827e12e6cd22c1cbc5ca9000ca51a90e6806295c3cc8d8f2 |
| OpenCodeX.Desktop_0.2.16_x64_en-US.msi.sig | 452 | fdda26a96c631cdad1cbe0ab624ba21b18f94448cf1bf33d5e13d420d4ad9a41 |
| OpenCodeX.Desktop_aarch64.app.tar.gz | 6593510 | a65ef8eb53de97caf2b62e4f9ae12b2dc8b85b83ae49c88d3145969a7d7708a5 |
| OpenCodeX.Desktop_aarch64.app.tar.gz.sig | 440 | 77bb0da5a8673171a3b165d79f1e1efc5ebc7f05615f511501ec092e5863fd30 |


## 兼容与边界

无需清空配置，保留未知字段、有效凭据、导入候选、精确采用、历史恢复与备份。显式历史恢复仍独立处理；只清已证明废弃的局部连接快照、按钮接线与样式。

布局与配置支持进程内失败补偿，不承诺突然断电或进程崩溃时双文件原子性。回退前保留当前配置、布局与备份，stable回退沿显式通道流程，不移动旧已发布tag。

未替换用户日常安装或写日常配置，未进行付费生成或强制重启活跃Codex。WebKit引擎真实Rust文件链、Windows原生制品和更新包签名分别记录，不将浏览器mock等同原生安装。安装包继续采用未配置OS代码签名/Apple公证的测试分发，自更新验签单独完成；全部供应商、网络、登录和设备组合不在本轮证明范围。
