---
id: REL-OPENCODEX-DESKTOP-21
object_kind: release
state: verified
title: 0.2.18 Windows修复与概览原型还原发布
summary: 已公开stable；W18-01～13及U18-01～05修复，正式原包、四安装组合、三签名、匿名资产和两种实际OTA通过。最终main六项CI全部通过，历史npm根因未知。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/29
  - https://github.com/gzers/opencodex-desktop/pull/30
  - https://github.com/gzers/opencodex-desktop/pull/31
  - 2e709d09590493f73c9e4f31883f67f949c8e90c
---

# 0.2.18 Windows修复与概览原型还原发布

2026-10-29，Asia/Shanghai。[v0.2.18已公开并投递stable/Latest](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.18)，公开时间10:36:13Z（北京时间18:36:13）。用户授权文档、开发、测试至发布；范围沿[0.2.18单一清单](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.18-bugs.md)，执行由[IMP-27](IMP-OPENCODEX-DESKTOP-27.md)承接。

## 交付与源码身份

W18-01～13保留并验证Windows可读路径、受控环境、官方shim/卸载目标、COMSPEC、npm view诊断及失败输出；U18-01～05按冻结8136cf707的0.2.0总原型还原概览更新确认/进度、公共导入、公共安装和环境六态。导入720px上限、独立标签/带框选择区，安装来源/代理分组和离线清除动作补齐。现有模型文件保存、通知入口、路径安全及安装失败补偿保持，未采用并行的全景概览/Logo/对话策略原型。

正式tag/main源码为2e709d09590493f73c9e4f31883f67f949c8e90c。最终PR引用14ca2c0e和21bfd436候选的应用树均为1c9a4a3d82c36a07afe510f381c06b6ba56ad70b。PR31合并main b7cd66343只修改cfg(test)握手夹具；生产模块前缀逐字节一致，原tag/签名包未重建或移动。正式Windows原始构建EXE SHA256为220b24a7825798fca445136b9a672f1a512602aa6dafea9c6122c7008673e1fb，安装后的MSI/NSIS bundle marker按真实格式分别校验。

## 验收回执

