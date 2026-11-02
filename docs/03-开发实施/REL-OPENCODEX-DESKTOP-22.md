---
id: REL-OPENCODEX-DESKTOP-22
object_kind: release
state: verified
title: 0.2.19 发布后修复版本发布
summary: v0.2.19已公开stable；main六项CI、八原始资产、三更新签名、四安装组合、匿名下载与MSI/NSIS实际0.2.18→0.2.19 OTA全部通过。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/33
  - https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.19
  - cb4c9c9b7f342fe582ea6b6c95e3cf957611403c
---

# 0.2.19 发布后修复版本发布

2026-11-02，Asia/Shanghai。[v0.2.19已公开并投递stable/Latest](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.19)，公开时间02:19:48Z（北京时间10:19:48）。用户明确授权“补充并发布”。范围为[0.2.19清单](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.19-bugs.md)S18-01～09，冻结原型cfbd90c4b；未纳入并行全景/对话策略/扩展包改动。未修改日常配置，未执行真实WebDAV或跨设备配置同步。

## 源码与交付身份

PR #33已合并；main和正式tag v0.2.19均指向 `cb4c9c9b7f342fe582ea6b6c95e3cf957611403c`。生产功能代码与已完成双平台复核的730b9e8b2相同。发布准备提交1e1c8eab补齐CHANGELOG；58d8bea1扩展0.2.19正式安装与0.2.18→0.2.19 OTA验收并保留旧组合。Release ID为430987382。

修复包括快照来源与单次预览确认、MCP来源选择与node_repl默认排除、Skill同名覆盖、忽略官方openai、Windows面板内嵌和启动反馈、Windows Mica/实底回退、渠道模型连续保存、表格选择列和公共通知。既有实施偏差及未验环境仍保留在原清单；本次发布通过不消除这些边界。

Windows原始构建EXE SHA256：`8ab722398fca455a6c580757571db0282bfc260d4d31993feed930db3b0f1a11`。macOS正式EXE SHA256：`391d48bca38f381c3d864b699fe17b762ae0cf8b36301e3ae6268a53c937d8fa`。公开前后八资产ID、大小、digest一致，未重建或替换原包，未移动tag。

## 验收回执

