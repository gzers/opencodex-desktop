# 冻结离线producer专项：完整结果与反证

Run42586501118/job129057491679/source4162f5234c911be28e371f9a8463609ae1fa5d31；4项pass/0fail，**accepted:false、candidate-only技术范围**。44.183秒核心执行（无需等待凑8分钟）；不是双SID产品验收。

| 项目 | 结果 | 首错 |
| --- | --- | --- |
| 01 hypothesis-and-inputs | pass |  |
| 12 frozen-producer-supply | pass |  |
| 13 runtime-readiness-and-ports | pass |  |
| 18 owned-producer-cleanup | pass |  |

H-frozen-producer-stop-20261109-02中供应/启动预测得到支持；“Stop会持续失败”和“未知退出需保留fixture”的预测被实际成功反证。不能据此把旧42578649105或完整core42581940251失败改成成功，也不能断言旧失败必因依赖或硬件，候选装配和负载均不同。

离线依赖4054文件/308400749字节，inventory SHA256637557b7356853bf5c6dfd12efff72043750061a8e8d5aabdb9e5d3e3d5e0c41，候选/原供给逐字节一致且原供给未变，Bun --no-install。实际启动6732.7923ms；protected record、health HMAC、lifetime proof和ready通过。原GETcapabilities/reattest/POSTstop同TCP，实际HTTP200/success:true/sharedTeardown:performed；退出码0、Jobempty、leasefree、recordabsent，完整stop8141.3016ms。自有fixture/源码/离线暂存清理通过，无强杀/日常配置变更。

阶段：manager-risk471.6268ms；ownership-preflight2.1758ms；manager-stop5497.1401ms；native-client-restore1807.4541ms；grok-restore1.4059ms；official-teardown1812.7814ms。原源码manager-stop确会调用最长5秒PowerShell/CIM包装器清理。此轮证明该阶段耗时高，不等同于已经设计/验证可以跳过其完整性条件。

证据边界：当时驱动的officialClientRestoreConfirmed以官方performed响应判断，尚未读回配置文件；只作为官方响应/正常退出证据，文件确实还原的证明仍缺。全表后ef1a81a54新增启动时实际localhost/port注入读回、停止后model_provider=openai及所有代理路由/catalog字段移除读回，计入原10秒且不采集内容；反例函数本机过，实际Windows新断言尚未跑。将与Desktop消费者/维护包集成一起验证，不另跑一关。

原ZIP14234字节/SHA2569d5f037d8ee25fd4d5e4f2e9b7010638219c13e736c63c888c6d7b3f59528f0b与全部evidence-manifest成员摘要核验。生产names.2、Desktop全机停止扫描未更改，正式制品/双SID/升级回退/发布仍未完成。
