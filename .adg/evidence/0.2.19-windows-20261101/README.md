# 0.2.19 原生 Windows 回归证据（2026-11-01）

代码 feature/0.2.19-fixes，最终提交 1fc19df6868dc3c069fb2c24f69a2f74919c7076；原型 cfbd90c4b；Windows 11 build 26100，官方运行时 2.94.0。制品 SHA-256 与检查计数见 summary.json。本目录属于 docs/governance-main，不包含发布制品。

## S18-02

先只读查看日常 app.log 和通知记录，未找到同次页面故障日志；日常代理未做启停复现。沙箱确认同类故障：旧边界要求 y≥28，而 Windows 原生标题栏之外的客户区面板实际 y=1，嵌入请求在创建 WebView 前被拒。修复后只有 macOS Overlay 保留标题栏占位。

- panel-existing.json / panel-existing-native.png：已有代理运行时进入，运行且健康，完整官方仪表盘已嵌入。
- panel-cold.json / panel-cold-native.png：从管理器停止、再启动的隔离代理，约17202ms收敛到 running/healthy，进度 completed=true、error为空；afterRender 为路由渲染完成后的独立确认。
- sandbox-app.log：带源码提交的启动、进程动作、面板创建与 page loading/ready，及原生材质事件。
- 早期冷状态采集约6829ms，超过冻结5秒窗口；它是时序线索，不作为原日常故障唯一根因。首次快照前 Loading、瞬时 unreachable 继续等待及明确 starting_failed 的分流均有回归测试。
- 初次挂载无有效尺寸时由 ResizeObserver 补发 Show；页面失败重试只重载子页面，既有浏览器入口仅为兜底。

## S18-07

原型完整导出包含产品原型目录、跨目录引用的2026-09-28概览动效候选及品牌资源，全部来自cfbd90c4b。浏览器设1800×1000以让1180×760产品窗口与右侧评审工具并列，截图只截产品窗口。最终配对使用默认mesh渲染；早期漏导出iframe造成黑块的截图已全部替换，未据此修改软件或认定GPU故障。

[浅色对照](light-comparison.jpg) / [深色对照](dark-comparison.jpg)：每行左软件、右原型；按 high/mid/low、每档 mica/solid 排列。完整 PNG 保留在同目录。原型图为冻结基线导出的本地浏览器画面；软件图为 Windows 原生窗口或真实 WebView2 客户区捕获。非活动窗口的客户区使用 CDP 捕获，避免其他前台窗口遮挡进入证据；活动图与面板图保留原生标题栏。拼图缩放仅用于查阅。

- materials.json 与 prototype-materials.json：两主题×三档×真实 Mica/非活动实色，12组。DWM读回2为 MAINWINDOW，读回1为 NONE；HRESULT 0表示调用成功。
- material-comparison.json：页面、卡片、通知的填充及模糊72/72一致。页面高/中档 alpha .40，低档白色/深灰实底；所有档位 main 均无 backdrop-filter。
- extra-surfaces.json：设置行、真实下拉和公共弹窗的亮暗三档有效样式，配套 settings-modal PNG。其 fixed 字段为早期查询未命中，不用于固定栏验收。
- sticky-surfaces.json：实际滚动至 data-stuck=true，亮暗高/中档为通知玻璃和8px模糊，低档为实底且模糊none。生产构建已复核标准 backdrop-filter 声明。
- Mica是系统底面，不复制原型示意渐变；原生标题栏高度由系统决定。原型 mock 内容和软件真实内容不同，不把整屏像素差当作材质失败，也不声称整屏像素相等。
- Win10、透明关闭、高对比、DWM能力失败用策略测试覆盖；未改变系统设置实测这些条件。无模糊支持有CSS实底回退，本机支持模糊，未伪造不支持设备。

## S18-01/03/04/05

Windows文件系统上的公共测试覆盖：快照指针和来源、单一预览差异；同名MCP来源选择及凭据边界；同名Skill源目录覆盖、旧文件清理、保留未传凭据；占用目录拒绝覆盖、旧内容完整、释放后重试成功；旧包官方openai过滤与同名自定义渠道保留。传输使用夹具或内存transport，未执行真实跨设备同步。显式原生凭据测试只创建随机夹具账号并验证清理，未替换既有账号。

## 命令和结果

UI目录：

~~~text
npx vitest run --maxWorkers=4       # 174文件，1064通过、10跳过
npm run typecheck                  # 通过
npm run build                      # 通过，保留既有分包告警
~~~

Tauri目录：

~~~text
cargo test --locked --workspace --features integration-test
OPENCODEX_SANDBOX=1 cargo test --locked --features integration-test --test provider_auth_native -- --ignored
cargo build --locked --features custom-protocol --bins
~~~

Rust主库915通过、3忽略；顶层目标合计1032通过、16忽略（不重复计入进程测试自调用的1个子用例）；另显式原生凭据1项通过。Windows runtime 为17通过、4忽略。部分旧网络用例缺少显式环境时提前返回，不能据总通过数宣称真实服务器已验收。macOS记录中的超时不稳定用例带cfg(unix)，Windows不编译；没有把单独运行0项视为通过。

工具链：本机GNU目标，使用父目录 .toolchain/env.ps1 的Rust/llvm-mingw/gcc兼容shim，RUSTFLAGS附加 -C link-arg=<toolchain>/test-manifest/cc6.o；OPENCODEX_TEST_NODE 指向已存在的 Node 22.12.0；Windows runtime测试目录复制了WebView2Loader.dll。此结果不等同MSVC矩阵。一次不限worker的Vitest与Rust并行时组件清单扫描超过5秒，限制4个worker后的完整复跑通过；未调大用例超时。

所有运行在显式 OPENCODEX_SANDBOX_ROOT 下；没有发布、真实同步或日常配置写入。测试程序、临时代理会在取证后停止。父目录 .toolchain、Cargo目标目录及 %TEMP%/ocxd-0219-native-20261101 是本机可选清理项，不属于发布包。

cdp-eval.mjs、native-cold.js、native-sticky.js保留本次本地取证脚本。cold脚本固定检查沙箱名及随机测试端口60385，复用时应先创建独立沙箱并修改端口，不可对日常实例运行。
