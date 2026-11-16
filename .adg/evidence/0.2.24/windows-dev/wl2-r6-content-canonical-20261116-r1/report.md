# r6原内容资格进入同一canonical链

源码 d29dbc36d567d4344407a07d652311f2bfa140d9 / tree 033892db91a7624aa5762dacb60c83a4e070feb2。承接ed8929154/a81796823；TEAM16/IMP142/r6及fetch共同源不变，仅Windows允许集。

| 运行 | raw | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 101 | 50/1/0 | 307720 |
| library02 | 0 | 48/0/0 | 161553 |
| product01 | 0 | —/—/— | 227149 |

首轮50过1败/raw101：新增测试前置恢复数据的快照引用以Value顺序编码，原typed reference摘要校验正确拒绝；仅修合成测试构造，不放宽生产断言。原失败EXE另留私有目录并校验，87场景/原源码/日志保留。末轮48过0败；复用首轮未受影响纯Bun/sourcepin结果，未重跑空GUI/旧raw5。

新ContentQualificationIntent/ContentQualifiedVerified保存实际private assessment的scope/series、原Ready/PreStop/Stopped上下文/6帧及三固定顺序动作；SnapshotLinkedVerified只在原canonical阶段更新严格阶段序号/上下文hash。结果必须匹配intent完整数据，pending阻断其它事件，完成后不再回工作/阶段/新action。current只增加有界资格摘要，完整四阶段+恢复/history/handles为3490字节，原4096上限不变。

生产内部入口在同一actual typed Stop/source/owner/routes/NormalExit/Absent/claim/绝对end下先写intent，重跑原完整BOTH-tree内容屏障，再写verified/current，之后才capture_retirement。恢复每段和history完成再核原native assessment与canonical资格摘要。DATA解析不构造Stop/claim/Owner/assessment或Closed许可；原shared policy/原生内容算法、源pin、预算和公开Start拒绝不变。

真实普通桌面原包Owner/资格/恢复仍仅编译未执行。后续完整原件handle inventory/所有权关闭、timely completion与typedClosed、同次5秒Status、dispatcher/后继、官方config/auth/Grok/动态目录副作用、首次初始化/ownerreceipt/原缺席Codex安全退根、普通MSI/CAP-trust/历史清理继续未完；全部继承状态归final-status，不将组件通过当整版验收。

本轮172场景私有ZIP按原字节/全membership核验；仅2完成纯库/pin场景清理，170失败/pending场景及56个D根保留。1700份原canonical/快照/history文件另入raw-scene-records。ZIP非NTFS身份/安全克隆，历史拒绝删除未重试，overallCleanupComplete=false。

最终 184 产品输入 / 138 Rust实源逐字核对工作文件及Git（明确CRLF等价）；产品custom-protocol MSVC debug仅编译，不运行服务/同步/模型。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77804544 | 0f7791bb948b1e64c5fa9a43ae79db12364d2a32872363d24d1281b149242042 |
| ocxd | 5398528 | 0e7275099dcc4ea2dfaca23a386c3b67c662e031fbcaf0969f8557c78fdb3452 |
| ocx | 306176 | 2c6c25153938af7e5efd73d13952216c65375d30a35b340b80f5f0edbfd922da |
