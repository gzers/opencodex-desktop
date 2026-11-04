---
id: "REL-OPENCODEX-DESKTOP-23"
object_kind: "release"
state: "verified"
title: "0.2.20 原型一致性与默认字体修复发布"
summary: "v0.2.20已公开stable/Latest；八资产、三更新签名、独立四安装和Mac/MSI/NSIS相邻实际更新及最终字体/入口审阅完成。"
---

# 0.2.20 原型一致性与默认字体修复发布

用户已授权治理、开发、测试和stable连续推进。范围为DMD-OPENCODEX-UI-020、IMP-OPENCODEX-DESKTOP-28及原0.2.20-bugs的S20-01～20。采用cfbd90c4b、MCP增量1da4cabca及用户确认的默认字体调整：正文/导航13px、控件12px、副标题15px、弹窗18px、页标题22px，小字与所选缩放语义保留。首次成功云端目录全选，之后新增不自动勾选。

[v0.2.20已公开stable/Latest](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.20)。冻结产品源码b1750f5cf78b294b2edb34b9674d8c707ef8f810，PR67合并28d7f62e994beed57d291c904ab2c775491bc928；Release ID433103683、八资产ID/字节/摘要均与已验新候选相同。原PR65/66由PR67整合覆盖并关闭；最终tag/source b175包含所需产品与QA提交。旧bda候选因UpdateCenter共享缩放失败撤回为私有validation/0.2.20-bda7c92d归档，原八资产/失败/原安装OTA证据保留；最终不采用其包体。

完整产品CI41699143981的最终结果与此前普通账户首次启动失败原件分开保留。该失败窗口有响应但沙箱日志未生成、托盘Access denied，未推定未经证明的根因；复验保持原阈值。最终直接源码/本机与双引擎检查、所有必需CI事实见final-ci-completed.json及final-promotion-gates.json。官方合同41671597006 attempt2两项通过，其Rust/协议实现与最终b175完全相同，明确复用未变模块，不把诊断日志当产品变化。

构建41699584553完成双平台原包/签名/上传，Mac job成功，Windows后置原生菜单点击检查超时的原始失败留存。相同原包的独立四安装验收通过后承接安装门，未重建已上传字节；不将原Release job写为绿色。八上载资产、三份tag公钥签名/sidecar、五平台manifest、匿名下载全过；Mac原封包deep/strict、嵌入源码与LICENSE/NOTICES、DMG完整性通过。Mac原始EXE SHA 544de246de1a498b8c681c0bd6ce254c7043ffe4b5fe754d67d3477a8fe4370b；Windows原始EXE SHA 7f6788eb996e9ff5ed3a2283650567b35ccabcf24bb747fd40cc1a2c3afaed77。Windows类型标记只允许唯一MSI/NSS变化，仍比较完整预期字节和原始构建身份。

独立41738559767的MSI/NSIS×默认/自定义四格真实安装/首屏/卸载/数据保持全过。Windows41739113409实际0.2.19→0.2.20两格完成签名下载、普通退出保旧、明确安装与自动新进程、原配置/合成凭据读回和pending清理；安装后真实WebView2更新中心100/150/200%按钮高34/51/68px，四边与关闭可达。自动OS新进程观察后，Windows原生安装器重启经Shell不继承测试变量，故使用相同已安装EXE显式隔离重开作文件/窗口读回；边界如实记录。此前探针误用尚未检查快捷入口、无运行环境的概览维护卡、普通按钮role选择器及一轮初始CDP超时均保留，最终修正后严格断言通过，不将失败改写为通过。

Mac实际相邻OTA使用原封0.2.19身份/签名和规范/private/tmp路径，进程级ApplePersistenceIgnoreState YES只处理本机已有AppKit旧恢复提示，不改系统设置。两次真实旧版UI保存及精确前像后，经既有beta端点下载签名安装，确认重启后同路径新进程自动挂载b175/0.2.20并继承隔离环境；直接在同一新进程读回原渠道，9份业务/偏好/新备份前像摘要保持、handoff消费、比例100%保留。最终轮没有二次readback副本，区别旧bda历史轮次。最终原封升级包在同一自动重启进程另实屏验证概览/设置两宿主200%放大、最新版当前唯一版本与原位Escape；整包逐文件等同上载签名载荷，日常安装未替换。

逐入口最终口径为37处Dialog/Modal组件引用、26处动态调用、3处另列UpdateCenter/既有AgentPanel宿主引用，不是66个唯一窗口。独立复核补齐模型助手、更新中心和最终字体在模型/扩展/次要窗口的布局；发现共享缩放漏项后先修复再重打新包和全部包体验收。历史恢复草稿已通过真实原生窗口/IPC将64000→128000放入草稿并返回模型页，已保存策略与业务config保持；记录目录为显式seed的已校验legacy消费者夹具，不冒称真实官方运行时保存。原型窄屏探针纠正的是browser viewport未改变固定产品window的测量方法，不改原型源或借未来模块补功能。

正式证据位于.adg/evidence/0.2.20/final-candidate-updates-20261104和final-entry-reconciliation-20261104。原生连接/导出/备份/范围恢复等此前bda层证据只有在测试模块逐字节相同条件下复用，包体安装、签名和OTA必须引用最终b175原包。20项AC及六项review由RuntimeStore的TASK/RCP/CERT关闭；文件范围、来源、绑定与人类事实回写均校验。

兼容与回退：交付macOS Apple Silicon和Windows x64；保留公开v0.2.19原包/签名与本机保护备份作为手动回退来源，不承诺自动降级、不移动稳定tag。沿用macOS ad-hoc与Windows无Authenticode方式，更新载荷另有真实密码学签名。WebDAV连接密码沿用既有本机文件后端（Mac0600/0700、Windows DPAPI），内容保护口令独立使用原生安全存储，不能混称全部系统钥匙串。底层文件权限错误可能显示系统详细英文原因。真实客户端MCP键路径、多Agent扩展/未来对话与连接图不在本版采用范围。没有读取/发布日常秘密、改变日常配置/安装或执行日常真实WebDAV/双设备同步。

公开stable于2026-11-04T00:33:52.788684+00:00核验，原tag与Latest匿名manifest及八资产身份保持。正式关闭TASK-OPENCODEX-UI020-CLOSEOUT-20261104 / RCP-OPENCODEX-UI020-CLOSEOUT-20261104 / CERT-OPENCODEX-UI020-CLOSEOUT-20261104。
