---
id: REL-OPENCODEX-DESKTOP-20
object_kind: release
state: verified
title: 0.2.17 Windows托管安装可靠性发布
summary: WP89～95受支持范围修复及原生验收完成；正式原包复验、三包签名、匿名stable与实际MSI/NSIS OTA通过。物理设备和企业网络矩阵保留明确未执行边界。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/26
  - d7c7978f043278baae8f2fc8c2119315a4660f12
---

# 0.2.17 Windows托管安装可靠性发布

2026-10-28，Asia/Shanghai。[v0.2.17已公开并投递stable/Latest](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.17)。用户授权至发布；范围/实施见[IMP26](IMP-OPENCODEX-DESKTOP-26.md)，问题沿[WP89～95](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.16-bugs.md)。无需清空配置。

## 交付与真实身份

修复默认/扩展Windows路径、目标归属及无副作用准入、包体/入口/来源记录失败补偿、零退出有界版本探针、受控工具发现、包内许可、CLI描述。真包链路补修脚本确认及离线tgz参数，普通账户启动补修可选托盘外观刷新失败不应中止setup。保留既有原型、模型文件事实源、详情局部保存和通知层级。

PR26 head f77327aa8，原生EXE内CI合并7fe03af7，tag/main发布提交d7c7978f0，三者树68486e2b完全一致。PR27合并a354b130b、PR28合并65581b589仅维护验收脚本/工作流；apps/desktop与正式tag完全一致，不重建/移动v0.2.17。

## 回执

| 阶段 | 实际结果 |
| --- | --- |
| 双平台CI | [40632048037](https://github.com/gzers/opencodex-desktop/actions/runs/40632048037)，五个必需job成功，真实npm/故障回滚/凭据与最终EXE通过 |
| 普通账户 | 标准SID、elevated=false，真实WebView2及保存/Key/模版/同步界面业务通过，临时桌面DACL恢复成功 |
| 本地门禁 | 前端988/后端1289通过，既有10跳过/14忽略明确保留；21发布门禁全通过，19协议回归及PowerShell语法通过 |
| 正式构建 | [40636062852](https://github.com/gzers/opencodex-desktop/actions/runs/40636062852)生成macOS及Windows包和签名；macOSjob成功，Windowsjob因验收脚本引号解析失败，原失败保留 |
| 原包复验 | [40647352613](https://github.com/gzers/opencodex-desktop/actions/runs/40647352613)，真正MSI/NSIS×default/custom四个独立首次安装全部通过；原始摘要/EXE标记严格匹配，不重建包 |
| 公开 | [40647289835](https://github.com/gzers/opencodex-desktop/actions/runs/40647289835)，保留Latest=v0.2.16，三包验签通过 |
| stable | [40647642314](https://github.com/gzers/opencodex-desktop/actions/runs/40647642314)，Latest=v0.2.17，三包再次验签 |
| 匿名 | release非draft/非prerelease，latest manifest=0.2.17，8资产HTTP200/大小/SHA256匹配；原公钥三包独立验签 |
| 实际OTA | [40648471564](https://github.com/gzers/opencodex-desktop/actions/runs/40648471564)，公开0.2.16 MSI/NSIS经生产stable入口下载/验签；普通退出保留旧版、明确重启真实替换、OS观察新版自动重开；新版隔离读回配置/合成凭据原字节不变、pendingRestart清除 |

[证据矩阵](../../.adg/evidence/0.2.17/acceptance-matrix.json)、[安装器逐项回执](../../.adg/evidence/0.2.17/installers-final-summary.json)、[OTA回执](../../.adg/evidence/0.2.17/ota-summary.json)、[资产与签名](../../.adg/evidence/0.2.17/release-summary.json)。初次“绿灯四项”曾被报告审查发现实际均NSIS，未接受；修正种类变量后按实际MSI/NSIS报告收口。

## 资产

| 名称 | 字节 | SHA-256 |
| --- | --- | --- |
| latest.json | 4913 | 7e5161fa8824f8d01f75b9f4f4519007adcc49f6512d03d51d3de437cadc898a |
| OpenCodeX.Desktop_0.2.17_aarch64.dmg | 6493171 | 2e0dff42d24710431fe020cfd3f32143ad9b155e0d50416dc20f3ead00c3d1c9 |
| OpenCodeX.Desktop_0.2.17_x64-setup.exe | 4883438 | efa5254c27308c8e0b06aa0aff45ae4be674bf424bac52efcb79d9735e60526d |
| OpenCodeX.Desktop_0.2.17_x64-setup.exe.sig | 452 | b8fbde659e033616c8470cb38ba54ebc4a3c7adcf0e72f7fbabfb3094eeddce9 |
| OpenCodeX.Desktop_0.2.17_x64_en-US.msi | 6836224 | 2680d2e807f240f60c1447be43b524f67f6a16cba31e5c6b349733b34a056e5c |
| OpenCodeX.Desktop_0.2.17_x64_en-US.msi.sig | 452 | 372eedd1212e442c01afe2b3bf7ad943fcaea32801f166f6bce7873a9d2af6b3 |
| OpenCodeX.Desktop_aarch64.app.tar.gz | 6602236 | 294f13c58511ef28fff46c147c96973add55ce96b78c21da5a1ae3649eda8b77 |
| OpenCodeX.Desktop_aarch64.app.tar.gz.sig | 440 | 23b0b395b505e6ff7f35018c5d9890172adb3008b2fd9caa2b8ecc0b932ecd29 |

## 边界与回退

普通账户为准备后的临时Actions标准SID：Profile原生创建、仅该SID镜像错误LocalAppData修正、GUI权限临时授予并恢复。实际OTA自动重开与隔离读回是两份证据：安装器Shell重开丢弃测试CDP环境，因此自动新进程/版本由OS观察，随后用显式同一不可变沙箱重开更新EXE检查文件/IPC；不假称直接检查自动进程的CDP。

物理Win10/11、混合DPI、真实SMB/企业重定向、PAC/企业证书、私有服务及全交互设备/登录矩阵未执行；各域范围在验收矩阵说明。原型列表截图的部分文字渲染缺口不作像素一致证明，结构/行为/几何/材质检查分别记录。OS代码签名/Apple公证仍未配置，更新签名独立验证。

不改用户日常安装/配置、不强制重启活跃Codex、不进行付费生成。安装补偿是进程内恢复，不承诺突然断电跨文件原子；外部改动和恢复失败保留材料。回退前保留现配置与备份，stable回退走显式通道流程，不移动已公开tag。

## 发布后反馈纠正（2026-10-28）

用户Windows截图及发布源码复核确认：安装向导仍原样显示扩展路径及混用斜杠，上一轮WP89的可读展示/预填验收遗漏。原生安装、签名和发布回执保留，但不能将其扩展为该界面所有要求已解决；WP89显示子项重新待修，当前入口为[0.2.17问题记录](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.17-bugs.md)。截图下一步可用且未显示安装失败，用户机器后续实际安装结果仍未确认。本次仅分析记录，无软件改动或新发布。
