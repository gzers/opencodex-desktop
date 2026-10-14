# 查询夹具阶段 CI 终态与 Windows 接续

2026-10-14；此终态补充前一记录的“CI进行中”，不改其封存证据。源码 feature/0.1.10-integration-candidate@8be5b4cc7497d08b323839a77548ae189909a3be，仅为共享Windows夹具阶段记录及CI输出补充；生产面板所有权修复来自fd、图标来自bb，详细身份/本地红绿见[前一记录](../20261014-面板查询后台提交与Windows失败诊断/README.md)。本轮没有启动、激活、移动或关闭任何桌面实例，没有新原生性能样本。

## 精确状态与实际检出

| run / event | 关联 head | Windows 实际 checkout | Rust Windows | Rust Mac | 完整 CI |
|---|---|---|---|---|---|
| 38507499433 / push | 8be5b4cc | 8be5b4cc | 库516通过/7失败/4忽略，后续组未执行 | 35组/892通过/6忽略 | failure |
| 38508035604 / PR | 8be5b4cc | 032088fd60c24ea669cb0d7611ebcc4e59b7c9e8，PR4 synthetic merge | 35组/620通过/7忽略 | 35组/892通过/6忽略 | success |

PR实际checkout是合成合并，不把关联head或PR测试EXE当独立head制品。ignored不计通过；push Windows失败阻止后续release build/EXE smoke，frontend、Mac backend、release-tools与token证据job成功，main-only build条件跳过。PR成功不解除push或原fd/fb失败。

两份run REST原件绑定各自run/head/event/终态；push完整jobs原件与PR4 check回读交叉核验。两个全日志ZIP261796/290910字节，SHA256、ZIP CRC、安全条目路径已验证；原job字节逐一压缩存放selected-log，不改换行或缓冲输出时间。新push binary artifact API可读，但ZIP下载TLS/读取超时；没有声称取得或核验新Windows测试EXE/安装器，下载失败记录保留。

## 阶段记录说明了什么

push的七项version success/nonzero/timeout、metadata success/nonzero/overflow/timeout全部在夹具观察失败。约2.0–2.84秒查询结束时，每个父脚本、子脚本、句柄就绪以及独立control Node的启动标记均不存在。控制进程通过std::process独立启动、不加入查询Job，也没有CREATE_SUSPENDED；因此仅检查生产Job的resume/cleanup不足以解释当前观测，必须同时检查宿主调度、Node启动与标记I/O。

缺少标记不等于已证明Node内部哪一步停住；没有进程CPU/线程/加载器/防护事件或完成时控制存活快照，不能认定为系统故障、Defender影响、环境波动或产品缺陷。主查询的assign/resume已返回，仍不证明脚本执行。标记在查询完成后读取，不能拿测试结束缓冲打印的GitHub时间作为子进程真实阶段时间；阶段JSON的atUnixMs才是夹具写入时刻。

PR成功轮次中，三版本夹具control/parent标记距fixture开始约454–461ms，创建子进程、ownership-ready之后，成功/非零退出分别502.5484/503.7305ms；timeout仍按2.0057744秒结束并验证原树清理。metadata非零退出约109.4978ms。成功/非零的部分descendant-started标记未写，仍有已验证OS句柄与原存活/清理断言，不能误写为没有创建子进程。该数据提供对照而非根因/性能通过证明。

新增标记带来少量测试I/O；未预热/等待Node、不减少并发、延长截止、替换结果或忽略断言。生产查询两秒和<4秒清理要求、外部8秒watchdog干预即失败、无关进程存活与父子句柄创建/图像身份合同保持。

## 下一项具体诊断

Windows以固定8be源码继续后台调查，保留当前唯一bb原生实例。先避开既有性能采样窗口，核对显式Node路径/摘要、Rust测试EXE摘要及source checkout；原完整CI日志先对照。运行原夹具时同时收集限定时间窗内自有query/control的创建、状态与启动阶段，区分进入JS前迟延、标记写入迟延或句柄接管迟延；原默认并发、两秒截止和断言不变，失败不当场盲目rerun或增预算。不要按名称结束其他进程，不启动第二个原生实例，不执行第三方生命周期脚本。

当前bb旧制品不含fd面板取消修复，后台Rust夹具不等于新产品原生验收。原因定位和修复后的新SHA CI通过之前不触发新候选；后续制品仍须单实例协调切换并做原矩阵。Mac没有可运行Windows夹具的宿主，不继续堆诊断脚手架猜因；等待Windows原始进程证据，同时仍可源码审阅与后台修复。

## 门禁与治理

main仍eae85322，PR1–4均未自动合并；PR4已更新为终态与限界并保持草稿。原TASK182字节/摘要、revision2/in_progress/pending及空acceptance_results保持，回执总体failed。原macOS CPU+1.817629百分点、RSS+117.008965MiB失败和冷热各20、启动p95≤100ms且≤5%、10min CPU≤0.5百分点/RSS≤10MiB、主线程≤50ms、有效缓存100切页/10重启零自动查询、真实周期及双平台矩阵不变。

Windows仍在验收、Mac仅后台；Mac原生启动必须先协调Windows实例结束且确认0。没有FZ50许可，未执行其脚本；不改真实数据/日常安装，不签名、发布、stable晋升、提升稳定核心事实或创建完成证书。目标保持active，本轮仅封存当前进度。
