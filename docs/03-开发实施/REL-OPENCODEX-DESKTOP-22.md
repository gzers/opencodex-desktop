---
id: REL-OPENCODEX-DESKTOP-22
object_kind: release
state: preparing
title: 0.2.19 发布后修复版本发布
summary: 用户已于2026-11-02授权补充并发布；补齐CHANGELOG及正式安装/相邻版本OTA验收，待最终main、原包、签名、stable与实际OTA完成。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/33
  - 58d8bea1
---

# 0.2.19 发布后修复版本发布

2026-11-02，Asia/Shanghai。用户明确授权“补充并发布”，涵盖发布准备、合并、打tag、草稿包、原包验收、公开stable及隔离实际OTA。产品范围仍为[0.2.19单一清单](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.19-bugs.md)S18-01～09；冻结原型cfbd90c4b，未纳入并行全景/对话策略/扩展包改动。不修改日常配置、不执行真实WebDAV或跨设备配置同步。

发布准备：补齐CHANGELOG 0.2.19段落（2026-11-02）；正式安装白名单增加0.2.19；OTA增加0.2.18→0.2.19并保留旧组合。生产功能代码与已通过双轮CI的730b9e8b2相同。PR #33已合并，正式tag v0.2.19指向main cb4c9c9b7f342fe582ea6b6c95e3cf957611403c；main CI 41372541021与草稿Release构建41372617031已启动，尚未公开。最终身份、资产、门禁、失败轮次和回退结论据实际回执回写。

证据目录：[0.2.19正式发布](../../.adg/evidence/0.2.19-release-20261102/)。发布前保留0.2.18原始资产、签名及stable；公开前先核验正式字节和更新签名。公开后验证匿名资产/清单，以及MSI、NSIS真实stable更新、普通退出不安装、明确重启替换并自动重开、沙箱配置及合成凭据原字节保持。
