# 0.2.24 Codex树阶段字节源与原生准入计时

revision: 1；2026-11-12。归IMP-34 P24-04，承接[树清单](client-tree-inventory-r1.md)与[原生协调](native-client-coordination-r1.md)。字节归档代码main 39fe025d5d9f0e506e7da03bbec46d9aa6527250；协调修正独立da2fa5939。accepted=false、非冻结候选，实际客户端写回未接。

## 从清单到原字节源

阶段帧升级schema3，新增要求为每个tree普通文件都已有独立SHA256内容对象。Originals::begin和Ready/PreStop/Stopped在发布原帧前调用TreeSnapshot::archive；目录0700、对象0400、当前uid、普通单链接且无未知ACL/flags/xattrs。已有schema1/2拒绝，不迁移、补文件或重拍基线。当前范围仍是有界.codex树；Grok/外部Agent、缺失根及大文件/特殊文件等原未完范围保持。

内容对象名称仅由原树SHA生成tree-content-<SHA>.blob，O_EXCL/create-only。源从原目录FD逐层openat/no-follow，核对原父目录/文件身份与保护；64KiB清零缓冲流式复制并核对长度/哈希，fsync对象和父目录后按原清单读回。完整树前后仍须一致；源中途变化、未知父目录或写出半文件保留现场并失败，不写成功阶段帧。原目录/文件不改权限、不覆盖或删除。

相同内容在同一原件attempt内去重。已有对象必须私有/原长度/SHA完全匹配，并再次同步已核对FD与父目录后才可被新帧引用；存在本身不证明先前fsync成功。损坏/不完整/错误权限/符号链接对象均拒绝而不修。独立帧SHA现在绑定tree内容引用；重开Originals验证每个原引用对象及原元数据，不从当前源文件补缺失对象。archived_tree_file只允许原阶段内登记的相对文件，返回清零缓冲；更改后的源文件不能改变该读回结果。

归档可能包含配置/认证/Agent原字节，这是私有原件存储而非脱敏日志。测试全部是明确合成输入；公开证据仅保存固定synthetic config原件，动态生成内容对象只留SHA/大小/模式/引用匹配表，未输出运行时令牌。树保护属性/路径仍属敏感元数据，不宣称全局脱敏。

claim与精确3sidecars继续只记协调身份，绝不作为普通内容对象打开/存档，也没有新增删除/恢复权限。真实子进程探针确认：空claim及活跃journal锁内完成普通配置归档和读回后，外部BEGIN IMMEDIATE仍SQLITE_BUSY；正常释放后才可取得锁。完整树归档本身不构成进程停止/监听缺席/文件写者许可，MutationPermit工厂与apply保持未开放。

## 原生交接的时钟修正

首轮原包在任何字节归档前被client_coordination_expired拒绝，索引Preparing/revision1，服务未启动；该错误码有多种拒绝来源，不能仅凭它认定唯一原因。源码及定向时钟检查确认一处过度拒绝：Rust从spawn起算两秒，官方helper只在imports后真正held时起算物理持锁两秒，冷导入可能先耗尽Rust窗口。

修正没有延长官方helper两秒，也没有给恢复另开计时。公开with_coordination借用回调保留原保守Rust两秒；私有native_admission固定只复制held准入元数据，沿原动作deadline核对进程/协议/上下文，必须取得released及正常exit0后才返回。随后with_native_coordination仍逐件匹配原三数据库/所有权摘要并重新取得实际NativeLease，真正native回调在此之后才会执行。metadata阶段的child存活不被描述成锁权限；不是忽略过期或拿JSON授予写回。physical helper脚本逐字节未变，原30/5/10秒预算、完整镜像核验及busy拒绝保持。其它client_coordination_expired原因仍须按实际证据拒绝，不能一律认为已解决；之前独立revision4首错继续保留。

## 验证和真实产品边界

99项去重相关检查通过：83项archive/tree/original/restore/session/index，加6项helper准入及10项原生锁（包括无参数probe入口，不当独立反例；子进程probe输出不另计）。覆盖原配置与Agent字节、空文件/去重、当前输入变化、损坏/缺失/权限/链接拒绝、独立阶段重开不补档，以及真实进程在durable blob后exit91的保全。追加已有对象fsync后只做4归档/22原件/1活跃claim归档增量，不重复算新检查。首编测试PathBuf move错误原stderr保留，改借用；一次workdir拼写和patch上下文错误只原工具输出，未改产品判定。最终编译/格式/全目标严格Clippy通过。

原包第1轮失败原目录不重跑；第2轮新profile复用全量核对的相同官方镜像，使用最终算法：Start13760ms、Status1105ms、Stop含阶段归档与计划登记9374ms，原30/5/10秒不变；新collector805ms明确Stopped+Required且三门槛false。四个schema3帧/原件独立SHA与索引RestorePrepared/revision6、plan及原stage身份匹配；11个唯一普通内容对象逐个验证原引用SHA/长度/0400/uid/单链接。真实轮后只有Lease文档注释澄清，算法/测试/输入与最终提交一致，来源差异单独留证。

这些原件已可独立读回，但catalog/cache/audit实际仍未还原，原完整assert保持exit101/accepted=false。Stop9374ms已接近原预算，后续真实apply仍必须计入同一原10秒，不能把本次归档通过当额外时间授权。Grok备份本轮只有缺席断言，不冒称完整恢复；不把config/Grok/Agent全范围缩为当前.codex已支持部分。

## 留证、清理与下一步

.adg/evidence/0.2.24/client-tree-bytes-r1/保留首错/最终/进程突退/锁探针、两轮原索引/四阶段/plan/stage、内容对象匹配表、唯一固定合成config字节原件、各源码/binary、manifest和cleanup。动态生成内容字节未公开存档；不能称证据库含全部生成令牌/完整原blob集。

Darwin完整TCP表及自有镜像列表核对原主端点与进程缺席；两轮精确新N/K/history数据库逐一取得/释放排他后删除。自有程序副本/profile/所有原blob/frames/索引/stage/test binary及核对后的重复回执根已清理，无日常配置/Shell/正式安装/官方包修改，无模型/真实WebDAV业务请求，未出新.app/DMG或发布。

下一步仍是受控进程集合/监听/写者许可及实际补偿、Grok与外部Agent/一般路径和图/显式继续回退，及旧revision4首错。WindowsW0 HOME/Profile/W1/W2、NAME、复杂System代理、官方发布验证与固定候选保持IMP-34完整原范围，目标active。
