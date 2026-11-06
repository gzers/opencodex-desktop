# 0.2.23 G0合同与分批放行

2026-11-06，用户已授权目标模式文档、开发、测试、发布及团队协调。合同分批冻结，不将部分放行当整版G0全部关闭。

- [L0路径/运行v1](L0-paths-runtime-v1.md)：新版程序根/用户根分离及启动/安装/Windows边界，基础代码由L0接线，原生制品待验。
- [L1模型合同v1](L1-model-contract-v1.md)：采用内容SHA256=1db45cc502033c28fcc77a07d25aceab8d30da7bd8ed611dd48dabdc9e1ca7b5。Library/Policy v2、content v2、portable model v6，来源/人工/自动引用和物化记录分开。L0选择采用此语义作为模型核心R1基线，生产验收仍待实现。
- [L2同步合同v1](L2-sync-v1/PROTOCOL.md)：采用contractDigest=sha256:8f082326a1503b66b6b7a4eefc13dcba1610780205c3ddb17c1e8dc64ded5267；实际DTO Schema/错误/预算/摘要样例一并冻结。首上传确认才初始化、多G停写只读恢复、不可变提交图及原操作只读核对；模型payload由L1 v6编解码，外壳版本独立。协议核心可R1，真实网络/应用集成仍待实现。
- [L3状态稿](../../../.adg/evidence/0.2.23/team/L3/states-v1/L3-PROTOTYPE-STATES.md)：63状态用于本版原型制作，非视觉通过；具体交互按冻结L1/L2及L0合同。

当前L1核心可分批开工；L2最终DTO/预算已按上述合同冻结，L0网络trait与payload装配继续实现；L3只在核准干净docs根和精确写集内制作，不复制dirty探索。L4执行入口见[Windows Git任务](../WINDOWS-0.2.23.md)。正式部署/同步不因测试发布目标授权而自动操作用户配置。
