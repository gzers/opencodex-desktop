# 已索引代次目录元数据扫描 r1

revision: 1；2026-11-13；IMP-34 P24-04；main287ee0c9d2a2048b83f8db54d945caedf02e29b5；accepted=false，非固定候选。承接 [资格同源接线](client-stop-eligibility-r1.md)，本批没有官方服务轮次。

## 元数据扫描与内容认证边界

verify_generation_slots 原先对每个已索引目录执行 child_private＋private，多次重复其祖先链及child的元数据采样，期间不读取任何计划／来源／归档记录。本批改为私有check_private_children：父链原private完整准入，各child仍以目录型nofollow打开、鲜读初始identity及完整Protection并要求原private策略；随后两轮named stat／held FD stat对照固定初始device/inode/birth、目录类型、uid/gid、完整权限位、flags，并每轮鲜读ACL/xattr等完整Protection对照初值。

所有child检查后再完整父链recheck，两轮鲜读和父链检查原语保持；逐child句柄释放，不累积511组祖先FD。helper不返回pin／内容／token或写入许可，数量上限511。外层canonical UUID、排序去重、exact expected names的前后枚举、generations.private及manager.recheck原位保留，之后才进入原有来源记录读取。后续每帧Plan／provenance／archive／namespace、原件／读回／proof及source/candidate/postcommit／deadline未删改。

这减少了共同祖先以及child自身的重复metadata采样，不声称原生syscall序列、所有瞬态竞态轨迹或采样时刻完全等价。稳定状态校验和内容读取前完整准入仍保持；namespace目录合规本来就不认证其旧inode，独立来源记录的身份与摘要门槛继续负责真实代次绑定。范围仅macOS，不冒称Windows reparse／mount策略新增支持。actual diff只读审查无阻断，未代运行测试。

## 反例和完整历史对照

两项目录测试覆盖empty集合、multi-child、512数量拒绝、missing／非目录／链接／非法组件和不修权限；首／次child在open后和first-pass后，以及末尾父链fence前的mode／ACL／xattr／replacement／symlink变化均拒绝，现场保留。

四个独立同形完整四帧首代历史fixture按旧／新／新／旧对照第二代真实登记代码，frame/archive48、capture339、provenance61、namespace60、完整历史begin/finish各3及内容双遍6的次数全部相同；Protection61744→54373、native xattr103631→91157，减少约12%。

初步ABBA耗时518.581／452.202／444.752／528.253ms，最终补全ACL／次child／对称计数断言后样本641.435／584.410／572.563／682.168ms，均保留原日志，不挑较快初步样本对外推断。最终局部收益约12.6%，不同宿主负载有明显波动，不能据此声称完整Stop10已达标。样本时间不设新放宽预算或替换原生断言。

## 回归、证据与后继

完整原件模块315项通过／9原有忽略，协调29项通过／6原有忽略，版本化命令4项通过／1原有原包忽略，共348项不同受影响测试；先行三项包含在相应模块计数内，不重复计入。最终all-targets Clippy -D warnings、fmt、diff-check通过。完整模块先于仅测试ACL／次child等扩充开跑，产品helper／其它原模块用例未变；最终三项定向已用最终源码重新编译执行，对应source／binary范围单独注明。

证据 [.adg/evidence/0.2.24/client-generation-directories-r1](../../../.adg/evidence/0.2.24/client-generation-directories-r1/manifest.json) 包含开跑前假设、初／终全部计量及压缩原日志、五源与测试二进制摘要、审阅边界、范围说明和独立重验脚本。unit TMP为空，全部原字节压缩解压逐字核对后删除证据临时根，工程target／旧原包失败保留。无日常配置／Shell／已安装app／官方2.99原件或真实WebDAV／模型请求变更。

Windows精确远端代码仍b2e89665d、文档0eef59a01，无新推送，不推断未推的本地工作。下一步需继续以完整历史／关闭事务评估累计成本，再以新假设固定release／原包整轮验证总预算及资格接线；不将本批模块通过或局部下降当作原包验收。原第二Stop11428ms／pending19失败保持，根中断／缺Ready／一般客户端图路径及完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选原完整范围不缩减，目标active，未发布。
