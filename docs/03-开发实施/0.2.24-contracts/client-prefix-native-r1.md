# 累计子目录与索引prefix优化的原包双轮 r1

revision: 1；2026-11-13；main0f0e85fb3；IMP-34 P24-04；accepted=false，整轮失败。

| 关卡 | 第一轮 | 第二轮 |
|---|---|---|
| Start30 | 8305ms，通过 | 13465ms，通过 |
| 同连接验签Status5 | 236ms，通过 | 1220ms，通过 |
| 完整normalStop10 | 8285ms，通过 | 12143ms，失败 |
| 独立状态 | RootAbsent9及读回 | RootRetirementPrepared17，无完成摘要 |

第二轮原artifactrestore在预算内完成：apply1600561us起始余量、耗1081034us，后admission/finaltime完成仍余453634us。随后rootplan/index/registration完成耗尽剩余预算，root-retire入口time检查remaining0/timeout。rootretirePlan已独立登记并有SHA/source16/schema2，但根未移、无Retire-readback/RootAbsent；不能将比旧轮更晚的失败位置当成成功。

两代原件/scope/provenance/三个phase完整摘要链和首代Plan/readback、第二退役Plan从raw77文件重验，firstbinding/raw9events/旧archive保持。两轮launcher/service实际normalexit0及每轮4listeners消失，原Stop失败判定不变。三个准确新协调文件留原字节后核firstseenidentity/exclusive清理，globalL身份/Protection/bytes保持且释放；ownedprocessbirthgenerations及ports无残留。source/tree18hash/实际releasebinary与失败/trace/清理在.adg/evidence/0.2.24/client-prefix-native-r1/。

下一步针对root-retire计划登记和registration的连续重复观察做计量，保持beforeSQL/afterSQL/COMMITreadback及当前native/source/root/Plan/frame/保护/path完整核验，封闭构造不输出跨动作证明。新反例/假设前不重跑本轮，不调整原预算/官方2.99/日常配置。

同时只读盘点确认root显式中断恢复及成功Stopaction的canStart容量一致性等仍待实现；其它IMP34/Windows/NAME/网络/候选范围不缩减，未发布。
