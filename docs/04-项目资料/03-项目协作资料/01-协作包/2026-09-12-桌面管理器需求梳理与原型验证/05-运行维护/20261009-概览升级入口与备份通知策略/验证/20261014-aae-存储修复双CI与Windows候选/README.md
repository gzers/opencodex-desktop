# aae 存储修复双 CI 与 Windows 安装候选

2026-10-14。替代上一验证记录的“新CI进行中”，原快照不改写。固定源码feature/0.1.10-integration-candidate@aae2a3fe37a19a33b2f423a1f0b04e5b376c9579；实现与红绿见[更新存储修复](../20261014-更新存储链接隔离与a7双CI终态/README.md)。本轮Mac仅后台，Windows当前真机实例不操作、不替换；源码与文档分别留在feature和docs/governance-main。

## 完整终态与实际检出

| run / event | 实际源码检出 | 结果 | Rust Mac | Rust Windows |
|---|---|---|---|---|
| push CI38521648491 | aae2a3fe | completed/success | 35组/900通过/6忽略 | 35组/627通过/7忽略 |
| PR CI38521655917 | 7365f7b0e776987f2b01e3d034e19990cf924120，PR4合成提交 | completed/success | 35组/900通过/6忽略 | 35组/627通过/7忽略 |
| workflow_dispatch候选38524086177 | aae2a3fe | completed/success | 不适用 | 35组/627通过/7忽略 |

两个CI的五必需job成功，main-only build条件跳过；Windows后端job内的release build/final EXE smoke成功。candidate package全部步骤成功，包括完整回归、MSI/NSIS构建、隔离新装/首屏/卸载。ignored与零测试组不算通过项目；新Junction分区、硬链接、有界读取及持久提交失败合并用例在Windows三轮确实执行通过，原件日志有名称/结果。PR关联head不代替合成实际checkout，独立候选使用head aae。

运行终态REST、job状态、checkout与完整日志ZIP摘要/CRC/安全条目一致，selected job字节gzip保存。两份CI evidence ZIP11874523110/11875146610大小9474982/9482979字节、API SHA256/CRC一致；测试EXE对应原hash回执。PR第一下载在8519680字节连接reset，截断摘要与错误保留，补取同artifact得到完整包；没有重跑CI、关闭TLS验证或执行包内文件。

两个CI startup实际应用均elevated=true/type1、modeany；PR startup expectedCommit为7365f7b0合成，push为aae，原sourceCommitMatched真实。它们不证明普通权限、物理DPR/DPI、真机外观或安装器数据保留。

## 候选资产与安装回执

