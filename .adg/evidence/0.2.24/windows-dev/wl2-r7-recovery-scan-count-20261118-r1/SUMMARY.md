共同输入 main 6bafdd342e69ee83e842f4b35a20bafa1320645a、governance 7cc64ed4ffd0f8b67f642e676290fbcfae7eac84、TEAM48／IMP187／r7。源码 34a766bfd3655036016e1718e3a2919f03fa7f06，核对六个编译输入和 Git blob。MSVC 测试制品 SHA-256 7839ad7b3ec9a1ce01c13d2508e2f4864b91078f397d198e60b281cbb2e66238。首编译 raw101，retirement 和 journal 各报 E0502，日志保留于 compile01。计时回执改为持有自身数据后，复编 raw0，148117ms。计量仅在测试构建启用。三个产品制品复用上一批 37fd73ffa 的编译证据。

整链在原 10 秒截止内执行，raw101、10144ms。160 次完整 Journal 回读与 82 次 retirement 树回读全部完成，.codex/.grok 各 41 次，区间并集 3780540μs。模拟工作阶段有 127 次 Journal 回读，用时 2051446μs。固定 Bun 真实竞争 SQLite claim 返回预期 raw73。首错在最终释放，bounded-original-deadline，底层 raw=None。本轮最终释放、原件历史和关闭未证实。

部分解锁用例 raw101、10039ms。模拟工作阶段第 138 次 Journal 回读首次未完成，原窗口已满，底层 raw=None，尚未触发注入 raw5。计量区间并集 1867474μs。两条 stdout 有逐次时间线。expectedObjects/Bytes 记录扫描预期范围，未完成行无法证明实际全读。本轮通过 0 项，独立历史 raw5 仍按其原制品和范围使用。

两次失败确认了高频完整回读，读回只占整体时间的一部分。现有合成夹具将初始发布、模拟工作和恢复置于同一 10 秒。下一步区分同一次 Journal 核验内的原生对象检查、全字节回读、纯数据重放及未计量阶段。相同执行边界内如需删去重复遍历，先按 r7 提交实际序号和对象等价性证明请 Mac 审查。固定副本 program_image 单扫 2634.174ms 按原组件范围复用。本轮合成证据无法给出真实 Owner 或原包动态扫描次数。

4 个失败根保留，221 条成员、对象 ID 和文件摘要见 measurement.json。生成 schema 经归档核摘要后清理。旧自动审批拒删保留。当前 r2 用既有归档，原 Job 24 项未知、raw50、executionReady=false。普通桌面私有 V2 未收件，Pending 耐久化等待既有交接决定。公开 Start 维持拒绝。原包新轮、W2、双 SID、发布均未执行。累计 251 项矩阵见 status.json。
