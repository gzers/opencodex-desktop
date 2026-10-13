# Mac 源码审阅、旧性能证据复核与标题栏诊断

2026-10-14，Asia/Shanghai。Mac 仅做后台源码 / 单元与离线证据复核，没有启动桌面应用、占用前台或采集新的原生性能样本。Windows 保留 be7395a / PID10904 的交接安排；是否仍存活须在 Windows 实际核验。

## PR #1 与已有真机证据

PR #1 已附加本任务，head=be7395acfce0ec1e36b83300b576db408f8c35bb、base=eae85322ebca3b755c034db97cbebf2ea5bf71fc，open / draft、未合并。五文件补丁中，焦点 watcher 在视图重绘后仅对仍打开且焦点外逸的弹窗恢复同标签控件或 dialog；保留已有内部焦点，关闭后不夺焦。另三份 Windows 测试诊断仅添加 cfg(test, windows) 阶段 / 就绪和 CI 宿主记录，2s / 4s / 5s 原预算不变。未发现此有界补丁需要阻止继续真机验证的源码问题，不扩张为全更新功能审阅通过；旧 d975 CI 失败原因仍未隔离。

在精确 be7395a 代码 checkout 复跑 update-center、overview-motion、motion-performance、mesh-performance、overview-mesh-center、overview-glow-host：6 文件 / 52 项通过。记录见 feature-review-ui.*；这些是模拟 DOM / WebGL 与黄金帧回归，不是实际 WebKit 性能。原 Windows FOCUS-NATIVE-RESUME 清单 48 份逐项 bytes / SHA256 再验证通过，原件未改，见 windows-focus-evidence-verification.json。

## caption 还原尚未定位

旧原件证明同 HWND6685560 / 普通用户 / DPI192：最大化前 false；caption 还原动作后仍 true / 相同边界；Alt+Space 系统菜单还原后 false / 恢复原边界。AX index96 标签已是“还原”，但动作记录没有实际点击物理坐标或 WM_NCHITTEST 返回值；不能据此判定 UIA 目标偏移或产品缺陷。生产没有自定义还原点击处理，window_appearance subclass 先调用 DefSubclassProc，仅在设置 / 主题 / 激活 / 大小变化后排队刷新材质，不吞系统命令。源码事实不能替代真实消息投递证据。

独立分支 feature/0.1.10-caption-diagnostics@04c5f54d05d50243a83925511dd8f3ccbc4ca951 新增 test/smoke/windows_caption_probe.py 与 12 项工具契约，CI 的 release-tools 增加便携测试。该提交只改测试 / CI，产品运行代码与 main/eae 和 focus/be 的本次审阅路径逐字节一致。没有合并焦点 PR，也没有制作 / 签名新产品候选。CI38429545855 最后记录 in_progress，未宣称通过。

工具对显式 PID / HWND 核验归属、EXE 完整 SHA256及可选进程创建时间，观察标题栏按钮屏幕范围、三个内部物理坐标的命中结果、DPI / 样式 / DWM / 窗口状态，前后再验证身份。只在观察器线程设 PMv2 并恢复原上下文，避免 RDP / 负坐标屏幕的查询虚拟化；不改应用或系统 DPI。Win32 查询各限 250ms，失败保留错误与空值，坐标溢出 / 无效范围不绕回其他位置，输出只写新文件。没有输入、激活、ShowWindow、SC_RESTORE、启动或退出。工具测试覆盖这些边界，未在 Mac 执行 Windows 原生 API，Windows 绑定本身仍待实际运行。

Windows 后续先核实现有实例身份，然后在同一实例最大化时保存工具快照；人提供操作窗口后，从新截图对应的真实坐标点击 caption，并记录实际输入方式 / 坐标和新快照，再独立比较 UIA 目标点击与系统菜单。HTMAXBUTTON 只能证明被查询坐标的命中，不能冒充 click 已投递。未获得人的前台信号时只读，不抢前台。若 PID / HWND / SHA 不符则停止该观测，不能自动启动新实例。示例仅对应交接身份，使用前仍须核验：

```powershell
python test/smoke/windows_caption_probe.py --pid 10904 --hwnd 6685560 --expected-sha256 3a26098e813ddd56070a0092cbcb640473cb73e626aee45198c4b335ec2d8bd1 --output caption-before.json
```

诊断工具源码与被观察应用的源码 / EXE 身份分别记录：该工具可观察原 be 候选，所得结果只归实际 be 候选，不称新产品代码验收。后续如修产品源码，必须另制新 SHA 候选再交 Windows；旧制品不能证明新源码。

## macOS 性能失败离线核验

对 20261012-deb740-十分钟性能对照的原归档运行既有独立 recompute，显式输出到外部新文件；结果与旧 comparison.json 逐字节一致。两轮各 61 样本 / 600s；CPU +1.8176288611467015 百分点、平均 RSS +117.00896516393442 MiB、结束 RSS +220.671875 MiB，均超原预算。performance-recomputed.json 为本次重算输出，原失败比较不改；offline-performance-diagnostics.json 从同一原始采样提取进程与时间点，未新增原生样本。

候选 WebContent RSS 103.953125→309.468750 MiB，基线 94.750000→95.296875 MiB；候选主进程 RSS 118.968750→119.015625 MiB。已有 sample 的 SVG mask 链仍为线索；source-comparison.json 核验 Logo/SVG scene.ts 在官方 0.1.9 与 deb740 中相同，新增差异主要为背景光 Teleport、正文层叠 / 覆盖尺寸和 Logo 锚点观察。mesh 连续帧不读布局，ResizeObserver 只读布局变化，现有测试不代表真实浏览器重绘 / 分配行为。这些信息用于缩小下一轮受控实验，不能证明具体泄漏或布局因果。

旧对照仅一组顺序测试，准备态，缺连续前台 / 请求与长任务仪器；因此保留失败但不下因果结论，不用 Windows 局部资源样本解除。切到 Mac 原生前必须协调 Windows 实例结束并确认实例数为 0，再按同机器、同版本 / 画质 / 状态 / 分辨率 / 前台和充分预热重现。先控制 CSS / mesh、静态 / 动态和布局差异，单变量实验只作定位；最终源码必须保留顶部 / 左右光层覆盖及软件三档，不以降低画质或预算求绿。确定可复现差异后再修生产源码，最后完整重新配对。

## 门禁保持

原 TASK 已在 docs/governance-main@ae6cd080 同步原件，revision2 / in_progress / pending不变。冷 / 热每版各至少 20 次，启动可交互 p95 增量 ≤100ms 且 ≤5%；10min CPU 增量 ≤0.5 百分点、RSS ≤10MiB；无 >50ms 主线程任务；未过期缓存 100 次切页 / 10 次重启零自动网络与查询进程，以及真实24h / 6h / 恢复 / 并发 / 通道矩阵均不变。八项 required_checks 未整体闭合，review:scheduler-performance-0110 failed，验收结果空。FZ50 生命周期脚本未获许可不执行；无真实迁移、日常替换、自动合并、签名、发布、通道晋升或完成证书。