[候选运行](https://github.com/gzers/opencodex-desktop/actions/runs/38524086177)；[下载候选ZIP](https://github.com/gzers/opencodex-desktop/actions/runs/38524086177/artifacts/11875775415)。登录有权限的GitHub账号后下载，ZIP内含MSI、NSIS与installer-verification.json。该链接是Actions候选，不是公开Release或stable更新源。

| 项目 | 字节数 | SHA256 |
|---|---:|---|
| installers artifact11875775415 ZIP | 7625913 | ea98090530f0cf981e028d95ddb27aa73b6efb49d3df380e65577aa2072e9d2f |
| evidence artifact11876045020 ZIP | 1690231 | e75f594995ca71cc93f7a2052f11d79946eddcccb0c52f7083b9b2ab43642396 |
| OpenCodeX Desktop_0.1.10_x64_en-US.msi | 4567040 | 0dbcb8cf2891c977b6cfa203e9ac13393bf68bb64e34c15c12af1b46af81dbd8 |
| OpenCodeX Desktop_0.1.10_x64-setup.exe | 3270031 | 3937bdcd97a84de7b62b3f198855c6ce9b9d1d7d1560e97b0db6d957f3b0ff9f |

两个ZIP实际API大小/摘要、CRC、安全条目、唯一条目都校验；两处installer-verification.json1970字节完全相同。回执sourceCommit=aae/version0.1.10/resultpass；MSI/NSIS包字节hash分别匹配；runner每个installExit0、首屏pass、uninstallExit0、应用EXE已移除。MSI预期/实装EXE hash为2e31e51552bb2ab1e0ae9fb49db0e0f4aabed080cc70bf8b31e50c12e5906824；NSIS为d191ac0bdf46eeb725bc057a9683dd8ec927b418508058393a9d434b39081870，与各自startup hash/路径/PID一致；包含预期bundleMarkerMSI/NSS、sourceCommitMatched及30秒responding/mainWindowPresent。

候选installed EXE本体未在artifact中供本机独立核验，本轮只证明包字节与runner记录的绑定；没有在Mac执行Windows安装器或EXE。两个实际应用均elevated/type1、modeany，不能抵双令牌；CDP沙箱首屏、自动结束不替代真实正常退出、默认安装路径data、旧安装器升级/卸载数据保留、KnownFolder/Profile或物理DPI验收。codeSigned=false/releaseAccepted=false。

## 依赖身份与原始字节

runner为win25-vs2026/20260925.250.1，Rust/Cargo1.99.0、Node24.21.0、npm11.19.0、TauriCLI2.11.4，environment.source=aae。Cargo.lock原Git blob135241字节/SHA225e522bcd78cc87155c85a09880af78415c3773f9b54cd98109e2135088b083；runner140827字节/SHAf7be175dc8a55c139781d3076b1eb79bb0af38ecb5664a77aaa8f2a87af9d8ab。5586行由LF变为CRLF，原字节不相等；只在比较副本中移除CR后LF字节逐一相等、TOML解析依赖对象完全相同，cargo --locked通过。归档原件未改换行，不能把“依赖相同”写成原文件byte-identical。锁原件留在远端证据artifact，本页仅记录身份，避免复制源码事实源。

## Windows接续顺序与原门禁

新aae候选包含焦点、精确说明/缓存、确认过的图标、查询持久终态与存储隔离修复，旧bb/d3制品不能证明新代码通过。使用新制品前由Windows端协调结束旧唯一隔离实例并确认实例0，再依既有隔离提取/专用安装方案启动唯一新实例；保留日常0.1.7/0.1.9及真实数据/注册信息。当前真机实例是否仍存活以Windows实时证据为准，本轮没有启动/终止/前台输入或切换。

固定SHA/包摘要/EXE身份后，优先复验新增说明四态/正文滚动/可靠网页/独立重试焦点/真实请求计数，管理器/面板详情返回全部与Tab/Escape焦点，后台持久提交/失败合并及cache静态链接拒绝，随后继续原双令牌/DPI/标题栏还原/图标、默认与自定义根/安装保留、更新/备份/事件/真实周期矩阵。测试使用自有临时根与受控来源，不能对真实数据制造权限/链接失败；性能采样独立安排，不以夹具负载干扰它。原Node控制进程启动迟延失败证据仍保留，需要原始进程诊断，不能因为新轮成功宣称原因已解。

原macOSCPU+1.817629百分点/RSS+117.008965MiB失败和预算不变：冷热每版各≥20，启动p95增量≤100ms且≤5%，10min闲置CPU≤0.5百分点/RSS≤10MiB，主线程≤50ms，有效缓存100切页/10重启零自动网络/查询进程，真实stable24h/beta6h。Windows局部低资源、CI与runner安装均不解除该失败或版本整体验收。

PR4已更新终态/候选说明，仍draft/open/unmerged/headaae；main仍eae，不自动合并。原TASK182字节3878/SHA bc7965b4…/revision2/in_progress/pending、acceptance_results空，overall RCP failed，目标active。FZ50第三方脚本未许可不执行，无真实迁移、日常替换、稳定核心事实晋升、签名、发布、通道晋升或完成证书。Mac原生测试要另先确认Windows实例归零。
