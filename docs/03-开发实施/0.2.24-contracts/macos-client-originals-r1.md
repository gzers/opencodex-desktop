# 0.2.24 Mac客户端原生原件及持久阶段记录

revision: 1；2026-11-11。归IMP-34 P24-04，承接client-artifact-policy-r1；代码maina19669114fc5e05e60c56cf6bf78df0ad686e5c1。只完成原生快照/原件留存，不代表实际客户端恢复或最终候选，accepted=false。

## 代码输出

infrastructure/macos_client_originals.rs提供Mac真实FD快照及Originals留存。客户端只读固定三个对象opencodex-catalog.json、models_cache.json、opencodex-catalog-audit.jsonl，不从记录中接受任意路径，不读config凭据/其它客户端。每个文件捕获原字节或明确缺席、dev/inode/birthtime、mtime/ctime、uid/gid/mode/flags、原生扩展ACL及全部有界xattrs。对象不可读、非普通文件、硬链接/符号链接、外部owner、超限或读取过程中变动均错误，不能降为缺席。

PinnedDirectory从/起逐层openat O_NOFOLLOW/O_DIRECTORY并保留原FD；每次使用重新对比所有路径组件的身份和安全元数据，不能只比文件hash或拼路径。原目录改名后用同内容新目录替代仍拒绝。文件内容两次集合读取及发布前/后原件检查一致才完成留存；这不是可替代官方写锁的瞬时监视器，真正写盘仍需对应协调/原生写前核对。

Originals在当前D的manager-state/client-originals/<UUID>/原件目录保存original.json→ready.json→pre-stop.json→stopped.json。目录0700、文件0400、当前实际uid，无扩展ACL、无未知权限或属性；创建使用O_EXCL，不覆盖已有阶段或损坏半文件，每件写入fsync并同步父目录、写后原生读取核对。保留frames绑定schema1、当前uid、原D/CODEX_HOME目录身份、attempt、阶段及前一frame摘要。原客户端owner/权限/ACL/xattrs不修改，原件字节不重新编码为产品配置。

macOS会给新建对象自动添加com.apple.provenance：只在私有日志准入中允许这一不超过64字节的OS溯源属性，并保留捕获值。它不替代owner/mode/ACL验证，不能据此承认其它扩展属性；未知journal属性原样保留且拒绝，不自行移除。目录FD保护读回仍会比较已捕获值。

## 摘要与重开权限

Frames的hash不提供写者/进程归属授权。Originals::open必须由外层事务显式提供原attempt、原件摘要及各阶段已独立保留的摘要；不能仅因某个阶段文件存在就从该文件重新生成信任。缺摘要的阶段不可读为已承认状态，不能作为后继阶段前件；不能重新封存original或跳阶段，也不能覆盖已发布但尚无外层回执的未知阶段。

本批没有启动自动恢复。真实子进程exit91后，父进程按先前回执的原/ready摘要重开通过；只传原摘要时，磁盘ready仍保持但拒绝读为可信阶段。原件留存可以被外层生命周期继续消费，但外层持久保存摘要/正常退出/官方claim/N/K协调/客户端写前CAS/原件应用及回退尚未实现。两个reader对同一源拍照不代表写者排他，不能以这些私有JSON为正常Stop或文件补偿证书。

## 验证

18项真实Mac文件系统定向检查通过：原字节/原权限及stage顺序、阶段摘要重开、真实子进程突退、半文件保留、同字节inode替换、目录身份/权限变化、symlink/hardlink/FIFO/目录/不可读、源发布前外部修改、真实原生ACL/自定义xattr保持、私有数据/日志权限及未知属性拒绝、日志祖先链接不写外部目录、重复JSON/篡改原件、超限、根重叠/缺目录拒绝。OS sandbox禁止网络、日常HOME读取和夹具外写入，唯一外部命令chmod仅用于自己新建的合成ACL测试文件。没有服务/真实同步或模型请求。

首轮编译7项SDK/libc字段名及测试trait错误记分类记录（当时原stderr未捕获，不宣称留原件）；后继变量遮蔽编译错误原stderr保留。原16项测试12项失败：实际OS provenance被误判、chmod把数字UID当name无法译UUID；用自己新目录的独立原生诊断确认属性后修正，仅允许上述OS属性，真实ACL测试改用当前uid经getpwuid解析的用户名称，源保护断言不放宽。修后16项通过，补两项未知日志属性和祖先链接反例后18项通过。

严格全目标Clippy首轮要求测试权限字面量0改为0o000，原失败保留、不禁lint。最终Clippy/格式/编译通过；18项原生运行源与最终源码的唯一后继差额是该测试字面写法，数值/算法一致，记录独立源摘要，未为纯字面改写重复整组原生测试。实际算法源码摘要一致，不冒称当时native binary嵌入后继提交SHA。

## 证据和当前限制

证据.adg/evidence/0.2.24/macos-client-originals-r1/保留各次编译/Clippy/原生失败与最终结果、原生provenance诊断、实际运行源摘要、清理及manifest。自有测试子进程已结束，复制测试二进制/夹具目录及核对后的重复临时回执已清理；未改日常目录/正式安装/官方包，未重构.app/DMG，也无Windows原生执行。

当前CODEX_HOME必须已经存在；缺目录明确拒绝，不能默认为空目录或在读快照时创建。后继正式首次启动要单独记录原目录缺席、其受控创建及可证明的撤销边界，不能因此要求用户手动预建。仍需config/Grok/Agent范围、真实原生生命周期代次/外层摘要登记、官方协调helper、客户端原子补偿/CAS及完整持久失败恢复，才能关闭上轮三个文件未还原问题。Windows平台继续独占Git原语/安装器/PATH贡献，当前本模块仅macOS条件注册，不冒称跨平台实现已齐。
