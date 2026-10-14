# 51a 双 CI 完整终态

2026-10-14。此前进度“51a进行中”由本终态补充，原快照不改。

| 关联源码 / run | 事件及实际Windowscheckout | UI | RustMac / Windows | 终态 |
|---|---|---|---|---|
| 51a35eff / 38535387208 | push / 51a35eff | 99文件533通过 | 各35组，900/627通过，忽略6/7 | success |
| 51a35eff / 38535398225 | PR4 / fe4fb178d7ea70e376a60321624bd538795e61d7合成 | 99文件533通过 | 各35组，900/627通过，忽略6/7 | success |
| f545cb33 / 38533772616 | push / f545cb33 | 98文件511通过 | 各35组，900/627通过，忽略6/7 | success |

各run REST/原完整日志ZIP/CRC/安全条目、checkout与前后端实际数量交叉核验；ignore和零测试组不算通过。所有五必需job完整成功，main-only build条件跳过，Windowsjob内release build/final EXE smoke成功。独立head与PR合成检出分别记录，不把PR二进制当head制品。本轮未取新安装器/EXE，也未dispatch候选。

f545 PR38533779767真实Windows三项版本夹具失败仍保留，见[失败原件](../20261014-Windows向导路径与父提交CI失败/README.md)；这里父push/新51a成功不能解释或撤销它。独立control与parent未及时进入JS的问题尚未因果定位。bd2f6eb1只加测试自有句柄观察，push38537161445/PR38537166560当前仍运行Windows回归、Mac/前端/发布工具/tokenjob已完成成功；最新明确job ID/步骤原件记录，不当完整终态或WindowsAPI已验。继续读取同run，不重派/放宽截止、断言或并发。

产品源码51a、诊断headbd2、已打包候选aae/run38524086177三个身份保持分开。oldaae不含向导真实阶段和Windows路径UI补丁，新51a仅CI通过而未设备/候选验收。PR4保持draft/open/unmerged/main仍eae，Mac只后台，Windows现有唯一实例未动；原macOSCPU/RSS失败与所有预算/八门禁/FZ50许可/真实数据和日常安装保护保持。

原TASK3878字节/SHA bc7965b4…/revision2/in_progress/pending、空验收不改，RCPoverallfailed/goalactive；无新候选、真实迁移、核心事实晋升、自动合并、签名、发布或关闭证书。Mac原生先协调Windows实例0。
