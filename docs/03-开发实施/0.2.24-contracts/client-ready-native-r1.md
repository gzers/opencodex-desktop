# Ready取得优化后的原包原因诊断 r1

revision: 1；2026-11-13；main4163514fe；IMP-34 P24-04；accepted=false，整轮失败。

本轮是[Ready取得改动](client-ready-acquisition-r1.md)后的唯一新原官方2.99双轮release-library诊断；未改官方包、原30/5/10预算、归属/原件判定或旧失败证据。

| 关卡 | 首轮 | 第二轮 |
|---|---|---|
| Start30 | 8839ms，通过 | 14110ms，通过 |
| 同连接验签Status5 | 226ms，通过 | 1528ms，通过 |
| 完整normalStop10 | 8608ms，通过 | 10729ms，失败 |
| 停止后status/preview | 1381ms，可启动/Restored | Required、不可启动/待恢复 |
| 独立Head | RootAbsent9 | RestorePrepared16，无完成摘要 |

本轮已确认原deadline耗尽：第二loadReady636537us、preStop2092753us；进入artifactadmission余908879us，guardimage/planopen/初authority均成功；applyretaining开始余696169us、耗735432us，最后authority.live开始余1633us、耗40838us，结束remaining0。底层原pair/authority返回lifecycle_timeout，经原IO映射成为会话未确认，动作层再报告client_original_restore_pending。第二cycle无writer-refusal trace。不是通过错误名或总时长猜原因，亦不据此重判旧无trace轮。

两轮服务/launcher真实normalexit0、四个listeners均消失，仍不能替代完整normalStop成功。旧binding/first9events/完整archive保持；两代provenance/Original/scope/三phase摘要链、首代Plan/readback及第二未完成状态由raw75文件重验。artifact恢复可能已有分步归档，不能将RestorePrepared当作零artifact变更。

证据.adg/evidence/0.2.24/client-ready-native-r1/绑定clean tree/18源/实际releasebinary和全部原始日志、独立trace/failure核验。全局L身份/保护/字节不变且exclusive释放；准确3新协调文件先保存字节后核身份/独占清理，ownedPIDbirthgenerations/端口无残留。测试目录归档核验后清理，工程target保留。

下一步测量每帧来源连续文件读取的parent路径链重复检查，尝试仅本次调用内前后完整校验与逐文件完整认证的封闭batch；所有每帧调用、内容/来源/权限/path检查、原deadline保持。反例和计量通过前不开下一native轮，不调整渠道配置、不放宽判断或cache跨动作。其它IMP34全范围和Windows最终候选条件保持；未发布。
