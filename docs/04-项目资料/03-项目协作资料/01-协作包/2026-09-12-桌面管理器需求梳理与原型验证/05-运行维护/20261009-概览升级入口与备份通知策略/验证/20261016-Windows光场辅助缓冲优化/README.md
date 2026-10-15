# Windows 光场辅助缓冲优化

代码 `feature/0.1.10-windows-query-etw@8be87e2d54a895d4fe02e31300100a1ffed90121` 已提交推送。相对02候选只改 `meshGlow.ts` 的WebGL上下文选项与说明：`depth:false`、`stencil:false`。着色器、DPR上限、画布尺寸、颜色、动画、画质及光层覆盖均保持。

此前02同机配对平均RSS +38.328092MiB失败保留。本改动是对实际未使用缓冲的优化，不是已经定位该差异的唯一根因，也未获得新EXE性能通过结果。

## 修改前的真实运行观察

在仍运行的02隔离原生WebView中只读读取实际应用画布：light/mid、真实DPR2、1496×1342绘图缓冲、CSS935×839；`getContextAttributes().depth=true`、`DEPTH_BITS=24`，但深度测试关闭。该观察记录于 `.adg/evidence/OCX-0110-20261015-WIN-MESH-BUFFER-DIAG/candidate-webgl-attributes.original.json`，不是GPU内存绝对计量或物理DPI通过证据。

随后仅创建两个未挂到文档的诊断WebGL上下文，使用修改前完全相同的生产顶点/片段着色器与统一参数，比较保留/取消深度缓冲：两种尺寸×浅深×三个动画时刻×两个中心位置，共24组。所有组均有非零实际像素，RGBA逐字节相同、SHA256相同，深度位从实际>0变为0。诊断上下文和资源已释放，原产品画布未改，不是给用户展示的画面替代品。

## 源码验证与制品边界

本地原锁依赖的UI99文件/533项、类型检查、生产构建通过。真实像素结果及着色器来源摘要、原始日志、代码远端读回均保留；不新增仅镜像上下文选项的测试。两轮精确head CI `38763397150`（push）/ `38763409850`（PR）沿原执行，不为等待历史失败重跑成功ETW。完整终态、实际checkout、候选与后续复测另按原件追加。

00:13:55已取得两轮完整success终态；四份原job日志独立下载、大小/摘要复核，每轮Mac923passed/6ignored、Windows648passed/7ignored、各35组。push实际checkout8be，PR实际checkout`7bf52c8efd86c8de14bc54732946cc3f8bf42f13`，两者完整Git tree均为`16749535d8f1e47b31150de5817d473d15e8eec4`，不假定EXE字节相同。fmt/Clippy/前端/工具及最终EXE沙箱冒烟随CI成功；原查询失败窗口仍未取得。

只派发一次同head候选`38767255195`，00:14:10已读回排队，后续沿此run，不另发构建或发布。API读取TLS失败及PR更新方法修正原件保留，未关闭证书校验或修改全局网络设置。

PR #5说明已按真实范围更新并读回，仍draft/open、未合并、未开启自动合并；明确保留RSS失败、Tauri恢复交接短暂双EXE、外置HOME及其它未完门禁。PR第一次说明文件按系统GBK读取失败发生在变更前；后续PATCH的TLS握手错误经独立GET确认原文未改变后，仅作一次Git curl写入并读回，不盲重试。

原019配对/p95/CPU/RSS/50ms/周期预算不变；该优化尚不能解除Windows或Mac性能failed。新候选须先核验同SHA完整CI、制品API摘要、实际安装载荷及原生上下文，再测实际RSS。不开新日常实例，不替换日常安装，不发布。
