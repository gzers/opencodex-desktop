# 多代根来源、闭合审计与目录集合 r1

revision: 1；2026-11-13；IMP-34 P24-04；main ef00dd58c。accepted=false，非固定候选。承接[第二代捕获](client-root-successor-capture-r1.md)。本批支持第三、第四代及有界后继的存储滚动；没有接 Active / Start / 状态资格，也不把合成存储闭合当正常服务 Stop。

## 单一来源与历史

新增纯 root_lineage：从同一持锁 SQLite 连接读取的 canonical events、singleton binding 与当前 Head 解析全部代际，不从目录补 origin / scope / 摘要。首代固定 Preparing1；后续每代 Preparing 必须紧邻前代最终 RootAbsent，核前驱 revision 与完整 Head 原编码 SHA。ticket / origin.id / generationSlot 一致且唯一；同 UID、data/profile/HOME/逻辑客户端路径与身份保持，新客户端根 identity 不可复用任何旧代。

所有代内事件逐步核阶段、不可回退的 Original/scope/live/phases、既定恢复决定，rootGeneration 只在最初 Preparing。重复闭合仍是同一固定 plan/source/readback，所有 RootAbsent completion 必须一致，不能仅检查末次而漏掉早先回执篡改。保留 4096 events、4MiB history、单事件65536字节、512代的界限。

每份闭合归档按所属代 origin、scope、源事件、首次 plan 登记、最终 Prepared 前驱及读回核验；历史检查不使用当前 .codex/.grok 内容。整代归档与目录投影后重复核验，通过同一连接重读全部历史/binding/Head，复查 storage 和 action pins。closed_root_history 可返回多代事实，不授予 Start 或创建权限。

## 目录集合与上下文

- 正常观察/重开/关闭历史：client-root-generations 必须准确等于完整索引登记集合 S。
- 新根创建未登记：仅持活体 create-only Reservation / Transfer 的当前动作核对 S 加自己的 pending p。
- candidate SQL 后及提交：从候选全部事件重算 S′，必须准确等于原登记加本次 p；未知名字、缺失、链接、权限或 provenance 变化均拒绝。
- client-originals 仍准确等于已闭合代加当前阶段允许出现的唯一 attempt；部分发布不能自动采用为 Original。

References 的私有 namespace 是仅内存的事实上下文，serde 跳过，不改变持久五字段或旧字节。持久相等比较仍只比较原五字段，目录集合每次单独实查。正常上下文只能从封闭、只读的 Lineage 派生 typed IndexedNamespace；Transfer 私有 seal 防止外部 struct literal，创建 token 不由磁盘反序列化得到。缺上下文的裸引用只保留原空/单 slot 严格检查，不能假定它掌握完整多代集合。

当前 resolve 保留 enriched origin；Frame 使用 owning Originals 的 origin；Plan 在持久 origin 相等验证后继承 Originals 的上下文。provenance 尾、每帧尾、整 prefix 回调后和独立归档文件读取返回前再核名单。这检测检查边界内持续存在的变化，不声称消除任意外部并发最后瞬间竞争。

创建前先克隆原 action open-file-description 并完成 S 审查。Reservation 持续核 S+p；事务内旧归档审计只接真实私有 Transfer，先要求其上下文准确等于事件 S 加自身 id，临时装饰不逸出，专用入口只返回空值。SQL 后仍完整核候选、旧归档、新 provenance、空根/原 created fd、父目录 fsync 和原 deadline；失败事务回滚 event 与 Head，已提交失败保留事实与文件。

## 验证边界

最终118个唯一受影响函数通过（新增rollover8、旧索引/捕获56、根归档37、首代原件9、根创建8；4项ignore不计），strict all-target Clippy、fmt、diff通过。

证据 .adg/evidence/0.2.24/client-root-lineage-r1/ 保留开跑前假设、当前10源与实际 library-test binary 摘要、各轮日志、四代完整原始文件字节、三个第三代 exit91 现场、离线 SQLite/JSON 核验与清理记录。八项新增场景覆盖四代来源/事件/归档不变、裸 JSON 无法恢复上下文、代际/阶段/前驱/身份篡改、异常目录、末帧后变化、重复闭合回执和提交边界 unknown 不登记。

真实子进程分别在第三代 published、SQL Head 更新未 COMMIT、COMMIT 后退出91，父进程 wait91。离线核验将完整未提交 Head 与 predecessor.json 比较，已提交来源 SHA 与原前驱 SQLite 字节绑定；所有旧代原件/scope/plan/readback 与本代 provenance 逐字核验。此证据不是断电、官方服务或固定应用候选验收。

尚需统一只读后继资格接生产 Active/Start 与状态、明确中断入口，并至少两完整 release 原官方包循环/受影响负控。缺 Ready、一般客户端图路径、完整进程集合/刷新、NAME、复杂代理、Windows W1/W2、版本发行验证与固定候选等 IMP-34 全范围保持，不因本批 storage 通过而关闭。
