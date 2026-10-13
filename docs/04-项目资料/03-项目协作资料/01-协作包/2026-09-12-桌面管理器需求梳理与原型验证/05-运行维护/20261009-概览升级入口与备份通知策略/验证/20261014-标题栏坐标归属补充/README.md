# 标题栏坐标归属补充与上一工具 CI 终态

2026-10-14，Asia/Shanghai。只修改诊断工具和后台测试，不改产品代码或窗口状态。工具最新为 feature/0.1.10-caption-diagnostics@5a36cda04018fb367c2f77e3a2a1bdc22d991e03；parent=04c5f54d05d50243a83925511dd8f3ccbc4ca951。

## 为什么补充

对主 HWND 直接查询 WM_NCHITTEST 返回 HTMAXBUTTON，只证明主窗口对该坐标的回答；没有证明这个坐标下的实际窗口属于主窗口。工具现在为每个按钮内部坐标并列记录 WindowFromPoint、GA_ROOT 和目标家族类名，另查询主窗口命中。遮挡 / 子窗口 / 坐标偏移均仍是待查原因，不能凭这些查询声称 click 已投递或软件缺陷。无关窗口仅记录“是否存在 / 是否属于目标”，不保存其句柄、类名或进程身份。

原工具04对同一份新增测试：11项通过、1个断言失败、4个错误（缺point_target/字段）；新工具16/16通过，测试文件SHA一致，原红日志保留。这是工具合同补齐的对照，不是产品原生故障复现。所有新增API路径仍用fake Win32后端测试；没有在Mac执行Windows API或发送输入。committed-results.json的源/测试摘要与5a Git blob一致。原身份、250ms查询上限、负屏幕坐标/溢出拒绝、只写新证据文件和无验收结论继续保留。

只读坐标来自观察器PMv2物理空间；UI工具截图/点击坐标可能使用另一空间，必须记录截图范围、DPI/DPR、实际输入坐标及转换方式，再查询相同物理点。查询按顺序发生，不是原子瞬时快照。历史不同观察器的窗口边界数值不能不加坐标/时间条件直接比较；相同DPI不证明相同观察器坐标或RDP显示条件。

## 源码与 Windows 回写

Cargo.lock钉住Tao0.37.1与tauri-runtime-wry2.12.1；两份本地源文件与crate归档逐字节一致，归档SHA与锁文件checksum匹配，见dependency-source-review.json。Tao的WM_NCLBUTTONDOWN发DecorationsClick并保留默认消息派发；Wry的Windowsresize子窗口只在resizable且未装饰时接入。实际窗口样式/消息路径仍须真机证据，静态依赖审阅不证明还原原因。

远端文档b3eb4274已快进同步，原未提交状态保持。Windows独立复核原TASK字节/三摘要/九项AC绑定/八项checks通过；be7395a/PID10904正常窗口、DPI192、只读2/9/20符合旧测试允许的2或8/9/20。原期望最小化8的辅助脚本差异已由Windows更正为工具合同差异，原件保留；这不证明真实最小化点击。最大化还原、实际WindowFromPoint和新工具Windows绑定仍待验，下一前台仍需新的人信号；不启动第二实例。Windows原件见20261014-Windows分工与原TASK复核。

## CI 的实际范围

旧工具04的push CI38429545855已completed/success，五个必需job成功，main-only build为skipped。原完整日志883556字节/SHA25613445e7479fd19547baac5534d58bd69da721b820d9c845f150ca453db45fb1d留外部；prior-ci-caption-excerpt.log是按step名称筛选的21行衍生摘录，不称原件完整日志。日志独立汇总MacRust34组/872passed/6ignored，WindowsRust34组/601passed/7ignored，失败0；ignored不计通过。该CI运行旧12项工具合同，不能证明最新16项或新Windows API通过。

新5a的CI38431616359最后为in_progress，ci-runs.json为观察快照。不重跑/取消旧run，不手动触发候选/签名/发布。两工具提交均未修改apps/**，main仍eae85322，焦点PR仍be7395a草稿；现有be制品可用于本次原因定位，证据分别绑定工具源码与实际应用制品，不能当新产品源码验收。

## 门禁保持

Mac无桌面应用启动或新增原生性能样本；原CPU+1.817629百分点/平均RSS+117.008965MiB失败不解除。原性能、缓存、真实周期、双令牌/物理DPI、安装/数据保留、更新/备份/事件及发布门禁保持。TASK revision2/in_progress/pending与八项required_checks不改，acceptance_results为空；FZ50脚本未获许可不执行。无真实迁移、日常替换、自动合并、核心事实/通道晋升、关闭证书或公开发布。
