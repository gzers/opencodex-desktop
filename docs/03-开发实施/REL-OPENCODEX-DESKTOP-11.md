---
id: REL-OPENCODEX-DESKTOP-11
object_kind: release.candidate
state: in_progress
title: 0.2.0 模型配置与范围同步发布
summary: PR8已合并，main及v0.2.0固定f1bbe7d9；当前进行主线CI和双平台私有草稿构建，实际制品安装、验签与公开通道尚待。
source_refs:
  - IMP-OPENCODEX-DESKTOP-21
  - f1bbe7d996389097dcf6237a6bc65b7995d68190
---

# 0.2.0 模型配置与范围同步发布

2026-10-21，Asia/Shanghai。本对话用户已明确授权目标模式从0.1.10 stable完整实施至测试与发布，含提交、集成及投递；WP-06内部生成/持续纠偏、公开资料检索和外部真实提交全部纳入。账户范围经用户原型确认，当前仅受支持信息，同平台多账户独立，新设备需登录。使用与升级/回退说明沿模型配置协作包发布目录，用户版本说明唯一归main根CHANGELOG。

## 源码与范围

独立feature/0.2.0-model-config从0.1.10 stable开始。PR8最终head b19b01e2，完整CI39455864253通过，前端143文件710项；Linux1127通过/11忽略、Windows843通过/10忽略均保留逐组原始范围，忽略项不计通过。真实installed2.84.0配置/恢复与协议wire、TLS WebDAV双根冲突/范围、不可变版本及权限/锁/幂等、两平台原生模型交互、macOS多轮与多建议选择另存、复制/取消/缓冲和实际Codex菜单均沿IMP-21源码绑定证据。

PR8于2026-10-21T02:33:41Z合并为53816867d；main随后f1bbe7d996389097dcf6237a6bc65b7995d68190只增加0.2.0 CHANGELOG及共享中英文README，apps/.github/test树与已验b19严格相同。main CI39533974950与Release39534419661固定同一源码，v0.2.0已推送。中间合并提交的重复CI39533699701已取消；不算失败或通过，不用它替代最终main检查。

完整原条件与证据关系见.adg/evidence/OCX-020-IMPLEMENTATION-20261018/acceptance-source-review-20261021.json，包含原94模型/32同步/12通知/8按钮/43组件，沿原编号而未另建AC。模型/范围同步与公共UI的自动、实屏、原生证据分别使用；源码映射不能替代实际行为。原版有效且未变证据按字节资格复用，REL-10未验或例外保留旧结论，不自动升级为本版通过。

## 交付边界

- 完整运行目录当前兼容预算100条，超限保留清单/草稿并拒写；模板库和schema没有永久100项限制。
- 最终协议与能力经过共同保存门；installed2.84.0未兑现的Responses/Anthropic显式高级推理请求约束写前拒绝，候选保留，不静默改协议。
- 模版候选、以后默认、精确采用、运行保存及客户端核对各自记录；实际CLI菜单和app-server读取各有来源，不自动关闭/替换用户日常软件。
- 账户信息同步不携登录/刷新令牌；原免登录/旋转真实门保留后续且未验证。
- 安装包沿既有测试分发策略未作OS正式代码签名/Apple公证；updater资产须实际签名并使用现有信任公钥完成密码学验证。

## 当前发布门

main完整CI、双平台Release、实际NSIS/MSI安装及卸载保留、Mac容器/压缩包身份、更新包旧公钥验签、公开/stable与匿名清单/资产验证都需取得各自真实结果。Release仍默认draft/prerelease，出包或上传成功不等于公开。当前未公开0.2.0，不提前宣告稳定投递。后续结果只在实际完成后追加。


### 双平台草稿制品与独立验签

Release39534419661在f1bbe7d9完成双平台出包。Release ID419091323，仍draft/prerelease。三种实际updater资产全部下载、大小/API摘要核对并以tag内原公钥minisign密码学验签通过；原公钥解码SHA30190687…与v0.1.7相同，未轮换信任。

| 资产 | 字节数 | SHA256 |
| --- | ---: | --- |
| Mac app.tar.gz | 6454019 | a7178fa6f66d0f930fba5e74dec1286154b80362ab50cca76fc918173d8ad50f |
| Windows MSI | 6701056 | dd8641ca3d6fdff7c5400b58da949e37e72580f2efa138c331b478be2e924ea9 |
| Windows NSIS | 4774724 | 19ff816d2c191312584865eca968995d9a32642879c7af2f4de5d1fa7efe4b8f |

Mac DMG6347202字节/SHA0e602441…容器校验通过；压缩包版本0.2.0/production标识/内嵌main均正确。严格OS codesign仍exit1，按既有未正式签名/未公证测试分发边界保留，不混同updater已验签。

Windows Release真实MSI/NSIS均成功安装，实际EXE包标记/源码/0.2.0首屏匹配，默认卸载保留数据，隔离清理helper拒绝未确认、确认后只清受管根且外部HOME不变。Built EXE918c9f80…与每安装器预期标记后的installed字节核对通过；不以标记合法差异误报相同哈希，不宣称人工DPI/OTA或日常升级。发布工作流本身只出私有草稿，releaseAccepted字段仍false，原回执保持。

Mac发布tar逐字节相同Mach-O4321c151…制作仅Info.plist改隔离身份的副本，显式独立root；PID92731首屏及关于页读得v0.2.0，启动日志commit f1bbe7d9，Command-Q后进程不存在。未替换日常安装/改日常配置。证据统一.adg/evidence/REL-020-20261021/。

### main CI首次失败与限定复跑

main39533974950首次Linux/前端/发布工具与准入通过，Windows原始冷版本探测3项通过；大套件733 passed/4 failed/4 ignored。失败4项为旧about::remote::windows_tests元数据子进程fixture，2秒查询内无parent-started等任何标记，5秒observer也未见父子PID，控制进程仍活。该组源码与v0.1.10未变，特性b19完整CI曾通过；不据此忽略当前失败，也不称业务保存回归。保留原始日志并仅复跑该失败Windows job一次，deadline/断言/默认线程数及产品代码全不变。第二次结果尚待，当前不公开/晋升stable。