| 阶段 | 实际结果 |
| --- | --- |
| 开发复核 | macOS/Windows与冻结原型回归见原清单；Vitest1064通过/10跳过，WebKit材质31/31、几何40/40；初始Rust不稳定失败及完整复跑保留 |
| 发布工具 | 离线发布29项、ETW13项、保留策略6项通过；验收脚本允许0.2.19及0.2.18→0.2.19 |
| 最终main | [41372541021](https://github.com/gzers/opencodex-desktop/actions/runs/41372541021)六job全部通过，含Windows MSVC、两版Node安装/失败/恢复、普通账户、最终EXE和macOS集成打包 |
| 正式构建 | [41372617031](https://github.com/gzers/opencodex-desktop/actions/runs/41372617031)生成八资产；macOS job成功，Windows串行安装验收失败保留，具体见下节 |
| 原包四安装 | [41373902280](https://github.com/gzers/opencodex-desktop/actions/runs/41373902280)四独立runner的MSI/NSIS×default/custom全通过；原始EXE恢复与hash严格匹配，资产未重建 |
| macOS原包 | 三更新包minisign独立通过、公钥与0.2.18相同；manifest和sidecar解码一致。原app归档版本、tag源码嵌入、许可资源、codesign严格检查、DMG完整性与沙箱首屏/设置页通过 |
| stable公开 | [41377101811](https://github.com/gzers/opencodex-desktop/actions/runs/41377101811)成功；非draft/非prerelease，Latest=0.2.19 |
| 匿名资产 | 八资产HTTP200、大小/SHA匹配，版本与stable两份清单一致；无Authorization头 |
| 实际OTA | [41377311243](https://github.com/gzers/opencodex-desktop/actions/runs/41377311243)MSI/NSIS均通过：生产stable下载验签、普通退出仍旧版、明确重启替换并OS观察自动0.2.19进程，配置与合成凭据原字节保持，pendingRestart清除 |
| 重复OTA轮次 | 触发连接重试造成第二轮[41377322313](https://github.com/gzers/opencodex-desktop/actions/runs/41377322313)，同样成功；不重复计入覆盖范围 |

完整证据见[正式发布证据](../../.adg/evidence/0.2.19-release-20261102/README.md)、[原包核验](../../.adg/evidence/0.2.19-release-20261102/verification.json)、[四安装](../../.adg/evidence/0.2.19-release-20261102/installers-summary.json)、[匿名下载](../../.adg/evidence/0.2.19-release-20261102/anonymous-public.json)与[实际OTA](../../.adg/evidence/0.2.19-release-20261102/ota-summary.json)。

## 正式资产

| 名称 | 字节 | SHA-256 |
| --- | --- | --- |
| latest.json | 4592 | 343e9acf73f4d055d94a25f0b4fcdc5eb2d4edbba619595c830fba0efdbc8aa5 |
| OpenCodeX.Desktop_0.2.19_aarch64.dmg | 6615601 | 378af2c8817d3a48ff6b4d71b6c8899036363501c0b274e754c16d46110cfb63 |
| OpenCodeX.Desktop_0.2.19_x64-setup.exe | 4967844 | 72e00a1f70e1630fd3f46a66b30ea1b2f89480b2c0a34551e1d727d5128f5d03 |
| OpenCodeX.Desktop_0.2.19_x64-setup.exe.sig | 452 | e402ff9622c3b9fa0066ddb0e7cc4287be43d59ab3df11cf3ecb3d178d6bbe1d |
| OpenCodeX.Desktop_0.2.19_x64_en-US.msi | 6955008 | 63b26f2b19c5d0b05ab9c98fa775d260d11b077969b21ff12c024ac2f55b7dc7 |
| OpenCodeX.Desktop_0.2.19_x64_en-US.msi.sig | 452 | 33cd0a4a2a8bcf97ff9b04e76046ab091dbe18399375243ef3c33c653e52e023 |
| OpenCodeX.Desktop_aarch64.app.tar.gz | 6719009 | 27b8e246660f55f4c9dfe9ad296aec7d7de2aa7d238addc835733aab17f2f0f8 |
| OpenCodeX.Desktop_aarch64.app.tar.gz.sig | 440 | 015397e20f2895a86914c5476238e8d17582ee369b455ec73cfaf08f9bcee4a1 |

## 失败、限制与回退

首次Release Windows安装步骤失败：同runner先默认安装MSI、再默认NSIS，后续MSI自定义安装时记忆目录覆盖INSTALLDIR；msi-install.log明确记录从临时路径改为前次NSIS路径，退出码0但预期自定义路径没有EXE。该轮保留failure，不重跑构建；四独立runner沿同一资产确认各组合通过。此四组合不代表任意交叉安装历史；安装路径记忆行为未在本次改写。

本地旧Python不支持tar解包filter参数，改用本机Python3.11完成正式原包核验；GitHub临时EOF触发重试，HTTPS OAuth缺workflow scope而SSH成功推送，连接器403由gh完成PR操作。没有扩大凭据权限。签名私钥只在Actions秘密配置中使用。

macOS为arm64 ad hoc代码签名，无Apple公证声明；Windows无OS代码签名，更新包签名独立验证。原包实屏是显式沙箱运行，不替换日常安装。实际OTA自动重开由OS观察；为读回文件和IPC，只停止该次测试进程后以同一沙箱显式重开更新EXE，不假称自动进程提供CDP。

物理Win10/11全矩阵、企业代理/证书、混合DPI及真实跨设备同步未扩大验收。客户端MCP键路径适配仍归扩展包；同步流程与备份等原有偏差沿原清单。

保留上一正式[v0.2.18](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.18)资产和签名。需要回退时先保留当前配置/备份，stable回退须显式流程授权；不移动公开tag，不把旧包自动降级当作已支持。当前0.2.19发布、main验证和实际OTA均完成，按本次支持范围关闭。
