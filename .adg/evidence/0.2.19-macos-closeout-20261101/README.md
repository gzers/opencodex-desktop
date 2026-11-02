# 0.2.19 macOS 收口复核（2026-11-01～02）

代码起点 feature/0.2.19-fixes `1fc19df68`；最终收口 `730b9e8b2`（仅 Rust 格式、CI runner 与验收脚本定位补正，未改产品逻辑）。文档起点 `988d8c107`；原型固定 `cfbd90c4b`，从 Git 导出完整依赖，未使用工作区并行原型改动。macOS 27.0.1 build 26A434 arm64，Node 26.8.1。

## 命令与结果

| 检查 | 结果 / 证据 |
| --- | --- |
| npx vitest run --maxWorkers=4 | 174 文件；1064 通过，10 跳过；vitest.log |
| npm run typecheck / npm run build | 通过；typecheck.log、ui-build.log；保留既有动态分包警告 |
| cargo test --locked --workspace --features integration-test | 首次默认并发：主库1132通过，1失败，3忽略；cargo-test.log |
| 同命令追加 -- --test-threads=4 | 完整通过；主库1133通过，3忽略；顶层总计1309通过，14忽略，不重复计入子进程自调用1项；cargo-test-bounded.log |
| cargo fmt --all -- --check | 发现Windows补丁3文件未格式化，格式化后通过；rustfmt-initial.log、rustfmt.log |
| cargo clippy --locked --workspace --all-targets -- -D warnings | 通过；clippy.log |
| cargo build --locked --features custom-protocol --bins | 通过；native-build.log；原生二进制内嵌来源1fc19df68 |
| node qa/geometry-webkit.mjs | 40/40通过；geometry-webkit.log |
| AUDIT_ENGINE=webkit node materials-webkit.mjs | 31/31通过；materials-webkit.json；使用生产CSS的overview-harness，非真实IPC |

首次 Rust 失败与历史登记为同一用例 `metadata_query_rejects_oversize_and_kills_timed_out_process_group`，失败点为200ms超时后夹具未生成child-pid；未放宽用例截止时间。完整限并发复跑通过；不把默认并发失败抹成通过。clippy首次还命中工作树改名前target缓存中的绝对路径（ocx-0218-postrelease）；仅清理本分支tauri缓存后重建通过。

## macOS 原生与原型核对

- 原生Tauri窗口由显式 OPENCODEX_SANDBOX=1、OPENCODEX_SANDBOX_ROOT=/tmp/ocxd-0219-macos-20261101 启动，独立.app容器。沙箱home仅放本机已有Node/npm入口；运行时为本机已有2.84.0包的副本，不复制日常配置。
- 用该副本的官方startServer入口在127.0.0.1:60419启动隔离服务，避免CLI start可能触发客户端模型投影或服务接管。没有用管理器启停日常代理。isolated-status.json确认running与healthy；panel-native.png及panel-98-native.png、panel-98-ax.txt显示完整官方仪表盘来自60419。
- 实屏检查了首次进入、路由返回、面板快捷菜单重载；native-app.log记录creating/loading/ready及reload。98%缩放仍能嵌入，macOS标题栏28px边界保留。缩放反馈出现时原生子视图让出浮层，反馈消失后恢复。
- light/dark × high/mid/low 六份native截图检查设置行与固定导航。native-materials-contact.jpg为缩略图索引，原PNG为实屏证据；各图滚动位置不同，不作为逐像素配对。
- materials-webkit.mjs从cfbd90c4b导出的本地原型获取页面与卡片的有效backgroundColor/backdrop-filter，亮暗三档共24项值比较全部一致，6项能力确认和1项零页面错误，共31项。frozen-material-comparison.jpg为查阅图；背景动画相位、mock数据、标题栏与注释不同，不能宣称整屏像素一致。
- 另试验60420延迟转发：官方页面虽可加载，但转发端口触发官方管理API身份校验，未作为完整官方面板或故障恢复验收；delay-proxy.log仅保留方法线索。瞬时暂停60419服务导致真实健康变化，未作为“健康但页面失败”的证据。失败/迟到ready/只重载页面的确定性分流由panel.test.ts覆盖。
- 早期沙箱OPENCODEX_HOME预设错在active-root子目录，CLI只读status曾观察到10100日常服务并在原生窗口只读显示；校正为实际沙箱/opencodex-home且显式60419后才作为隔离验收。无日常启停、同步或配置写入；该早期状态输出不纳入隔离通过证据，native-app.log保留来源端口以不掩盖这一轮次。

## 收口发现与处置

1. `89dbaa32f`：Rust格式修正，未改变语义。
2. `991c504a5`：GitHub注解确认macos-14处于退役brownout，任务未进入检查；ci.yml与release.yml更新为macos-15，未执行release workflow。
3. `730b9e8b2`：新CI原生Windows安装/失败/恢复后停在旧按钮“导入配置”定位，0.2.19按原型为“导入”；修正测试定位，不改产品按钮。ci-closeout-failures.log保留旧失败。

Windows CDP协议检查14/14、发布工具单测29/29通过（windows-cdp-protocol.log、release-tools.log）。cleanup.json确认本轮沙箱程序和服务均已停止；保留/tmp便于复核。

GitHub CI [41365765185](https://github.com/gzers/opencodex-desktop/actions/runs/41365765185)（同一代码730b9e8b2）已成功：frontend、backend（macOS 15）、backend-windows（MSVC，含原生凭据、托管升级/离线重装、最终EXE、Node24/Node22.20安装失败恢复、普通账户）、release-tools、windows-smoke-evidence五job通过；build因仅main执行按设计跳过。同提交push CI [41365761570](https://github.com/gzers/opencodex-desktop/actions/runs/41365761570)亦已成功，PR Checks两轮五项均通过，main-only build均按设计跳过。PR #33标题/说明由gh更新（连接器仍403），保持draft，不合并。

## 边界

无发布、安装器或OTA验收，无真实WebDAV/跨设备同步，无日常配置写入。既有实现偏差仍在原0.2.19清单，不由本轮材质/面板回归消除。Windows旧证据SHA256SUMS 54项全部匹配。本目录为docs分支证据；沙箱、冻结原型导出和编译缓存留在/tmp，不是发布制品。
