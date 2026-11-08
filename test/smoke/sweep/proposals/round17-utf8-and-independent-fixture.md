# 第17轮：测试脚本编码与诊断独立性修正

Run 42462945397，source b5a4bf92dad36e9a49621c59530eebbab922a36f：10 pass / 16 fail，accepted:false；114份证据的bytes/SHA256全部核验一致。原目录 /private/tmp/ocxd023-sweep-r17-42462945397-download/windows023-sweep-42462945397-1/ 保留。

通过01、03～07、19、20、24、26。第26项七条真实Windows权限/HTTP/消费者回归全部执行并通过，包括protected owner-only记录、原创建合同保持、公开授权/硬链接拒绝。

第02项新checker的Path.read_text未指定UTF-8，Windows cp1252解码中文helper失败；08～18、21～23因此没有当前程序/用户/原生回执。第25项因同一构建前置没有UI dist而失败。不伪报这些未执行项通过，不将本轮当产品端口验证。没有增加产品修改。

本批次统一修正：

- checker的源码、生成驱动及子进程stdio明确UTF-8；Python证据流也明确UTF-8。ESM schema定位使用标准file URI，适配Windows盘符及特殊路径。
- 实际schema校验拆成独立第27项，always/continue-on-error及独立不可覆盖回执、完整summary/manifest；原26项全部保留。该辅助检查不能提前抑制当前二进制/原生/账户/清理诊断。
- 正式Release Windows安装验收复用已验证的临时Server MSI client-equivalent策略准备/精确还原，restore为always，证据沿同一发行目录上传；不改安装器权限或产品策略，不使用AlwaysInstallElevated。
- 本机真实采用包schema两个端口夹具通过，临时目录清理；全量sweep四条回归、发行装配十三条检查通过。七条Windows原生回归使用本轮原证据，不冒充真实两SID/端口或最终发布通过。
- /private/tmp/ocxd023-names2-manager-chain-b5a4bf92d.log：真实管理器四种改名→完整names.2安装包→自己的Codex目录链1过；原模型ID/未选配置保持，OS禁网，临时I/D/H/CODEX_HOME及精确目录锁清理通过。

前序失败均保留。后继完整轮继续实际两SID默认自动分配、固定冲突、健康/就绪、六秒重叠、B停A存活及完整清理。最终正式制品和签名相邻OTA/回退仍须独立证据。
