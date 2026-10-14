# 更新存储链接隔离与 a7 双 CI 终态

2026-10-14。Mac继续后台，Windows继续原有验收。本轮代码feature/0.1.10-integration-candidate@aae2a3fe37a19a33b2f423a1f0b04e5b376c9579，父a7feba5f；文档仅docs/governance-main。未启动/激活/结束任何原生应用，未触发新候选打包、安装器或FZ50脚本。

## 已证实的缺陷与修复

原调度Schedule.load/save及版本缓存load_cache/save_cache直接打开或原子写入拼接路径。活动根内manager-state、cache或updates若为链接目录，会读到另一根的截止/候选，并将新内容写进外根。两项真实临时根回归在a7实际0通过/2失败，read_refused=false/write_refused=false/external_unchanged=false；仅作用于测试自有两根，未接触真实用户目录。

aae在update/storage.rs建立更新领域共用路径检查，Schedule、版本缓存和release-notes缓存使用同一入口。活动根必须绝对且为实际目录；根以上OS别名（如/var→/private/var）不禁止，每个根内组件必须正常目录，Windows拒绝FILE_ATTRIBUTE_REPARSE_POINT。叶文件必须为普通单链接文件；Unix O_NOFOLLOW/O_NONBLOCK、Windows FILE_FLAG_OPEN_REPARSE_POINT与handle metadata/link count核对，拒绝符号链接、重解析点、硬链接及特殊文件。read_file先核定大小并保留limit+1读取上限，缺失读不初始化分区；写入仍使用原同目录atomic_write，原子替换前先检查路径/现有叶身份。

实现只改变既有路径的接受条件，不增加持久文件/schema/缓存槽位，未改mac默认或Windows实际安装目录data规则、活动自定义根优先、16KiB调度/512KiB版本与说明缓存读取预算、说明TTL、2秒查询截止、自动周期/退避或事件策略。目录元数据每组件使用一次结果，避免重复stat；这不是原生性能通过证据。

本检查覆盖已存在的静态分区链接和叶文件。它没有持有每级目录句柄完成全部相对读写，不能宣称证明抵御恶意并发替换父目录的完整race隔离，也不代表应用其他分区已全量审计。原W4/备份/原生矩阵仍开放，不把局部修复写成全存储完成。

## 红绿与最终本地结果

首轮Unix符号链接两项失败保留。加入WindowsJunction分支后，用最终完整调度测试模块9356字节/SHA256 58f6896781a06f86c7de6b4342a470a4d663567ee0c0745526c3a613486fa914，对a7生产调度前缀再跑：0通过/2失败；最终源码模块完全相同且通过。临时替换只用于受控基线红验证，finally恢复最终schedule字节；源码hash和提交blob独立比对。无原生运行或来源包生命周期执行。

链接用例覆盖manager-state、cache、updates及叶文件，断言拒绝读写且另一临时根字节保持。Windows使用系统PowerShell创建Junction，不要求开发者模式/符号链接许可，传入路径为环境字符串而非脚本插值；叶文件硬链接与handle数量检查需要新SHA WindowsCI实跑，Mac不称通过。其他测试覆盖无效相对路径/../越界、缺失读零创建、原大小边界/硬链接内容不截断、UnixFIFO无writer仍立即拒绝。说明领域既有缓存/代理/期限/缺失/正文语义回归保持。

最终优化后Rust35组/900通过/0失败/6忽略，fmt/严格Clippy all-targets/features integration-test/diff通过；第一次完整900通过及优化后完整900通过两轮原日志分别保留。ignored不计通过；UI无改动，本轮不宣称全套UI或原生重跑。四代码/测试路径工作字节与aae提交blob相同，a7三条面板持久终态路径、bb三十条Windows图标路径和五共享文件原字节保持，未跟踪协作探针留存。

## 精确双 CI 终态补充

| 源码关联/事件 | run | Windows实际检出 | Mac Rust | Windows Rust | 完整状态 |
|---|---|---|---|---|---|
| a7 push | 38518361054 | a7feba5f | 35组/895通过/6忽略 | 35组/623通过/7忽略 | completed/success |
| a7 PR4 | 38518366179 | f5f774dd47194fb2ea04a3b23e0ab3c75a71a6bd（合成） | 35组/895通过/6忽略 | 35组/623通过/7忽略 | completed/success |
| 9cf PR4补充 | 38517007573 | e0e8c8c41f4aa3963db6cd27ec4a9aca5ef2445c（合成） | 35组/893通过/6忽略 | 35组/621通过/7忽略 | completed/success |

全部日志ZIP的实际大小/sha256、CRC与安全条目、对应run REST及checkout原件已核验；各五必需job成功、main-only build条件跳过。a7两轮含最终release build/runner EXE smoke成功，仍非安装器候选或真机双权限/DPI验收，合成检出不当独立head制品。旧通过记录不证明本轮aae。

aae push38521648491/PR38521655917最后观察in_progress，head分别核对为aae。WindowsJunction/handle编译运行与新SHA制品仍待后续终态；没有手动rerun、dispatch候选或用a7替代它。PR4现title“修复查询持久终态与存储隔离”、draft/open/head=aae/merged_at=null，main仍eae，PR1–3未自动合并。

## 未解除的门禁

原fd/fb/8be Windows查询夹具启动失败证据保留，独立control Node也无及时启动标记的原因未定位；后续成功轮次不足以认定环境波动或根因已解。新存储修复不声称解决该问题。Windows可继续固定当前源后台诊断，避开性能采样，不增加/切换现有唯一原生实例。bb旧制品不能证明本轮代码。

原macOS配对CPU+1.817629百分点、RSS+117.008965MiB失败不变；冷热每版各≥20，启动p95增量≤100ms且≤5%，10min闲置CPU≤0.5百分点/RSS≤10MiB，主线程≤50ms，有效缓存100切页/10重启零自动网络/查询进程，真实stable24h/beta6h及双平台/安装保留/更新备份事件门禁全部保持。原TASK182字节3878/SHA bc7965b4…/revision2/in_progress/pending、空验收不变，新RCP总体failed，目标active。

Mac切原生须先协调Windows实例结束并确认0；FZ50许可未收到、不执行。没有真实数据迁移、日常安装替换、稳定核心事实晋升、签名、发布、stable晋升或完成证书。
