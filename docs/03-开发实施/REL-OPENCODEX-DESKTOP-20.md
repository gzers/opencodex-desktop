---
id: REL-OPENCODEX-DESKTOP-20
object_kind: release
state: in_progress
title: 0.2.17 Windows托管安装可靠性发布
summary: 发布准备中；本地完整门禁、Windows后端及真包安装证据已具备，普通账户及正式发布仍待最终回执。未打tag或公开版本。
source_refs:
  - https://github.com/gzers/opencodex-desktop/pull/26
  - docs/03-开发实施/IMP-OPENCODEX-DESKTOP-26.md
---

# 0.2.17 Windows托管安装可靠性发布

2026-10-28，Asia/Shanghai。用户已授权WP89～WP95文档治理、开发、测试与stable发布。当前状态仅发布准备，未公开0.2.17。实施归[IMP26](IMP-OPENCODEX-DESKTOP-26.md)、验收归DMD AC-WIN-017-01～06，源清单为[0.2.16问题记录](../04-项目资料/01-项目支撑资料/06-问题记录/0.2.16-bugs.md)。

## 当前范围

Windows扩展路径与默认安装位置、受管目录归属及无副作用准入、包体/入口/来源记录失败补偿、真实有界版本探针、受控版本管理器发现、随包许可和仍禁用的CLI说明。补修真实安装链路发现的脚本授权重试与本地离线tgz参数，不引入新UI、保存核对或通知表面。

本地后端1289、前端988通过，保留14既有忽略/10既有跳过；21项发布门禁全部通过。Windows真包2.90→2.91联网升级/热缓存离线重装及真实Vue向导完成安装已在隔离runner读回，具体身份见[证据矩阵](../../.adg/evidence/0.2.17/acceptance-matrix.json)。最终CI和普通账户、正式安装器仍待验，不能引用先前构建冒充最后提交通过。

## 尚待完成的发布回执

PR26最后提交双平台CI；真实普通账户最终EXE；正式macOS/Windows构建；默认/自定义MSI/NSIS及许可资源/数据保留；同一tag三个更新包既有公钥验签；preserve-latest公开和stable投递；匿名8资产摘要/manifest；实际旧版OTA替换/重开/配置凭据保留；清单、RCP与完成审计。缺一不标发布闭环。

## 兼容与回退

无需清空用户配置，不替换日常软件、不强制重启活跃Codex。托管安装非受管非空目录拒绝，失败保护旧包/入口/来源；进程内补偿不宣称突然断电跨文件原子。失败回滚材料保留，不移动已发布历史tag。stable回退沿显式发布流程执行，保留当前配置及备份。

安装包继续未做OS代码签名/Apple公证的测试分发，自更新签名验证独立执行。普通账户验收限定临时Actions账户，其Profile由Windows服务创建；镜像Default User中LocalAppData错误指向runneradmin时，只对本次新建SID初始化自己的标准Local目录，并保留报告。不是用户日常账户操作，也不证明未调整镜像的所有账户组合。

Win10/11物理设备、混合DPI、真实SMB/重定向企业账户、PAC/企业证书及私有服务保留未执行边界；不以单个Server runner结果宣布全域通过。
