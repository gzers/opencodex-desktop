# macOS 0.2.21 原包原生隔离验收

在代码工作树执行；传入的 app 不复制、不修改、不重签、不注入。使用候选自身的进程、WKWebView 和 AX，没有 browser mock 或独立测试 WebView。

~~~sh
python3 test/smoke/ocx021-macos-native.py \
  --app '/absolute/path/OpenCodeX Desktop.app' \
  --out '/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/.adg/evidence/0.2.21/macos-native-20261104' \
  --stage signed-final-reverify \
  --expected-build-commit '<完整的包内构建 commit>'
~~~

debug 预验使用 --stage debug-preverify。可选的 --expected-build-commit 必须与候选启动日志完全相同；不能把工作树 HEAD 当作包内 commit。--scope notice 只重跑保存通知与实际改名读回，同样创建新的隔离根，该 scope 的通过不能代替完整验收。

每轮创建 /private/tmp/ocx021-mac-native-UUID/root。只继承白名单系统环境，进程级设置 HOME/TMP/TMPDIR、OPENCODEX_SANDBOX=1、ROOT、BOUNDARY；CFFIXED_USER_HOME、CODEX_HOME、OPENCODEX_HOME 也位于隔离根。产品在启动时一次解析根与边界，脚本用进程环境、启动绑定、实际配置写后读回核验来源未变，不读取日常配置作为基线。

候选附加 macOS 进程 sandbox：禁止网络、个人 HOME 文件读写（仅允许读取候选包）、系统 Keychain 目录与 securityd/secd 查询。用文档保留地址 192.0.2.1 的内核拒绝回执确认 guard。关闭自动升级检查和自动面板，不激活真实检测、连接测试、同步或确认重启。WebDAV 仅保存合成端点和合成口令，渠道无系统钥匙串引用；不重跑主线 Keychain 改名测试。

Swift helper 校验精确 PID/可执行路径，每次动态定位 AXWebArea，操作前再次核对控件标签；拒绝系统菜单和原生关闭/缩小/全屏按钮。密码框用发往候选 PID 的键盘事件，不使用剪贴板。截图由候选 PID 找到 CGWindowID，再调用现有 capwin helper，不截整个桌面。编译产物仅在该轮临时容器内。

默认检查：

- 权限、可见原生窗口、进程级根/边界、内核网络拒绝。
- 节点检测弹窗居中、状态未检测、header X/Esc；从不按测量按钮。
- 配置对象详情只有 header X、无 footer；模型编辑输入取消后重开无残留。
- 渠道直接改名真实保存；完整 ID、默认渠道、顺序和停用引用迁移，重开干净。
- 保存通知自然 8 秒收起；确认框 X/Esc/取消暂停并续计，不确认重启。真实前台单调时钟与 AX 采样包含 IPC、采样及退场动画误差；后台/锁屏不判通过。
- WebDAV 原生保存重开为“已配置 · 未检测”，空导入候选；概览卡受真实环境门禁遮挡则记 blocked。

每轮立即写 run-context.json、report.json、candidate.sb、签名/哈希、owned-process、进程环境、actions、AX、PNG、计时轨迹和配置前后读回。失败保留截图与异常，继续独立项。空候选证据是新隔离根及本地读取后 AX 无候选行，不宣称观测到了 DOM loading 事件。临时根保留供复查。

退出码：0 为所选 scope 全过，1 为失败，2 为阻塞/未完成。中断不判通过。只清理 Popen 持有的 PID/新进程组，TERM 后必要时 KILL，再检查整组是否为空；发现 PGID 被其它可执行文件复用则拒绝发信号。没有按名称退出 app、pkill、全局隐藏或重启日常应用。

2026-11-04 debug 包内来源 d1eb79c517175c00dc0d5df175ea8ef645630de4，adhoc 签名。后续冻结代码与最终签名原包必须单独复验，不能从本轮推断同 commit。

## 保存通知的计时边界

自然收起与确认退出分别记录。原阈值仍为 6.7～9.3 秒。旧 driver 从等待 Save 通知之后才开始计时，并把“准备点击确认”至“取消后再次完整 AX 采样”的整段操作时间扣成暂停：包含了打开前仍在倒计时的控件查找、以及关闭后已恢复倒计时的读取时间。这会低估活跃时长；旧失败原件保留，不能反推产品计时错误。

计时观察现在在 Save 前启动，只在同一候选 PID 上读取原生 AXFrontmost、AXWebArea、通知按钮及确认框的 role/title/subrole；通过 Python 直接调用系统只读 AX 接口，省去反复启动全树/布局导出进程的开销。没有 AX 写属性/动作、DOM或Pinia读取、产品计时器访问、虚时钟、手动blur、焦点移动或计时重置。该计时改动自身不改变 Swift/input/dismiss；后续独立 Esc 输入改动见下节。完整 AX/截图仍由原工具取证，但它们的耗时不会被当成暂停。

每条观察记录真实 monotonic 请求起止时刻。一次显隐转场位于“上一状态读取起点”至“下一状态读取终点”之间。分别建立通知出现/消失、确认框出现/消失的区间；活跃时长下界＝总时长下界－暂停时长上界之和，上界＝总时长上界－暂停时长下界之和。只有整个活跃区间落在原 6.7～9.3 秒内才计时通过。若区间跨阈值，记为测量不足 blocked；若整个区间越界才记录该计时断言失败。不能只挑区间中点判绿。

原进度条为 aria-hidden，AX 的“本通知将在8秒后收起”是固定帮助文案，不能冒充实时剩余秒数。确认框关闭结果独立于计时结果：Esc 超时即保留失败，既有 header X 恢复只用于继续诊断，不记作 Esc 通过；输入跨度只留作诊断，不参与暂停扣除。--scope notice 只做实际保存、确认退出和读回，不重复无关窗口验收。

## Esc 原生键事件的独立 A/B

2026-11-04 原封6e原包同一fresh app/PID上按 A1/B1/A2/B2 各测两次：A为完整保留原helper（连续post down/up、立即exit），关闭1/2；B为同PID/code53/flags0，down后真实按住至少50ms再up，发送后短暂runloop/等待，关闭2/2，通知均继续。B实际按住55.127ms和51.301ms。A2失败时app active且确认重启按钮仍focused，单列header X恢复，不记为Escape通过。8秒观察窗口不变，每轮只发送一对按键，没有重试、DOM事件或产品修改。

当前 Swift 的 key 操作采用B发送方式，逐post记录 DispatchTime uptimeNanoseconds 的发送前后时刻、PID、code、flags及CGEvent原始timestamp。CGEvent自身timestamp原始值为0，如实保留，不冒称其为实际送达时刻；单调发送时刻也不是WebKit收件确认。原A源码/编译件原样保留，所以A只有host调用边界，没有伪造内部post时刻。证据位于 macos-native-final-6e/esc-input-ab。此为有界的native输入时序差异证据，不声称修复产品间歇故障或证明所有设备键盘行为。该A/B不重跑计时；实际关闭/焦点和失败截图独立留证。
