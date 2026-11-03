---
id: REL-OPENCODEX-DESKTOP-23
object_kind: release
state: planned
title: 0.2.20 原型一致性与默认字体修复发布
summary: bda最终包已完成签名、四安装和两平台实际OTA，目前公开prerelease；入口/字号最终证据补正后晋升stable。
---

# 0.2.20 原型一致性与默认字体修复发布

用户已授权从治理到stable连续推进，范围为DMD-OPENCODEX-UI-020、IMP-OPENCODEX-DESKTOP-28及原0.2.20-bugs的S20-01～20。采用cfbd90c4b+1da4cabca叠加用户确认的共同默认字体增量；首次成功云端目录全选、之后新增不自动选，未来多Agent/对话/连接图模块保持原包边界。

产品源码bda7c92da4e1f2e02e6bc2c641e0747748b98dd5、tree1b72afcd13c1b02597ec3b321ccd953e15189eb6，PR64合并f1b52e97557b0959d34898f23354b5afd0c255a1。tag v0.2.20与release432929365保持同源同八资产；当前draft=false/prerelease=true，公开Latest仍v0.2.19。beta端点只含完全相同的latest.json，指向正式v0.2.20原包。未重打已通过的Windows包或改更新公钥。

完整CI41671607927全过，冻结bda额外官方合同41671597006 attempt2两项全过；test-only诊断通过和此前原始失败均留存，不假定暂时provider_write拒绝根因。构建41671938225 attempt2双平台通过，Mac首轮未分配ARM runner的容量取消单独保留，不计通过。八资产摘要/长度、三更新签名/sidecar、五平台manifest、Mac deep/strict签名/嵌入源码与许可资源、DMG完整性及八件匿名下载均已验证。

Windows41675807159独立MSI/NSIS×默认/自定义四格真实安装/首屏/卸载/数据保持通过；Windows41683704589的MSI/NSIS真实相邻更新分别完成签名下载、安装、自动重启与原生/文件读回。原始Windows EXE SHA f864de162ba28b4f2c386fce4bf06d3b959bfc4475c959b7c557a8c0d5b84158，允许且仅允许唯一包类型标记MSI/NSS变化后的整字节核对。

Mac实际相邻OTA通过beta安装，原始EXE SHA823a62869ca494acf83ccc955c04ae80d3fb6cd5b83b084b9983701a4ba17dbd与上载包一致。自动重启新进程已观察；本机AppKit旧crash-history提示阻住首轮自动挂载，随后同字节读回副本以进程级ApplePersistenceIgnoreState YES正常挂载，43个基线文件、旧版两份前像、100%缩放与原provider保持。/tmp软链启动拒绝、规范路径纠正及提示sample均保留，不能简写成无条件自动挂载成功。正式包隔离导出/目录/通知结果与补充本机HTTPS连接保存/失败/留空密码/候选启用另有实屏。

当前剩余为独立全入口与最终字号映射补正，再审阅后晋升同一tag/bytes到stable。本记录未宣称已发布stable。证据根.adg/evidence/0.2.20/font-final-candidate-bda7c92d、final-entry-reconciliation-20261104及原问题清单；正式TASK保持in_progress，最终六项检查与20项AC要形成RuntimeStore RCP/CERT。

兼容与回退：交付平台macOS Apple Silicon、Windows x64。保留v0.2.19原包及签名；必要时手动回退并核对本机备份，不移动已公开tag、不承诺自动降级。安装包沿用既有macOS ad-hoc与Windows无Authenticode方式；更新载荷另有真实密码学签名。连接密码沿用现有本机文件后端（Mac权限保护，Windows DPAPI），内容保护口令才使用独立原生安全存储。OS底层权限失败可能显示系统详细英文原因。没有本轮新增的真实多设备/日常WebDAV适配承诺；用户日常配置、凭据与安装均未修改。
