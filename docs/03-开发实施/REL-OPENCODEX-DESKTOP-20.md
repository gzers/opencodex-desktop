---
id: REL-OPENCODEX-DESKTOP-20
object_kind: release
state: in_progress
title: 0.2.17 Windows托管安装可靠性发布
summary: 发布准备中；本地完整门禁、Windows后端及真包安装证据已具备，普通账户及正式发布仍待最终回执。已打v0.2.17标签，正式构建中，未公开版本。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/26
  - docs/03-开发实施/IMP-OPENCODEX-DESKTOP-26.md
---

# 0.2.17 Windows托管安装可靠性发布

2026-10-28，Asia/Shanghai。用户已授权WP89～WP95文档治理、开发、测试与stable发布。当前正式构建中，未公开0.2.17。实施归[IMP26](IMP-OPENCODEX-DESKTOP-26.md)、验收归DMD AC-WIN-017-01～06，源清单为[0.2.16问题记录](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.16-bugs.md)。

## 当前范围

Windows扩展路径与默认安装位置、受管目录归属及无副作用准入、包体/入口/来源记录失败补偿、真实有界版本探针、受控版本管理器发现、随包许可和仍禁用的CLI说明。补修真实安装链路发现的脚本授权重试与本地离线tgz参数，不引入新UI、保存核对或通知表面。

本地后端1289、前端988通过，保留14既有忽略/10既有跳过；21项发布门禁全部通过。Windows真包2.90→2.91联网升级/热缓存离线重装及真实Vue向导完成安装已在隔离runner读回，具体身份见[证据矩阵](../../.adg/evidence/0.2.17/acceptance-matrix.json)。最终CI和普通账户、正式安装器仍待验，不能引用先前构建冒充最后提交通过。

## 尚待完成的发布回执

PR26最后提交双平台CI；真实普通账户最终EXE；正式macOS/Windows构建；默认/自定义MSI/NSIS及许可资源/数据保留；同一tag三个更新包既有公钥验签；preserve-latest公开和stable投递；匿名8资产摘要/manifest；实际旧版OTA替换/重开/配置凭据保留；清单、RCP与完成审计。缺一不标发布闭环。

## 兼容与回退

无需清空用户配置，不替换日常软件、不强制重启活跃Codex。托管安装非受管非空目录拒绝，失败保护旧包/入口/来源；进程内补偿不宣称突然断电跨文件原子。失败回滚材料保留，不移动已发布历史tag。stable回退沿显式发布流程执行，保留当前配置及备份。

安装包继续未做OS代码签名/Apple公证的测试分发，自更新签名验证独立执行。普通账户验收限定临时Actions账户，其Profile由Windows服务创建；镜像Default User中LocalAppData错误指向runneradmin时，只对本次新建SID的用户目录注册项初始化自己的标准Local目录，再用新进程读回，并保留报告。临时SID的GUI测试还临时授予窗口站/桌面访问并在finally恢复原DACL，记录实际恢复字段；软件进程仍须普通令牌。不是用户日常账户操作，也不证明未调整镜像的所有账户组合。

Win10/11物理设备、混合DPI、真实SMB/重定向企业账户、PAC/企业证书及私有服务保留未执行边界；不以单个Server runner结果宣布全域通过。

## CI及合并已验回执

CI40632048037五个必需job全部通过，PR26已合并为d7c7978f043278baae8f2fc8c2119315a4660f12，与验收head f77327aa8的源码树完全一致，v0.2.17标签指向此合并。

原生回执实际读回：ordinary-summary result=pass、applicationElevated=false、applicationModeMatched=true、desktopAclRestoredOnExit=true。真实Vue安装官方2.91.0、默认prefix、来源记录通过；官方2.90→2.91升级和热缓存离线重装通过。窗口/表格/排序/双渠道保存/Key/导入/模版的原生证据已归档windows-native，不用绿灯代替实际回执。

正式构建、安装器和公开/stable、OTA仍待完成。

## 正式包首轮与验收脚本复验

Release40636062852的macOS构建成功；Windows MSI/NSIS及签名资产成功生成，但验收脚本在NSIS注册表InstallLocation带引号时Join-Path读成错误盘符，故整体job失败。MSI默认安装/真实界面已通过；原失败报告和日志保留，不把整轮误记成功。

PR27仅修测试脚本和验收工作流：固定摘要下载既有未公开资产，MSI管理提取后只恢复Tauri标记并匹配首次构建EXE摘要610fb4cc…f5365776；安装包本体、签名及v0.2.17标签完全不变。验收使用ExpectedCommit=d7c7978f0，避免拿测试脚本提交冒充应用来源。Windows默认/自定义安装器复验继续执行。草稿三个更新包已独立使用原公钥验签及摘要匹配，macOS归档版本/标识/两份许可资源读回通过，仍未公开或晋升stable。

## 正式安装器、公开与stable已验

首次复验曾因脚本种类变量被目录循环覆盖而四项均运行NSIS，绿灯未直接接受；逐项回执审查捕获并保留限制，155bb6048修正后CI40647352613真正MSI/NSIS×default/custom四个独立首次安装runner均通过。报告逐项读回实际kind、源提交、原包摘要、首屏、默认/自定义路径与自定义默认卸载数据保留。原签名资产未重建，v0.2.17不移动。PR27测试接线已合并为a354b130b，后续仅验收脚本维护不改变发布应用。

公开40647289835先保留Latest=v0.2.16，stable40647642314成功投递Latest=v0.2.17。匿名release非草稿/非预发布，latest manifest version=0.2.17，8资产HTTP200/大小/SHA256匹配；三个更新包使用原项目公钥独立验签通过。证据release-summary.json与installers-final-summary.json已读回。实际旧版OTA替换/重开仍待后验收，不据签名通过声称该项完成。

实际OTA第一次下载/验签及pending_restart后端状态成功，但脚本直接读取Tauri Channel raw消息，未拆message封装导致进度断言失败；dbc367806仅修验收脚本，生产IPC不替换。CI40648079681重测普通退出保留旧版和明确重启替换。
