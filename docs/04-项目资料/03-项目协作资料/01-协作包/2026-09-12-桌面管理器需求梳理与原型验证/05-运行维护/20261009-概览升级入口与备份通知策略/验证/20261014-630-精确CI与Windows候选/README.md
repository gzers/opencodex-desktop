# 6305747f 精确 CI 与 Windows 隔离候选

2026-10-14，Asia/Shanghai。feature/0.1.10-release-details@6305747f035945f3563296b42d21edcf5c816464 已形成同源审阅与Windows候选。源码父提交为be7395a，包含已验证焦点修复；main仍eae85322。PR#1及依赖它的PR#2均open/draft、未合并；PR#2 base=feature/0.1.10-update-center-focus@be7395acfce0ec1e36b83300b576db408f8c35bb，只审阅本轮17文件。

## 三个实际终态

| 运行 | 结果与范围 |
| --- | --- |
| [push CI38490716335](https://github.com/gzers/opencodex-desktop/actions/runs/38490716335) | 同630 SHA，completed/success；五必需job成功，main-only build skipped |
| [PR CI38491351619](https://github.com/gzers/opencodex-desktop/actions/runs/38491351619) | head同630、base同be，completed/success；五必需job成功，main-only build skipped。PR检出/构建身份与单独head候选分别记录，不混同二进制 |
| [Windows候选38492240481](https://github.com/gzers/opencodex-desktop/actions/runs/38492240481) | push全成功、远端SHA复核且同源候选查重为空后只dispatch一次；同630 completed/success，MSI/NSIS打包及隔离runner逐个新装/首屏/卸载成功 |

push/PR两轮MacRust均34组/884passed/0failed/6ignored；两轮WindowsRust及候选Rust各34组/612passed/0failed/7ignored，ignored及零测试组不计通过。前端501项/97文件已在本地与CI通过，本次终态独立记录各run来源与完整日志；不把这些当物理设备性能或原生UI矩阵。

完整API终态保存为release-notes-*-final.json。三份完整CI日志无损gzip留在本目录，原始大小/摘要见各*-terminal-log-summary.json；解压roundtrip再次核验，不筛去失败或ignored。CI中其他已安装工具/runner初始化输出只为实际环境日志，不能扩张为应用验收。

## 制品独立核验

两个artifact下载以API原大小/SHA256验证，ZIP CRC、安全路径、非链接条目通过，分别3/316项；两份installer-verification原字节一致。未执行Windows二进制、未在Mac安装或启动新应用。

| 制品 | 字节数 | SHA256 |
| --- | ---: | --- |
| [候选ZIP11863515199](https://github.com/gzers/opencodex-desktop/actions/runs/38492240481/artifacts/11863515199) | 7658531 | 8861851c191bc03d57dd6d594a037a6e93264268903c4896e1699fd641e37616 |
| 证据ZIP11863360486 | 1685594 | 7b31c6eeefbb132065e83a50b4a1852eb20bcd48b58e89420acc868fc0a7176c |
| OpenCodeX Desktop_0.1.10_x64_en-US.msi | 4591616 | 6725f7f7982a40229c7465a95bbb3ba822e28afab6c62fb008b3e18cbb4499d7 |
| OpenCodeX Desktop_0.1.10_x64-setup.exe | 3275286 | 3d85d834e689342e1e9dc27e14c533b7dd29a2d00a4185c016411825386eae30 |

包锁runner为win25-vs2026/image20260925.250.1、Rust/Cargo1.99.0、Node24.21.0/npm11.19.0/TauriCLI2.11.4，environment source与630匹配。runner新装EXE的MSI摘要23d1b43a24f544866c4708421c65e5ae774116241876aecd80cf3d030818e99e、NSIS摘要025106a21a752fe5a532f319ec75dee8e612012e8cf8db2c3606d6b38148547d分别匹配严格bundle marker预期。原builtEXE摘要c28e50b4dc8be999ea11ac4a7d2002fed7d61ade853c4872208913b3a8857803、markeroffset8078005。安装后的EXE字节未包含在下载artifact中，因此本机只核对runner回执关联；不能把回执摘要相等说成Mac独立对安装后EXE整文件重算。

selected-runner保留environment、MSI/NSIS startup和token helper原JSON。实际MSI应用PID9132、NSIS应用PID1312均elevated=true/elevationType1，applicationTokenVerified=true但mode=any/applicationModeMatched=false；不是普通/管理员双权限通过。HOME移除、沙箱override、DPR及自动收尾不证明默认安装目录data、真实用户根、物理DPI、正常退出、升级/OTA或卸载数据保留。codeSigned=false、releaseAccepted=false；无正式发布资产。

## Windows接续

先同步docs与固定630源码；读取原TASK/IMP、本目录及实现证据，核实下载摘要。现有be/PID10904为交接状态，先在Windows重新证明身份/实例数。未协调旧实例正常结束且确认全机隔离实例数0，不启动630或另一个原生测试。先完成旧be标题栏原因取证或明确保留中断范围，再用已准入的隔离安装/只读提取方案，保护日常版、MSI注册与真实数据。只读提取不能当安装验收。

新630候选重点复验概览/设置、管理器/面板说明四态，精确正文/核验网页、长内容内部滚动、独立重试与焦点、关闭重开及晚到结果；缓存命中实际零说明请求、切候选/通道/根隔离，说明错误不改变版本检查/安装或解除相关通知。没有可核验候选时不做说明请求，不能通过制造候选或CDP注入给产品判通过。故障夹具与真实来源/原生输入按证据类别分开。

其他原安装/data/卸载保留、双令牌/物理DPI/标题栏/Snap/托盘/退出、更新进度/签名/重启、备份/树表/事件、未过期缓存100切页/10重启、最终源真实30min/beta6h/stable/panel24h/直接触发与原始请求计数、同机官方0.1.9配对均继续按原合同。FZ50第三方脚本未许可不执行；Mac原生必须先协调Windows实例0，不用本轮runner替代。

## 治理与限制

原TASK revision2/in_progress/pending、九项AC绑定/八项required_checks与空验收保持。原macOS配对CPU+1.817629百分点/平均RSS+117.008965MiB失败保留，原因尚未定位，单元/CI不能解除。性能预算与冷/热各20、p95增量≤100ms且≤5%、10min CPU≤0.5百分点/RSS≤10MiB、主线程≤50ms及其他矩阵不变。没有自动合并、核心事实/通道晋升、真实迁移、日常替换、签名、公开发布或完成证书。

PR#2仅更新正文到实际三终态/制品范围，原前后body和API读回保留，head/base/open/draft未改。源码/下载/CI成功只形成待设备验收的审阅候选，不代表0.1.10整体完成。
