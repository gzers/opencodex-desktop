---
id: REL-OPENCODEX-DESKTOP-12
object_kind: release.candidate
state: in_progress
title: 0.2.1 公共UI与原型还原修复发布
summary: PR9已合并，候选804ef0f6的两轮完整CI和Windows原生验收通过；main固定fbd6e1d1，准备双平台草稿制品，尚未公开或晋升stable。
source_refs:
  - IMP-OPENCODEX-DESKTOP-22
  - 804ef0f646a978d9498404c2b2dc937b3c5c4baf
  - fbd6e1d17a9029c89807100dab6f3f7089559c44
---

# 0.2.1 公共UI与原型还原修复发布

2026-10-21，Asia/Shanghai。用户已明确授权目标模式贯通文档治理、开发、测试、集成与发布；未授权替换日常安装/配置、真实账户/WebDAV写入或付费生成。发布前必须取得实际制品、签名和隔离回读证据；制作草稿不等于公开投递。

## 源码与修复范围

独立工作树ocx-021-ui-alignment、feature/0.2.1-ui-alignment从0.2.0 stable f1bbe7d9开始，按IMP-22分批修复UI-001～UI-015：公共图文按钮/信息行/分段单选、列表列宽、模型详情与主操作、同步主体范围、卡片与固定Tab组合，并补消费者守卫和逐类登记边界。账户能力、后端保存/传输事务、schema及签名信任不变。用户可见版本说明唯一归main根CHANGELOG。

候选804ef0f646a978d9498404c2b2dc937b3c5c4baf：本地145文件723项前端检查、构建与43项来源/采用图通过。CI39597935036（push）与39597940185（PR）全部适用job通过；非main的bundle job按既有条件跳过，不作失败或构建通过计数。

PR9于2026-10-21T12:50:06Z合并，main fbd6e1d17a9029c89807100dab6f3f7089559c44；apps/.github/test/CHANGELOG与已验804ef0f6逐字节相同。main集成检查及实际Release制品仍分别取得结果，不用特性分支源码测试冒充最终制品。

## 验收证据与边界

证据归.adg/evidence/OCX-021-UI-20261021，按源SHA和实际上下文记录：

- macOS隔离原生804ef0f6：列表/侧栏/分组/主体范围及初始化保护、按需连接、画质与Agent入口已实屏核对；保存本机范围只更新隔离scope文件且明确未上传。模版50/100/150/200%样例来自e993，相关详情/模版/分段组件与最终候选字节相同；未成功的窗口缩放尝试不算960×640通过。
- Windows真实WebView2：选择列34px、表头偏移0、图文中心偏移0、卡片gap16px；模板主保存50/100/150/200%均在749逻辑高度内。真实模版保存/默认确认、只读比较、外部上下文、文件往返和指针排序通过，没有发起模型生成。
- Windows无HOME/无运行时准入环境按真实三项准备门验收，不能要求或注入就绪卡；普通概览信息行由就绪Mac与同源Gallery验证。CDP透明通道不包含OS Mica合成，透明像素不误报为黑色产品背景；未变原生caption/托盘沿REL-11固定证据复用。
- 同源Gallery覆盖亮暗×高中低的分区样例和900×600下50/100/150/200%代表组合；这不是每页面/每平台/每状态的无限全矩阵承诺。所有登记、自动检查、实屏、原生复用分别表述。

首轮Windows18a2a0f5因测试错误要求就绪概览卡失败，原日志/截图保留，已改为严格区分完整卡与真实已落定准备门，空页/半渲染仍失败。e993的旧metadata子进程fixture曾4项失败，源码/时限/默认线程未改；最终804两轮完整CI实际通过。不借重试循环、放宽断言或跳过失败宣称成功。

## 发布门、兼容与回退

当前决定：可制作v0.2.1双平台私有草稿。只有main适用检查、实际NSIS/MSI与Mac压缩包身份、原公钥对实际updater资产密码学验签、隔离启动/版本及源码回读完成后，才能公开并晋升stable；实际结果完成后追加。当前Latest仍为0.2.0，不提前宣称新版已投递。

版本不引入存储或账户迁移；回退材料继续保留0.2.0原标签、原制品及REL-11，不改写历史或轮换公钥。若候选门失败，保持私有草稿和原稳定通道，不用日常安装验证。既有测试分发边界不变：安装包没有正式OS代码签名/Apple公证，updater签名须独立真实验证。

## 实际私有制品及公开前决定

main CI39602802413的6个job全部success；Release39604319577同一fbd6e1d1源码的Mac/Windows job均success。v0.2.1已固定该源码，Release ID419574046仍draft/prerelease。独立下载全部实际资产，API大小/摘要相符，三种updater资产使用tag内原公钥minisign密码学验签通过；公钥解码SHA30190687026fdc7b7e223884315baa52f6d42c1c37d447b8631e9597c1ecdfee与v0.2.0、v0.1.7逐字节相同。

| 实际资产 | 字节数 | SHA256 |
| --- | ---: | --- |
| Mac updater app.tar.gz | 6454094 | 6e70082d4417d69edb662a45167d20f71de7cbca3084e94c8d4600a100fd04b0 |
| Windows MSI | 6705152 | 30bdb68a3ee0774a33d26438b4c8baaad4393d1fc51fb08c7a9e21713b3a2d1d |
| Windows NSIS | 4775982 | f1d3467818caec936cb99c063e0c9af8e70bdf125bdf8ce1adea3b3852a96e7f |
| Mac DMG | 6347504 | 81306a5fa2dd44da34e28ba116a5037f2301baa985038d3dbd0dc8b078933a51 |

Windows实际MSI/NSIS安装、0.2.1版本/源码与内嵌包标记、原生模型交互、默认卸载保留及隔离清理边界通过；安装器摘要与上表已验签资产一致。Built EXE20457e30…与各包标记后的installed摘要分别核对，不将合法标记差异误认字节相同；生产者releaseAccepted=false仍保留，发布决定不由该脚本拥有。mode=any不冒充普通/管理员或人工升级/DPI/OTA验收。

Mac DMG容器校验通过，DMG与updater中Mach-O均为9885d99123d176bb270b9e3ce48072d2bce20b35cd1b024e18787e8a1d328233，版本0.2.1、production标识正确。独立root中仅改Info.plist隔离身份，逐字节相同发布二进制PID96813实际启动；关于页读得v0.2.1、macOS/Tauri v2，启动日志commit fbd6e1d1；Command-Q后PID不存在。日常PID19133继续运行，没有替换其安装/配置。原包strict codesign exit1按既有未正式签名/未公证分发边界保留，不混同已通过的updater签名。

公开前证据归.adg/evidence/REL-021-20261021，原始日志无损压缩，未将忽略项计通过。当前判断：修复、源检查与实际制品门均具备，可按用户既有授权执行stable公开；尚未把待执行的公开结果写成完成。