| 阶段 | 实际结果 |
| --- | --- |
| 原型与本地 | 双引擎亮暗/高低三档/宽窄配对各241项通过；完整前端1044通过、10既有跳过，21发布门全通过，实际Rust文件链46项通过 |
| 最终候选CI | [40839913190](https://github.com/gzers/opencodex-desktop/actions/runs/40839913190)五门全部通过；原生Node24.22.1及22.20.0/npm10.9.3安装2.91、失败保留、恢复2.93、查询恢复、路径和真实HTML File清除/重拖均通过 |
| 普通账户 | 实际临时标准SID、原生Profile、elevated=false、WebView2及模型保存/Key/模版/传输检查通过，桌面DACL恢复 |
| 退出边界 | 原源码Windows定向30次通过；活动列表遗漏将退出进程后，NEW_PROCESS通知保留经成员校验句柄，原2秒收尾/查询期限与真实signaled断言保持 |
| 夹具修正 | [40848017469](https://github.com/gzers/opencodex-desktop/actions/runs/40848017469)握手原测试30次通过，[40848030517](https://github.com/gzers/opencodex-desktop/actions/runs/40848030517)五门全部通过；只修改cfg(test)，原main前置失败保留 |
| 正式构建 | [40844799061](https://github.com/gzers/opencodex-desktop/actions/runs/40844799061)生成八资产；macOS签名资源成功，Windows串行all安装测试遇到先前记忆目录，原失败保留 |
| 原包四安装 | [40847331072](https://github.com/gzers/opencodex-desktop/actions/runs/40847331072)四个独立首次runner真正MSI/NSIS×default/custom全通过，原资产digest和原始EXE严格匹配，未重建 |
| macOS原包 | 正式归档大小/SHA、codesign --verify --deep --strict、资源封装及两份许可通过；原包WKWebView隔离启动/安装弹窗/取消实查，日常配置摘要不变 |
| stable公开 | [40858820250](https://github.com/gzers/opencodex-desktop/actions/runs/40858820250)成功；非draft/非prerelease，Latest/stable=0.2.18 |
| 匿名与签名 | 八资产匿名HTTP200、大小/SHA匹配，版本及stable清单一致；沿0.2.17原公钥三包minisign独立通过 |
| 实际OTA | [40859489907](https://github.com/gzers/opencodex-desktop/actions/runs/40859489907)MSI/NSIS均通过：公开0.2.17→0.2.18，经生产stable下载验签；普通退出仍旧版、明确重启真实替换、OS观察自动新版进程；同一隔离配置/合成凭据原字节保持、pendingRestart清除 |
| 最终main持续复验 | [40858818265](https://github.com/gzers/opencodex-desktop/actions/runs/40858818265)六job全部通过；macOS1300/14忽略，Windows1023/16忽略，嵌套自进程重复统计排除1项，真实两Node/普通账户/最终EXE和macOS集成构建通过 |

原始证据与分层范围归[证据清单](../../.adg/evidence/0.2.18/overview-parity-20261028/manifest.json)、[RCP](../../.adg/evidence/0.2.18/overview-parity-20261028/RCP.json)、[原包核验](../../.adg/evidence/0.2.18/overview-parity-20261028/formal-release/verification.json)、[匿名资产](../../.adg/evidence/0.2.18/overview-parity-20261028/formal-release/anonymous-public.json)与[实际OTA](../../.adg/evidence/0.2.18/overview-parity-20261028/formal-release/ota-summary.json)。正式发布、最终main持续复验和逐项证据审计均完成；原失败与未执行环境保留。本轮按支持范围关闭。

## 正式资产

| 名称 | 字节 | SHA-256 |
| --- | --- | --- |
| latest.json | 5899 | a5061ff3b580cbd63fbb2424bc806bd37313f211a34c3bf1b770d670c5fb30aa |
| OpenCodeX.Desktop_0.2.18_aarch64.dmg | 6536037 | db3516cc9169398500b34f0d239d184c1bf664073fc63d0fcb5202beeb2f4055 |
| OpenCodeX.Desktop_0.2.18_x64-setup.exe | 4911057 | 4b70c63964353ce2b66ef54df45c8d728a1442a18f5bfa8a222df8d288652301 |
| OpenCodeX.Desktop_0.2.18_x64-setup.exe.sig | 452 | a12f8a24e0cae97f4c52729d6948d79b675614c7d42cf8d42b802b60365f1ea6 |
| OpenCodeX.Desktop_0.2.18_x64_en-US.msi | 6877184 | 3b581e1b5eaadf616f2d56757cbdf1a2efd53dd58bbd704b63d642c7af60ab8d |
| OpenCodeX.Desktop_0.2.18_x64_en-US.msi.sig | 452 | 76fc4277aa15f80c25ef8e84db6382a41baa5c367a135b8b51db2a35667cdb98 |
| OpenCodeX.Desktop_aarch64.app.tar.gz | 6643717 | 3f33539c9874db710e59fb5ca8c7c67531e8b657d8c1eaadec8054ca17e07585 |
| OpenCodeX.Desktop_aarch64.app.tar.gz.sig | 440 | 617d3b74bbd88161997f8fa4539be3f0657b91695e232232f13e1f83d36e2434 |

## 失败、范围与恢复

原217项配对遗漏导入宽度/字段分组，新增断言在旧实现稳定12项失败，补齐后241项通过；不能把旧绿灯扩大成布局一致。实际拖入曾被ResizeObserver故障遮罩阻断，随后暴露postMessage字节回退；失败原文、修正和最终原生回执均保留。安装目录Access denied的具体持有者未证实，不归因为杀毒或权限；只在持锁、清单身份一致的受管临时包激活边界对Windows5/32/33有限等待10秒，新目标/清单变化拒绝。

main第一次退出测试失败是尚未取得helper handle的前置竞态，握手先持有并确认存活再允许root退出；不是降低期限或删除真实退出断言。正式Windows流水线混合默认/自定义安装时保留前次记忆目录，四独立runner使用原包通过；原失败不得改记为成功。

实际OTA的OS自动重开观察与后续IPC文件读回分开：安装器Shell自动进程不继承测试CDP环境，观察自动PID/版本后，仅停止本次受控进程并用相同不可变沙箱显式重开更新EXE读回。未假称自动进程直接提供CDP。

历史两次npm退出1缺原始输出，根因仍未知。物理Win10/11、企业PAC/证书/重定向、真实SMB、全部语言/混合DPI和完整设备权限矩阵未执行；本轮临时Actions账户和沙箱不代表全设备覆盖。无OS代码签名/Apple公证声明，更新包签名独立验收。不清空日常配置、不替换日常安装、不重启用户活跃Codex、不进行付费生成。

保留上一正式[v0.2.17](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.17)原始资产与签名。必要回退先保留现配置和备份，stable回退走显式通道流程，不移动公开tag；安装补偿维持原失败恢复边界，不承诺突然断电的多文件原子性。

## 发布后反馈登记（2026-10-30）

用户报告双向同步缺口、Windows已启动但内嵌面板打不开且误报启动失败、同步/WebDAV流程过于复杂、Skill应直接覆盖更新及官方openai重复阻断保存五组问题，统一登记为[0.2.18清单S18-01～05](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.18-bugs.md)。此前观察到的旧未配置错误、MCP同名多源与ETag提示/最终发布结果不一致并入S18-03；Windows与重复openai根因仍待取证。

原发布/签名/实际验收回执保留，但不能据此把这些发布后反馈记作已解决。本轮只记录，不改软件或原型、不清理日常配置、不启动下一轮实施或发布；修复版本待后续规划。
