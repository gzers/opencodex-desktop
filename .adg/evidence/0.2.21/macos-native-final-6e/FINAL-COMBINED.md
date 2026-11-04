# 原封6e macOS native 最终组合结论

结果：通过，采用同EXE已完成all各项 + 最终notice专项一次通过；不声称单轮all全绿。

| 项目 | 证据与结果 |
| --- | --- |
| 原包身份 | 6e2197fd0b8a36dcb8d893b62693c5a2a77151cc；EXE275dface26a0d1ab483a67303eb30df3253f1f050163548c51289f1812b8457e |
| 其它all项、ready/WebDAV/view/edit/detection/真实保存读回 | [de8原all报告](ready/ocx021-mac-native-de8d576b-26d2-4c19-ba9f-badfcb8a2231/report.json)中对应项均pass |
| 自然8秒 | de8原包8.266447791秒，原阈值6.7～9.3 |
| 最终notice专项 | [335原报告](notice-final/ocx021-mac-native-335766ec-379e-4a21-a2e0-924cfece5d14/report.json)，仅运行一次，退出0 |
| X/Esc/取消 | 实际退出均匹配请求，无X恢复/无重试/无确认重启 |
| 活跃时长完整保守区间 | 8.310690～8.597482秒，整个区间在原6.7～9.3内 |
| 同通知恢复/最终收起 | pass；无手blur/重置、无虚时钟 |
| 原包seal | deep strict通过，未修改/重签 |
| 进程 | owned8050/9732/15515/18241组均空，daily80916仍在；UI已释放 |

[最终结构化组合摘要](final-combined-summary.json) · [最终精确观察trace](notice-final/ocx021-mac-native-335766ec-379e-4a21-a2e0-924cfece5d14/notice-trace-True.json) · [自然计时/旧估算边界诊断](timer-diagnostic/README.md) · [原输入1/2 vs held50ms2/2有界AB](esc-input-ab/README.md)

原失败全部保留：de8旧活动估算5.921、f9全树读取宽区间blocked、09原瞬时Esc失败，以及AB的A2失败/X恢复。它们没有被改写成通过；最终专项使用经AB验证的真实50ms按键默认方式，仅automation改变，不称产品间歇bug修复。

当前native.py/.swift/.md源码稳定，哈希在组合摘要，可直接交父级提交；本轮没有再次修改源码、没有提交/推送。OTA仍等待父级真实beta publication receipt。
