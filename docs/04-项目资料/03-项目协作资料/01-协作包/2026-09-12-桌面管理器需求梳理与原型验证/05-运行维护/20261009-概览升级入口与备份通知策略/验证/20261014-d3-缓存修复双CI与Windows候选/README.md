# d3b83b38 缓存修复双 CI 与 Windows 隔离候选

2026-10-14，Asia/Shanghai。源码feature/0.1.10-release-details@d3b83b3822bf10a511556d80dc851c2ccfb17cf5（parent6305747f）已形成新的同源Windows候选。修复只改变说明缓存语义校验与HTTP客户端惰性初始化；说明四态、活动根缓存路径/schema/TTL、原来源/预算和调度/安装门禁保持。原12通过/4失败与新16通过、后端888/6ignored的本地证据见相邻“20261014-说明缓存一致性与延迟客户端”，不将旧630制品扩张为d3证明。

## 精确运行终态

| 运行 | 实际结果 |
| --- | --- |
| [push CI38495887680](https://github.com/gzers/opencodex-desktop/actions/runs/38495887680) | head=d3，completed/success；五必需job成功，main-only build skipped |
| [PR CI38495891289](https://github.com/gzers/opencodex-desktop/actions/runs/38495891289) | head=d3、base=be，completed/success；五必需job成功，main-only build skipped。PR检出制品与独立head候选分别记录 |
| [Windows候选38497980368](https://github.com/gzers/opencodex-desktop/actions/runs/38497980368) | 两CI全成功、远端d3前后复核及同SHA候选查重为空后单次dispatch；同d3 completed/success，MSI/NSIS打包和隔离runner逐个新装/首屏/卸载通过 |

两轮CI MacRust各34组/888passed/0failed/6ignored；两轮Windows及候选Rust各34组/616passed/0failed/7ignored。ignored及零测试组保留，不计通过。本轮UI未改，本地501结果只归630；新两轮CI实际包含完整前端测试/类型/构建通过。所有通过仅对应记录中的源码/环境/检查范围。

三个API终态及完整日志无损gzip归本目录，各*-ci-log-summary.json记录原字节大小/摘要与独立cargo计数。进行中快照和先前失败不改。Mac未启动桌面应用、Windows安装器或原生性能采样，未执行未授权官方包脚本。

## 制品与回执独立核验

| 制品 | 字节数 | SHA256 |
| --- | ---: | --- |
| [候选ZIP11864817991](https://github.com/gzers/opencodex-desktop/actions/runs/38497980368/artifacts/11864817991) | 7662256 | c1cb25d29668458269e696a2484d0e62608fda42f0381d7601b297e76cc13d6b |
| 证据ZIP11865046498 | 1694701 | b32ad560b19968e169efddc7993f9eadc3ee0f9f8d3ede647b3e298630d033d2 |
| OpenCodeX Desktop_0.1.10_x64_en-US.msi | 4595712 | 5032162d28a2c0ead5387cca058aeb42af80bddd69206feb7abe690a1915171c |
| OpenCodeX Desktop_0.1.10_x64-setup.exe | 3277473 | d51f5b47cda0c9010de948a687090737e430c35a702ee53b90111eef5b38b55f |

两个ZIP与API大小/SHA256一致，CRC、唯一安全条目与非链接核验通过，分别3/315项。两包installer-verification原字节相同；MSI/NSIS字节和摘要与回执一致，sourceCommit=d3/version0.1.10/resultpass。回执严格bundle预期与实际安装EXE摘要相等：MSI1c265b5c8310bc71bf411986b46c4b48db5b0ef3e4ddfb25d00ddb927785ae64、NSIS1fea9806bea06ed5c69f5e068ae57aa6c848e115aa73041e0b9fdf4a5b6546f1，marker offset8082541。安装后的EXE字节不在下载artifact内，本机只核对runner身份回执与摘要关联，未声称独立重算安装后EXE整文件。

selected-runner保留4份环境/startup/token原JSON，source与d3对应。MSI应用PID1376、NSIS应用PID8668实际均elevated=true/elevationType1；mode=any、applicationModeMatched=false。这不能计普通/管理员双权限通过。HOME移除、sandbox root override、模拟DPR和自动收尾不证明默认安装根data、自定义根、物理DPI、正常退出、旧版升级/OTA或卸载数据保留。codeSigned=false/releaseAccepted=false，不是正式分发/签名通过。

## PR 与 Windows 接续

PR#2head=d3，base=feature/0.1.10-update-center-focus@be7395a，open/draft、未合并、auto_merge=null。本轮仅更新正文到实际d3双CI/候选终态，前后API与正文保持原字节留存。PR#1/main仍未合并；本文不授权合并/签名/发布。

Windows可下载新候选但不得自动安装或启动第二实例。先重新核实当前be实例/根/EXE身份与保护数据，协调结束旧实例并确认隔离实例数0；采用已经准入的独立安装或只读提取方案再切d3，保留日常安装、MSI注册及真实数据。只读提取不替代安装门禁，旧630及be通过只归各自旧来源。

新d3优先复验两入口/对象四态、精确版本正文与网页、长正文内部滚动、独立重试/焦点、关闭重开与迟到结果；有效说明缓存真实零HTTP请求、异常代理下离线缓存可读、坏语义缓存重查，以及切候选/通道/活动根隔离。说明错误不改变版本检查、安装结果或解除相关通知。故障夹具与真实来源/原生输入/请求计数分开记录，不能注入候选状态给产品判通过。

标题栏还原、双令牌/物理DPI、安装根/data与卸载保留、签名/更新重启、备份/树表/统一事件、未过期缓存100切页10重启、真实30min/beta6h/stable/panel24h、直接触发/原始请求计数及官方0.1.9同机配对保持原矩阵。切Mac原生须先协调Windows实例0，Mac本轮仍只后台。FZ50显式许可缺口保持，不执行相关生命周期脚本。

## 门禁保持

原TASK revision2/in_progress/pending、九项AC绑定/八项required_checks、空验收保持。macOS既有CPU+1.817629百分点/平均RSS+117.008965MiB配对失败不解除；缓存修复未证明其因果。冷/热每版各至少20、启动p95增量≤100ms且≤5%、10min CPU≤0.5百分点/RSS≤10MiB、主线程任务≤50ms及其他预算不变。没有真实迁移、日常替换、核心事实/稳定通道晋升、自动合并、签名、公开发布或完成证书。
