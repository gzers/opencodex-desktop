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
