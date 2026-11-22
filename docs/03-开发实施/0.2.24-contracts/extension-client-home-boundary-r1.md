# 扩展落点边界：Codex 写 Desktop 自管根，真实 ~/.codex 只读

日期：2026-11-22。适用版本：0.2.24。代码：main 8b450a3fe（基线 6784f89fe）。

## 问题

所有权收缩后，Desktop 仍保留「拓展」页：把 Skill 与 MCP 分发到六个客户端。这条链路按客户端各自拼用户级落点，Codex 一格拼出来就是真实 ~/.codex/config.toml 与 ~/.codex/skills。两处不对。

一是越界。同一份 0.2.24 契约写明真实 ~/.codex 全程只读，拓展页却会往里写 [mcp_servers.<名>] 与技能软链，只有在点动作时才发生，也不是后台自动写，但确属越界。

二是写了不生效。0.2.24 起 OpenCodex 服务带 CODEX_HOME=<OPENCODEX_HOME>/codex-client 启动，运行中的服务读的是那个目录。写进真实 ~/.codex 的 MCP 定义，正在跑的 Codex 读不到。

顺带查出一处仍待处理的问题：目标矩阵的 installed 目前恒为 true，于是点一下 Hermes 的写入，会在没装 Hermes 的机器上新建 ~/.hermes/config.yaml。本次未改这一项，列在遗留。

## 决定

Codex 的落点改到 Desktop 自管客户端根：<OPENCODEX_HOME>/codex-client/config.toml 与 .../codex-client/skills。其余五个客户端保持各自用户级配置，这是「分发到各客户端」的实际含义。

真实 ~/.codex 因此不再是任何可写目标。

## 实现

- ClientTarget 拆出两个字段：anchor 是写入边界的信任锚（Codex 是自管根，其余是用户主目录），home 是该客户端落点所属的用户主目录，用于禁止把源目录、锁文件之类的受管根放进客户端目录。
- managed_codex_root() 只从已准入的数据根记录推导自管根；绑定不成立返回 None。
- client_targets() 成为唯一目标矩阵入口。命令层、技能投射、同步、迁移、模型配置比较、便携资源与可打开路径都改用它，不再各自拼路径。
- 写入边界校验改为「落在各自 anchor 内」。外置 OPENCODEX_HOME 只在锚与数据根记录完全一致时放行，并以聚合根作为逐级 check_path 的根，不做路径猜测。
- home_from_targets() 直接返回 Codex 目标记录的 home，不再从 skills_dir 上溯两级——自管根再往上两级不是用户主目录。
- 绑定不成立时（无数据根记录、锚非绝对路径）Codex 保持真实目录的只读视图，can_write() 为假。

## 一处额外收紧

数据根若直接挂在共享系统临时父目录下，resolve_opencodex_home 会推出 <父目录>/opencodex/home，那是整台机器共用的位置：两个实例会互相覆盖 Codex 配置。managed_codex_root() 对这种布局返回 None，按未绑定处理。正常布局（数据根在自己的聚合目录内）不受影响，外置 OPENCODEX_HOME 也不受影响。

## 验证

- cargo fmt --check、cargo clippy --locked --all-targets -- -D warnings 通过。
- modules::extensions lib 25 项通过，新增三项：Codex 落点固定在自管根、未绑定时只读、共享父目录不绑定。
- tests/extensions 39 项通过，全部改为按自管根断言；并显式断言真实 ~/.codex 未被写入。
- 全量 lib 2177 通过 / 5 失败 / 29 忽略。5 项与基线 6784f89fe 相同（macOS TCP listener fixture 3 项、data_root 的 /private 路径、受系统保护的临时路径），本次未新增。
- 未执行真实业务同步，未改日常配置，未发布。

## 遗留

其余五个客户端仍会重写各自的用户级配置文件（读原文件、改那一个 mcp_servers 条目、整文件重写），原文件里的注释和键顺序会丢。这是既有行为，本版未改；覆盖前会先备份到 <数据根>/backups/。

installed 恒为 true 这一项同样未改。稳妥的判定要看客户端自己的安装痕迹（应用位置或客户端已存在的目录），不能只凭我们即将写入的路径是否存在，否则会自证已安装。需要单独定口径后再动，避免为了修它把「刚装好还没配置的客户端无法分发」这种情况弄坏。

## 真机交叉核对

用仓库内固定的官方 2.99 运行包与机器上真实的 Codex 0.169.1 做了一次交叉核对，不是只看路径推导：

- ocx doctor 报告的 effective Codex home 为 <OPENCODEX_HOME>/codex-client，与 Desktop 传入的 CODEX_HOME 一致。
- 在该目录写入 [mcp_servers.proof_probe] 后，codex mcp list 列出了 proof_probe。

也就是说，Desktop 写进自管根的 MCP 定义，正在运行的 Codex 会实际读到；写进真实 ~/.codex 的做法既越界，也确实读不到。

这条链路已固化为 tests/extensions 的 desktop_mcp_write_lands_in_the_codex_home_the_service_reads（代码 81d823971）：断言写入文件、目标矩阵显示路径与服务读取路径是同一个，并从该路径回读；真实 ~/.codex 不存在也未被写入。

交叉核对使用的临时根为 /private/tmp/ocxd024-extension-proof，核对后整目录删除。

## 追加修正：锁的作用域

把 Codex 落点改到自管根之后，又查出一处独立缺陷：MCP 写入会为全部六个客户端的配置文件加锁，而加锁会连带创建父目录与 .lock 文件。于是一次 Codex 写入就会在 `~/.hermes`、`~/.grok` 留下空目录和锁文件，即便这些客户端没装也没启用。目标矩阵把 installed 恒设为 true，让这个问题更明显。

代码 e84d5e28f 改为按同一份权威配置、同一个 mcp_targets 判断决定锁范围：

- 单客户端写入或断开：只锁该客户端。
- 定义级新增 / 编辑 / 删除：锁当前启用矩阵下的实际目标。
- 配置锁改由调用方持有，覆盖到终态回执；观察者仍能在 completed() 时证明没有第二个写者进入。

回归测试 `writing_one_client_leaves_no_trace_in_the_others` 断言其它客户端的配置文件、.lock 与专属目录都不出现。扩展集成测试因此从 40 项增至 41 项。

## 候选包复核（代码 e84d5e28f）

重新构建并核验，未安装、未改动日常配置：

- 应用二进制 SHA256：9fd9b047b735fef053ed26dbc4ed7be4e2f696d2b2d6b40468778a77a9bbb04e。
- 首次运行供给（无头，隔离 profile，内核拒绝日常 HOME 与网络）：程序根建出，current.json 指向 2.99.0，node v26.8.1、ocx 2.99.0 版本查询通过，ocx doctor 报告 effective Codex home 为自管根，预置配置字节未变，退出后无残留进程。
- DMG：SHA256 51fb72c231e12c84b2abb2143cc3a67c7bfe730b068537e4d04ddb062564d464，273,951,518 字节，只读挂载与载荷核验通过。

界面实屏观察（首次供给完成后的界面状态与截图）仍待解锁补做，该项未计入通过。
