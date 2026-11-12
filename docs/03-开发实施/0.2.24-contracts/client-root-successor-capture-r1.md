# 第二代根的专用捕获与独立原件登记 r1

revision: 1；2026-11-13；IMP-34 P24-04；main d4af2abb2。accepted=false，非固定候选。承接[第二代创建与原子登记](client-root-generation-r1.md)。本批补第二代 CaptureReserved / schema5 Original / 独立 scope 登记和存储层归档闭合；不把合成 storage close 认作官方服务 Stop，不声称生产再次启动已接通。

## 来源、活体保管与登记

新增 `reserve_successor_root_capture` 和 `record_successor_root_originals`；首代入口仍拒绝第二代，普通 `reserve_capture` / `advance Ready` / `reserve_start` 不开放根缺席流程。

第二代 reservation 要求当前准确 Preparing、原始同 action 保管的创建 fd、未过期原 deadline、空根、generationSlot=origin.id=ticket，以及初始事件内 preparingRevision / 紧邻前驱 RootAbsent / 原编码 SHA / 当前 binding 解析。重开 Index 只取事实，不能从磁盘文件重新构造 created fd 或 deadline。

Preparing→CaptureReserved 时不再复制 rootGeneration；来源永远保留在最初不可变事件，由 ticket 解析。Original 登记依据本代 active binding 比较逻辑 scope，并核原件 SHA、startup-scope SHA、HOME/根 identity/逻辑路径及 Original 引用；不覆盖首代 singleton binding，也不修改旧 events、原件和 archive。

CaptureReserved / Captured 的发布在原 SQLite 连接、原 action owner 和 CAS 下进行，SQL 前、SQL 后完整候选核验后及 COMMIT 读回后检查原 created fd / deadline；reservation 还要求根为空。提交前变化回滚 event 和 Head；提交后变化返回错误并保留已提交事实，不清除原件或外部根。

## 历史与当前仓库职责

第二代初始 Preparing 之后逐事件核相同 ticket、允许的阶段边、不可回退的 Original/scope/live/phases 和既定恢复决定；其它事件不得复制 rootGeneration。无原件的 Failed 没有本批准入。

旧首代每份闭合 archive、来源及读回仍完整核验，整个 vault 的目录名必须准确等于已闭合 attempt 加唯一已登记且已出现的当前 attempt。Preparing 不准预存当前目录；CaptureReserved 可无目录或留部分发布；Captured 及后继必须有独立 references 和目录，再经过 `verify_capture` 完整 frame/blob/scope 核验。未知 UUID、额外文件、符号链接或旧归档变化不跳过。

当前 attempt 的投影只重复核对 private pinned directory 身份/权限/stat version/直接子名；不把部分 blob 当原件、不宣称其内容有效或完整稳定。创建器仍为 create-only，已存在的部分 attempt 不可自动采用。聚合枚举使用 `4阶段 × 2种树 × 16384 + 1024` 的独立有限条目预算；原客户端单树、单文件、总字节预算均不变。

## 验证与证据边界

最终112个唯一受影响函数通过（第二代12、旧索引44、根归档37、首代原件9、树10；5项显式ignore未计入），含13个新增函数；严格all-target Clippy、fmt及diff检查通过。

证据目录 `.adg/evidence/0.2.24/client-root-successor-capture-r1/` 保留开跑前假设、失败日志、最终源码/实际库测试 binary 摘要、三份实际 exit91 现场原始文件字节、SQLite/JSON 离线核验、四个发布边界反例和清理记录。

正例证明新代独立 Original/schema5/原缺席、本代 scope、首代 binding/归档不变和事实重开；第二代 storage close 保留两份归档，第三次 Start 仍拒绝。反例覆盖过期/错误 fd/新用户文件/过期源、预存或部分 attempt、未知或链接目录、旧归档变化、错误摘要、摘要正确但身份/路径/Original 语义错配，以及 17MiB/16385 项部分仓库不能冒作原件。

实际子进程在 reservation 后、Original 发布后和 Captured 提交后 exit91，父进程 wait91；重开仍拒绝重新取得 capture 权限。离线只从归档原始字节验证 SQLite 事件/current Head、初始 binding、前驱关闭/摘要、本代 provenance、Original/scope，不冒充断电验证或 OS 服务验收。

尚未接 Active 生产捕获、第三代及更多循环的完整 history/slot 集、统一状态资格或明确中断恢复；缺 Ready、其它客户端图/进程集合/刷新、Windows W1/W2、NAME/网络及整版固定候选等仍按 IMP-34 推进。下一批先扩展各代独立闭合审计和 indexed slot 集，再统一 Start/status 资格及生产接线，最后跑至少两完整 release 原官方包循环及受影响负控。
