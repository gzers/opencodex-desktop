# 投影观察优化后的原包双轮诊断 r1

revision: 1；2026-11-13；main236ed43dd；IMP-34 P24-04；accepted=false，整轮失败。

本轮是继[投影双遍改动](client-history-projection-r1.md)后一个新假设对应的原官方2.99 release-library诊断。前轮12.88秒失败保留，新轮全部使用冻结clean source与相同预算，未改官方包。

| 关卡 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 8202ms，通过 | 14368ms，通过 |
| 同连接验签Status5 | 227ms，通过 | 1453ms，通过 |
| 完整normalStop10 | 8749ms，通过 | 10736ms，失败 |
| 停止后状态 | 1375ms，NotRequired/可启动/Restored | 1674ms，Required/不可启动/待恢复 |
| 独立Head | RootAbsent9、完成读回 | RestorePrepared16，无完成摘要 |
| 原launcher/service与监听 | normalexit0、4listeners absent | normalexit0、4listeners absent；不替代Stop通过 |

本次第二Stop在root-artifacts-held处client_session_unconfirmed，动作层client_original_restore_pending；尚未建root movement。原deadline在该内部错误处是否已耗尽仍需精确诊断，不能只凭错误名认定根因。第二Start较前20700ms降至14368，Stop较前12878降至10736，但原10秒门不放宽。

原binding/first9events/旧archive内容与身份保护保持；两代原件/scope/provenance和Ready/PreStop/Stopped链、首代Plan/readback由归档raw bytes核验，第二代未伪造RootAbsent。全局L保持原identity/protection/bytes且exclusive释放；三个准确新协调对象先存原字节后独占清理，所有trackedprocessgenerations和导出ports无残留。证据及独立核验在.adg/evidence/0.2.24/client-history-projected-native-r1/。

下一步针对现有Stop入口load/preStop和原件admission重复历史观察的精确阶段测量，区分deadline与内容/权限/Guard拒绝后再选修法。保持完整当前root/provenance/Plan/帧/字节/Protection/path/OSproof及原Stop10，不重跑本轮或强杀补通过，不采用跨动作cache。

固定app/Windows候选、根中断/缺Ready/其它完整IMP34范围尚未完成，未发布。
