---
id: REL-OPENCODEX-DESKTOP-10
object_kind: release.candidate
state: verified
title: 0.1.10 运行维护发布与稳定通道投递
summary: 0.1.10已公开并晋升stable；最终main及双平台Release全绿，三种更新包旧公钥验签、匿名清单/下载及Mac隔离版本回读通过。本版例外及原TASK未验记录保留。
source_refs:
  - IMP-OPENCODEX-DESKTOP-20
  - TASK-OPENCODEX-DESKTOP-182
  - 8e69ff6db69ea13b8cb29feb9f7a1abacd202a7b
---

# 0.1.10 运行维护发布与稳定通道投递

2026-10-18，Asia/Shanghai。用户对最终发布确认明确回复：“批准公开并晋升 stable，接受清单中的本版例外”。授权固定产品源码8e69ff6d、Mac arm64/Windows x64、已审阅文档与版本说明；沿用OS未正式签名/未公证的测试分发策略，updater资产必须实际签名并验签。授权包含main整合、v0.1.10标签、既有双平台构建/签名、公开Release、stable投递及发布后验证，不包括FZ50脚本、替换日常安装或迁移真实数据。

## 本版明确例外

以[最终确认单](../04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略/分析/10-0.1.10最终发布确认单.md)为完整裁决范围：真实管理器/面板更新及相应终态事件未完整证明，备份/原生剩余组合、帧/缓存100/10/休眠和未证周期保留未验；损坏面板保护关联无受控重建入口，Windows CLI仍禁用。原失败与范围保留，CPU/RSS接受、DPI取消沿原裁决，不重测或扩建工具。

本次接受是0.1.10发布的人工风险处置，不改写旧测试passed，不提供原所有AC或完整设备矩阵通过证明。原TASK-182暂保持revision2/in_progress/pending；发布及发布后验证完成前不关闭任务/目标或生成完成证书。未验遗留作为明确例外/延期登记，不能在将来新增版本自动沿用。

## 源码与执行状态

产品8e的首次完整push及PR7集成CI已通过；PR检出48cff7dd，Mac938/Windows667/前端542。最终main只整合已验实现和已审阅发布说明/共享文档，不新增产品功能。最终main/tag SHA、Release构建与资产摘要、实际旧公钥验签、公开端点/下载及发布后验证在实际完成后补录，不预填成功。

发布及本轮发布后验证已完成，公开结果见下文；原准备阶段快照保留在证据中。操作系统签名与updater签名分开记录；签名、制品、端点或数据保留失败均阻止继续投递。回退沿保留0.1.9资产/配置保护，不自动降级用户已安装软件或宣称二进制回退可以恢复数据。

### main与标签

已验8e整合为main2578e11ba36fc133dda3a899b8085b94ad047044，父系保留eae与8e；相对8e只增加CHANGELOG/README中英文发布说明，apps tree严格相同。PR7已由GitHub记录merged；v0.1.10指向该main，main CI39097338564及Release39097612293均为首次运行且最终success。核心事实已按批准提案落地，原构建排队观察保留；公开结果以实际发布与独立验证为准。

### 草稿资产与旧信任验签

Release39097612293双平台均success，actual checkout同main2578e11b。草稿Release ID416235919，尚未公开；实际Mac更新包、MSI、NSIS的大小/API摘要及0.1.7原公钥minisign验签全部通过，当前与旧公钥解码SHA30190687…一致。清单包含darwin-aarch64/其app别名及Windows默认/MSI/NSIS平台。

| 更新资产 | 字节数 | SHA256 |
| --- | ---: | --- |
| OpenCodeX.Desktop_aarch64.app.tar.gz | 4401860 | 3553060a18da9d8e43972bb99e6b9ff7d2865c2c1ca515c15f1a497b8fa26f6f |
| OpenCodeX.Desktop_0.1.10_x64_en-US.msi | 4632576 | fa1be52304c4e2df49fb8e824cff25e6ef26b80b604f2a56e968ba1c83ea9d1b |
| OpenCodeX.Desktop_0.1.10_x64-setup.exe | 3310009 | a8ffb56c5b0eb65a08e9719ed1cf540360c4eb76173bbc63007ea184c80c70ed |

Mac DMG4383011bytes/SHA9413ea978ca1303a36f2bd19ca3a37cff1a7bdb55f18a6ef33bfd20aafbee261，容器校验通过；压缩包内0.1.10、生产标识与内嵌main SHA核同。strict codesign仍exit1，按已批准OS未正式签名/未公证策略保留，不写成Apple签名通过。Latest在草稿验签时仍0.1.9；main完整CI终态及公开/stable/匿名验证尚待，不由三资产通过提前宣告发布完成。

## 公开结果、稳定通道与发布后验证

[main CI39097338564](https://github.com/gzers/opencodex-desktop/actions/runs/39097338564)六个job全部success，实际checkout均2578e11b，包括Mac专属bundle、Windows最终EXE及35组后端667通过；Mac938、前端100文件542通过。双平台[Release39097612293](https://github.com/gzers/opencodex-desktop/actions/runs/39097612293)同SHA全部success，三种实际更新资产已独立旧公钥验签通过。

唯一[Publish Release39098939911](https://github.com/gzers/opencodex-desktop/actions/runs/39098939911)以stable模式成功。[v0.1.10](https://github.com/gzers/opencodex-desktop/releases/tag/v0.1.10) Release ID416235919，公开时间2026-10-18T04:59:09Z（北京时间12:59:09）；draft=false、prerelease=false，Latest由0.1.9晋升0.1.10。发布器在公开前验清单/平台/资产/密码学签名，公开后核固定与stable匿名端点；未重复派发或重跑。

集成线另以不带Authorization的HTTPS请求下载固定与stable latest.json，均HTTP200、5717bytes/SHA af24a230c0663801cb11c292a312b764f02b4040b5dca1fd16d6b111ebc5074f，内容与原已核清单完全一致。三个更新包及Mac DMG均匿名下载200，全部大小/摘要与公开前验签字节一致；不是只核资产API元数据或下载链接存在。

公开后使用该Mac更新包的逐字节相同Mach-O，测试副本只改Info.plist隔离身份，并显式独立数据根/边界。PID71872原生首屏、设置关于回读v0.1.10，实际启动日志commit2578e11b；Command-Q正常exit0，隔离实例0。日常PID57186及选定偏好/根配置摘要保持，不替换日常软件、不执行官方安装/FZ50、不宣称旧客户端OTA或Windows现场复验。该短观察不替代已接受未验的真实周期/帧/资源等场景。

完整证据在 .adg/evidence/REL-0110-20261018/，最终清单绑定授权、main/tag、CI/Release/发布原日志、三包验签、匿名下载及原生回读。版本/核心/共享说明与发布事实已回写；后续新增事件维护核心《统一事件与通知》，新增备份范围同步核心字段及消费者。运行观察见RUN-OPENCODEX-DESKTOP-20261018-01。

## 任务与遗留处置

本次发布目标按用户确认的版本范围完成。原TASK-182涵盖的完整AC未被假定通过，原字节和pending状态保持，八检查分别见accepted-exception-disposition.json；本版例外和未验遗留有明确人工裁决，不生成完整AC完成证书或自动带入后续版本。IMP-20保留实施/验收遗留入口，REL仅确认本次发布及所述发布后验证，不宣称整个长期需求全部完成。无需为已接受例外重新采样或扩建工具。
