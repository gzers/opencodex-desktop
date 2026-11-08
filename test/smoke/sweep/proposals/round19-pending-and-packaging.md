# 第19轮完整结果与同期候选/CI问题

Run42466451669，source396ed1fbdf332eb7fe963737b5e5874ae65d7a6b，26 pass / 1 fail，accepted:false。269份证据大小及SHA256全符，原目录/private/tmp/ocxd023-sweep-r19-42466451669-download/windows023-sweep-42466451669-1/保留。

01～13、15～27通过。14：user1省略端口自动取得51366，健康HMAC/PID/port通过；实际状态读取已返回相同PID/port的pending/degraded，保护事实null。独立实际后端激活验证452ms、5秒状态来源771ms均通过，只读来源仍是pending，不能以健康或晚到响应替代30秒真正就绪。user2默认自动分配/冲突/重叠协议未完成。账户/Profile、精确代际进程、原ACL、Server策略清理通过。

同期正式候选42463221022：原后端回归、私有创建、原生凭据均通过，release编译15m37s成功；Tauri下载官方NSIS3.11工具ZIP遇HTTP504，尚未生成可验收安装器。原失败日志/private/tmp/ocxd023-candidate-b5a-42463221022/failed.log保留。

同期PR CI42464484815：原生安装失败/恢复/WebDAV就绪观察通过，更新测试在新版本文字可见后立即点确认，没等共享check完成和真实启用按钮，query UI action断言失败。原证据/private/tmp/ocxd023-ci324-42464484815/保留。没有用该局部完成宣布整版通过。

## 一次批量处理，产品语义不改

1. 更新测试在原DOM等待预算内等实际启用、精确文案的公开确认按钮；仍经真实点击、备份、npm pack失败、重开、源/入口/清单字节保护，不直接调用安装处理器或取消busy门禁。
2. 合成渠道夹具明确openaiProviderTierVersion=2，避免原版启动迁移；实际采用包configSchema和projectOpenAiTierMigration共同核验两个端口夹具，要求迁移changed=false和providers完全相同。原无标记对照只重写格式和port0、provider不变，故不把它冒充pending根因。
3. 原30秒启动失败后追加15秒只读readyz状态和仅文件名目录观察，自有进程采样只输出固定类别、不记录原始argv/密钥。原30秒/10秒/5秒/100秒driver运输预算不变，晚到观察accepted:false且originalBudgetPassed:false，不放行下一用户服务。
4. 增加官方Tauri CLI装配wrapper：只有明确failed-to-bundle且HTTP临时错误408/429/500/502/503/504最多三次；编译/签名/403错误不重试，保持最终退出码。每次日志/结果独立保留，失败不覆盖；固定CLI2.11.4仍负责下载工具完整性、编译、签名和制品。候选/Release接入并上传尝试原件。版本查询只转发一次保留原格式。

## 证据与实际范围

- 实际子进程重试/原错误不重试/精确版本输出三条，以及原生探针回归合计28过；五条基础包装配、四条sweep、十三条发行装配检查过；真实采用包schema/迁移投影过，两用户helper生成驱动纯检查过。
- 当前源码CI396的Mac后端2227过/0失败/37原忽略；前端215文件1590过；原日志/private/tmp/ocxd023-ci396-42466441112/，PR merge3debb57产品apps树e106149ef8a0f88ef4e2a51035fa821a32a6a486与候选一致。Windows后端尚未完整结束。
- Mac当前完整app/严格ad-hoc签封、新目录离线names.2首启/实屏、精确I/U/进程清理通过；DMG只读挂载与源app树一致，scratch/mount清理。证据/private/tmp/ocxd023-names2-396-firststart-evidence/及/private/tmp/ocxd023-names2-396-mac-package/；不冒充正式更新签名/OTA。
- Mac startup对照仅作为诊断：OS禁止日常HOME读取，原版无法证明本机服务归属而明确拒绝Codex写入、readyz failed。不能用它解释Windows pending或将关闭客户端集成当作修复。全部临时H/服务已清理，失败和对照保留。
