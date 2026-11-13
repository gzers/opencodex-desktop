# 官方双轮根生命周期定向诊断 r1

revision: 1；2026-11-13；main 2e74359b4；IMP-34 P24-04；accepted=false。

本轮是继[后继启动接线](client-root-start-r1.md)后的唯一实际原官方2.99双轮release-library诊断。测试装配及证据详 IMP-34 同日小节和 .adg/evidence/0.2.24/client-root-two-cycles-r1/；不覆盖固定应用/安装器、旧数据迁移或整版验收。

| 关卡 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 8199ms，通过 | 20700ms，通过 |
| 同连接验签Status5 | 224ms，通过 | 2205ms，通过 |
| 完整正常Stop10 | 8782ms，通过 | 12878ms，超时，失败 |
| 停止后实际status/preview | 1880ms；NotRequired/canStart/Restored | 2939ms；Required/无canStart，保留待恢复 |
| 独立会话状态 | RootAbsent9及完整readback | RestorePrepared16，无completion |
| 原始launcher/service | 正常退出0 | 正常退出0，但不替代Stop成功 |
| 前代binding/events/归档 | 已固定 | 原字节/身份/权限保持不变 |

两代独立provenance/Original/scope/Ready/PreStop/Stopped摘要链均从归档raw bytes验证。失败没有被转为通过；第二代根与原件按待恢复场景留证后仅清理自有测试资源。existing L身份/保护/字节保持且正常取回，三个精确新协调对象先留原字节/fsync回执再独占清理，两个导出primary端口内核确认无监听。所有旧失败保留，不跑同轮重试或强杀补通过。

下一步针对Stop中的当前Index/历史归档审查成本计量：两次独立登记在第二轮各增加约1.6–1.9秒；尚未证明哪个具体frame/系统调用占主导。先测真实已定位检查点，再修重复观察，保留完整来源/内容/权限/path/原deadline，不能cache跨边界或跳过历史。其它IMP-34完整未完及Windows固定候选条件不变。
