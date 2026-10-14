# 面板查询后台提交、集成身份与 Windows CI 失败诊断

2026-10-14。用户明确 Windows 仍在验收，Mac 继续后台。此轮没有启动、激活或结束 Mac/Windows 原生应用，没有执行 FZ50 第三方生命周期脚本。Windows 最后记录为 bb3122d 候选 PID39680 的单实例；这不是本轮实时存活检查。原日常安装与真实数据不操作。

## 面板提交所有权修复

feature/0.1.10-release-details 的 fd31d7167c03b1e4e83554a9bf778a2ffb9baf4e 修复已接受面板元数据查询的观察者取消风险：原阻塞查询仍可能继续，但持有准入的观察者未来被取消后，后续缓存/持久截止/事件提交会消失。现在与管理器共用 run_check_owned 的独立任务所有者，准入持有至 official_remote_latest_owned 完成终态。既有两处事件 emission source 指向实际内部提交点；事件身份、触发源和策略不变。

加强版测试相同字节 SHA256 f489a81346e1f4ab84d2028b7522ba58bf10c643ee15e1401d57adeb348af2db 在 d3 基线红 1通过/1失败，新实现绿2通过。受控元数据未来配合实际临时根 Schedule reserve/complete/cache，验证观察者取消后成功86400秒截止或失败300秒退避，以及管理器 stable/beta entries 不变；冻结根在 setup 前拒绝。首轮 manager 字段取值不充分，随后改为读取实际 entries，两轮红日志均保留。终态事件仅为合成标记，不证明原生 Tauri IPC 取消或真实通知投递。

fd 本地完整 Rust34组/890通过/0失败/6忽略，相关 UI3文件/16通过，fmt/strict Clippy/diff通过。未增加查询或周期，未调整生产两秒版本/元数据截止、存储根或许可门禁。

## 与 Windows 图标集成

feature/0.1.10-integration-candidate 的 fb4a6fe32d6605da6f6eec26a32208c027909631 以 bb3122d678a9c6fdbd2c8bd78fa9f3f22b955c8a 为父提交，接入 fd 的四文件补丁。四文件与 fd、图标30路径与 bb 原字节相等；五共享文件与 docs/main 相等。集成本地 Rust35组/892通过/0失败/6忽略，fmt/strict Clippy通过。ignored不计通过；不能把 Mac 测试数量当 Windows 数量。

main 仍 eae85322，PR #1 焦点修复仍草稿未合并；PR #2 已更新为 fd 的实现和失败事实。新 PR #4 以图标分支为 base、8be 集成分支为 head，保持草稿已附当前对话。未自动合并主线或其他功能分支，不用旧 bb/d3 安装包证明新增面板修复。

## Windows CI 原失败

| 源码与 run | Mac 后端 | Windows 后端 | 结论 |
|---|---|---|---|
| fd push 38503064187 | 34组/890通过/6忽略 | 库519通过/3失败/4忽略，后续组未运行 | failure |
| fd PR 38503069354 | 34组/890通过/6忽略 | 34组/618通过/7忽略 | success；不解除同head push失败 |
| fb push 38503486962 | 35组/892通过/6忽略 | 库519通过/4失败/4忽略，后续组未运行 | failure |

两轮失败均有 version_nonzero_exit、version_success、version_timeout 三项。日志显示 CREATE_SUSPENDED 根已 assign/resume（9–26ms量级），约2.03秒返回 Timeout，但父/子PID及ownership-ready都不存在；该轮没有建立预期的 Node 父子夹具。fb另一个 metadata_nonzero_exit 已建立句柄，但2.0029228秒返回 Timeout而非Failure，失败位置是结果分类断言，不能误写为4秒清理耗时失败。尚不能认定环境波动或产品缺陷，不能把成功PR轮次盖过失败。

三轮镜像均 win25-vs2026/20260925.250.1、Windows10.0.26100、Rust/Cargo1.99.0、Node24.21.0；runner不同，四逻辑CPU、默认测试并发。下载只读证据ZIP的 API大小/SHA256、CRC与安全条目通过，测试EXE摘要与原回执匹配，选定原JSON/XML按原字节保存；没有执行下载EXE。全日志gzip还原摘要与原日志相等。仅版本一致不能证明相同负载或失败原因；原件未包含 JS 阶段时间，不能据此归因。

## 有界补充诊断

8be5b4cc7497d08b323839a77548ae189909a3be 只改共享 Windows 查询测试夹具及 CI输出。临时根中记录控制Node、父脚本、创建子进程前后、子脚本、ownership-ready读取和非零退出前的PID/时间，查询结束后读取；CI使用 --show-output 在测试结束后打印已捕获的成功输出，便于与红轮次比较。没有预热/等待Node，不调整生产代码、并发、截止、句柄身份、退出码、无关进程存活或清理断言。标记带来少量额外I/O，本诊断不是性能基准，单次转绿仍不足以确定根因。

本地fmt、Node --check与diff检查通过；Mac不执行Windows专属夹具，Windows编译/运行证据以新CI为准。push CI38507499433最后观察in_progress，新候选打包没有触发。后续先读取新阶段数据，判断问题在脚本启动、所有权接管还是终态退出；只有据实定位修复并形成新SHA通过证据后，才安排新制品与Windows协调单实例验收。禁止盲目重跑或放宽原断言求绿。

## 仍开放的门禁

原 macOS 配对 CPU+1.817629百分点、RSS+117.008965MiB失败保留；冷热每版各≥20、启动p95增量≤100ms且≤5%、10分钟闲置CPU≤0.5百分点/RSS≤10MiB、主线程≤50ms、有效缓存100切页/10重启零自动查询，以及原双平台、安装保留、真实更新/备份/事件、真实周期矩阵不变。Windows局部资源或本轮CI不解除它们。

原TASK182保持3878字节/SHA bc7965b4…、revision2/in_progress/pending，八项required_checks未闭合、acceptance_results为空。回执总体failed，不升级稳定核心事实、不生成完成证书；没有真实数据迁移、日常替换、签名、发布或通道晋升。平台切Mac原生前仍须协调Windows实例结束并确认0。
