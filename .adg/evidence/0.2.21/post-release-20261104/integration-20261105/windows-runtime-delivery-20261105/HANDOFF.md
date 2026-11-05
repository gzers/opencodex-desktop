Windows自动重启运行包接续（477d7edea / Mac补丁d5a6aaf99）。

只补本轮运行包查询与交付；上一轮材质、面板、云范围及完整CI不重复。治理读取4c39841f3，交付后补读2beea169f更新。独立上游dev完整检出0a30c551，ZIP导出Mac补丁保持35703字节、SHA256 0d8eaaec8c361d914fb901db2c8ef6a36702ce3c7ec209cb3f255817f8e32bb7，十个后像全部精确匹配。补丁不在管理器源码中应用，仅在.toolchain隔离上游副本应用。

两条查询已由GetOwner逐进程RPC换为同句柄SID、原生路径和创建时间；保留10s/8s、窄候选正则、同包根/当前用户、PID换代、祖先链、未知拒绝及原退出/激活流程，保留桌面base64命令行供兼容PAC上下文判断。新query直接嵌入TypeScript，分发不依赖相邻PS1。Windows110定向通过/2POSIX专属跳过，兼容启动+布局15通过；完整dev类型2317文件、Git索引隐私与结构检查通过。安装后的实际嵌入脚本PS5正反例通过，真实当前用户appserver约1028ms。完整affected-import图240秒未终结，1032pass/0fail只是中间记录，仍留上游CI缺验。

安装交付：注册表最新正式包2.97.0 / Git cab7eb7原归档完整性核验，仍含旧GetOwner。使用它的完整原包作为基底，只替换已审阅的三个运行源码，改为明确本地预发布2.97.0-post021.ownerquery.1并更新compatibility-version摘要；其它文件逐字保持，GUI仍取原包。生成opencodex-2.97.0-post021.ownerquery.1.tgz，SHA256 c8b1d4b7e78792df0b693ed5eda8e3fb0e9da6159a5643da1591abd65519e5ec，15405720字节。独立npm12.2.2/Node24工具、空user/global npmrc、独立缓存与前缀、ignore-scripts实际安装102依赖，2522包文件摘要核验；移除PATH外部Bun后Node bin入口仍exit0/正确版本。制品manifest、原包integrity、lock/Node/Bun摘要附证。15MB二进制仅留本机.toolchain/post021-runtime-delivery-20261105/installable-local1，源码构建器可复现，不将它冒称上游正式包。

正式交付仍需维护者集成并发布新的@bitkyc08/opencodex版本；如选择自建归档/分叉为生产安装来源，须先明确采用与维护责任。本次没有对外fork/PR/issue、发布、合并main、修改正式npm来源或覆盖日常已装包。

隔离Administrator现有测试资料已获授权，正式MSIX26.1029.7803.0；继续使用已验证产品apps树相同的MSVC fa9df83a及精确30804管理器。已准备POST021-INSTALLED-RUNTIME.ps1，核对当前测试服务56436/创建639294667532287570、正式测试主进程92496/创建639294668232896800，并实际运行保存65558→受控服务停机失败→恢复同候选后原通知重试→渠道公开输入改为Post021AdminRenamed→保存/默认重启/模型引用读回。全过程只走真实管理器公开动作与默认运行包流程，不注入restart/adapter/IO；比对其它账号客户端/appserver代际及15119配置摘要。当前等待Administrator会话执行结果，不能把准备、负例或旧2.84候选通过当本轮完整通过。无真实同步或推理。

失败保留：首次安装策略测试因测试驱动regex转义错误失败，修驱动后通过，运行源码未动；ignore-scripts使本地Bun依赖shim未配置，bun run门禁首尝试exit255，使用既有已核验Bun直接执行同脚本后passed；全导入图预算超时不写通过。历史长期任务误停日常Bun原件仍保留，不作整个会话零副作用证明。

回交代码codex/post021-windows-final / d54517eaf73ab2e06ca9556043ef665cac3e60ba（构建与保护测试02c7bcdef、隔离入口d54517eaf）；当前Administrator实测待执行，RCP保持failed，不生成CERT。
