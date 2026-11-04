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
