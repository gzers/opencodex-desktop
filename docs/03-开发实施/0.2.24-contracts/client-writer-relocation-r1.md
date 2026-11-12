# 官方配置写锁的移动连续性与释放

revision: 1；2026-11-13；IMP-34 P24-04；main e3085eb87。accepted=false，非固定候选。承接[NativeLease交接](client-native-handoff-r1.md)。仅完成Desktop写锁持有者的移动/释放协议；未构造生产根移动许可，未接完整缺根Start/Stop或next Start。

## 原官方API与活体持锁进程

Desktop helper改用未修改官方2.99公开acquireConfigWriteLock(config,{timeoutMs:0})，持有原handle/token；仍不默认等待或计时解锁。没有修改官方源码、原handle或维护包，也没有自行unlink原锁。释放调用官方公开prompt-lock.release，参数只在原活体root fd、原锁fd、准确inode/保护和原字节/token都核对后，使用当前准确物理路径与原token；外来复制的相同token不能代替原inode。

helper保留原root fd与lock fd，检查当前路径和句柄的dev/ino/birth/uid/gid/mode，锁文件必须regular/single-link且原字节完全相同。Rust Guard继续核对原PinnedDirectory与完整Image身份/Protection、当前NativeLease、helper真实PID及未退出事实。目录内容变化不是本协议授予的权限，完整树仍归恢复层核对。

协议2通过有界父管道传JSON命令，结果绑定operation/PID/targetSHA和固定阶段；phase白名单、严格JSON结果、单行512字节与请求16KiB界限保持，无路径或密钥出现在固定阶段trace。普通release无第二层动作；正常成功须官方release后锁Absent、子进程真实退出0、NativeLease仍有效及原deadline未过。

## 移动前登记唯一目的地

prepare_relocation仅供已预存claim OFD能力的Codex holder，先完整check，再登记唯一准确Absent目的地，不能对Grok或普通无移动能力lease使用。父进程保留计划路径，即使回执失败也进入poison状态，后续check不能放行。它只是活体holder连续性，不是服务Stop、文件移动或恢复许可。

真正移动另由owner负责。NativeLease先交接OFD并明确reanchor原root，Guard才能向helper确认同一已登记目标；原directory identity、当前锁完整Image和NativeLease当前物理anchor都须相符。helper独立核对原路径Absent、目标同root与同lock fd/bytes/token后才回复relocated。任一步不明拒绝，不新建目标、不扫描备份、不改取另一个锁。

目的地在移动前已登记，因此管道EOF发生在移动前、移动后但reanchor前、或确认后，helper清理都只核对原位置与唯一目的地。新逻辑根存在时不能匹配原directory身份，清理只释放准确归档中的原锁，不能删除替换根的同名文件。原锁被替换、硬链接或同token复制时保留未知对象及原对象，不为通过而强删。EOF/命令错误的helper退出1，明确是清理后的拒绝；不生成成功回执。

## 验证与失败保留

[证据及manifest](../../../.adg/evidence/0.2.24/client-writer-relocation-r1/)含19常规函数通过（2个子进程探针入口，5显式原包测试不计默认通过）、最终原包7场景与真实父进程exit91、两目标直探、原始失败、来源/库binary和清理。fmt/diff-check、node语法及all-targets严格Clippy通过。

首次原包轮exit101仅报write_guard未确认；后继相同原源码直探定位initial-check拒绝。APFS目录nlink随自己的锁/claims目录项变化，不能作为目录身份/Protection。修正为保留root dev/ino/birth/uid/gid/mode，文件nlink仍严格single。首轮和直探原输出/源摘要不覆盖，修正直探能held，EOF后退出1并清理原锁，不能冒业务成功。

修正原包轮使用固定官方2.99/原Bun、全新隔离Profile，沙箱拒绝网络与日常HOME。7场景：正常prepare→move→reanchor→官方release；登记后EOF；移动未确认EOF；确认后EOF；原token复制到新inode拒绝并保留两对象；新用户根出现拒绝并保留外部文件；非法命令拒绝。EOF场景测试明确等待helper正常exit1，无清理信号；父进程exit91场景由内核关闭管道后释放唯一预登记归档原锁，父实际wait91、helper ESRCH及锁Absent均核对。每场景config原字节保持，最长4738ms（含刻意持锁2100ms），均小于原10000ms。供应/整轮runner90.665s不冒动作预算，未运行服务，不等同完整Start30/Status5/Stop10通过。

两实际helper同时持Codex/Grok锁，先正常释放Codex退出0，Grok原锁字节/活体不变，再正常释放Grok退出0，两配置原字节不改。原包公开withConfigWriteLock在持锁时报告locked、释放后正控可获；失败对象保全证据含合成外部文件与已结束holder的原协调锁字节，不含模型渠道或业务凭据。

最终源码逐文件/实际库binary与修正原包轮source完全对应，binary d3743e9ffa3d0a37a9a3cc74f9bf26281e66fce22c07bd4ac51fd5bf2d8f19c1。verify_evidence.py独立核对场景/源/摘要/首轮失败/global L不变及清理，所有accepted均false。测试配置/替换对象清理由fixture在证据保全与全部进程结束后进行，不作为产品修复/业务成功。

两轮所有进程实际终止，自有Profile/program/resources/test binary和直接probe目录删除；精确各轮新N/K经exclusive重获后删除，预存global L的identity/uid/gid/mode/bytes/SHA保持并重获释放。unitTMP空，逐字归档后删重复临时根。无日常配置、真实WebDAV/模型请求、服务、新.app/DMG/Windows候选或发布。

## 接下来仍须整合的原件边界

完整根恢复必须把本次活体holder引入的临时config写锁与用户原件分清：现在原root-retirement计划要求执行树严格匹配Original，不能直接忽略新锁文件或把它纳入永久残留例外。后继要绑定当前Guard实际原Image作为仅本动作的临时对象，移动前/后分别核对，正常释放后确认其Absent及完整预期树，再登记独立根终态。保留失败现场，不能从磁盘JSON重建holder权限，也不能在未知树上简单解开Ready/capture guard。

之后接当前正常Stop/launcher/TCP、真实原件与原deadline构造不可反序列化生产许可，完成缺根Start/Stop、next Start与明确中断入口。Windows W1/W2、缺Ready/一般客户端图路径/完整进程集合与刷新、NAME/System代理/发布验证/README/固定候选仍属原IMP范围。
