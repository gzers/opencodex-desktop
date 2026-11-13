# 完整镜像路径准入去重 r1

revision: 1；2026-11-13；IMP-34 P24-04；性能实现main3701ee391，收尾测试诊断main0c988fc898fc2c4d495f28945a5479e73ac327d1；accepted=false，非固定候选。承接 [属性优化原包失败](client-xattr-native-r1.md)。

## 完整来源仍在收尾核验

with_admission_mode 的最后 read_complete 已在同次完整 inventory 内核验关键文件，无另一遍关键文件hash；没有可以直接删除的第二份完整内容验证。本批不改该入口或任何摘要，只优化 inventory 的路径准入：旧 safety::check_path(false) 每个分量先 symlink_metadata 只判断存在，随后 inspect 又取得相同metadata并判断类型。新增私有 check_inventory_path 对每个必需存在的分量直接 inspect 一次，保留root绝对目录、strip_prefix、Normal分量、缺失／权限／symlink／reparse／特殊对象拒绝。只用于collect、worker读前、worker全部join后的final目录检查，shared backup safety和Windows独占文件未改。

全部组件仍逐次检查，无缓存／allow_missing／创建许可；文件nofollow、singlelink、全部字节hash、aggregate预算／增长probe／取消和全部join、最多4worker、目录before/after identity/mtime及完整selection最终读回原位保持。少了一次紧邻采样，不声称与旧实现在全部瞬态调度中轨迹相同；稳定状态边界及内容读取前准入保持。每路径深度d的metadata探测仅静态推导1+2d→1+d，inspect调用数本身仍1+d，不能把计量边界次数当native syscall减少证明。

## 测试与官方镜像对照

新增3项覆盖existing/missing/中间普通文件/越界/链接/root边界、同一collect/worker/final流程ABBA精确rows/digest、延迟父目录替换两模式均在进入file-total和hash前拒绝，外部payload未被授权读取。镜像13／selection11／协调27／命令4共55项不同回归通过；镜像1、协调6、命令1项原有忽略保持。最终all-targets Clippy -D warnings、fmt、diff-check通过。独立实际diff只读补审无阻断，未代跑测试。

另外一次严格隔离、全网络拒绝／日常HOME拒绝的官方2.99只读镜像测试通过，未启动服务或打开协调文件。四轮旧／新／新／旧均核对完整13393文件、15646条目、17901路径边界、全部rows和manifest树摘要a29dedc…保持；耗时644.846／522.284／497.956／630.727ms，约20%局部镜像读取改善，不能当作端到端Stop10通过。官方原包b920ba02…／descriptor及bundle摘要不变。

官方只读镜像宿主二进制及两镜像源摘要先行固定；随后新增仅cfg(test)协调tail计量，产品镜像算法字节未变，证据明确分开image-only-tested-source与最终三源tested-source，不冒称前次二进制含后继诊断。后继诊断仅原flag及owned `/private/tmp/ocxd024-versioned-lifecycle-*` Profile记录lease末验、release、完整程序重读与deadline前后余量，不输出路径／配置／认证材料，不改真实执行顺序或错误。协调后继回归及严格检查已通过，但该新输出尚未在原包服务轮执行。

## 证据、清理和后继

证据 [.adg/evidence/0.2.24/image-path-admission-r1](../../../.adg/evidence/0.2.24/image-path-admission-r1/manifest.json) 包括开跑前假设、全部命令／压缩原日志、真实官方完整image ABBA／资源与二进制摘要、实际diff审查、分次源关联、cleanup与独立verifier。官方副本仅公开程序字节和合成安装状态，完整树／资源摘要保持；结果字节比对、进程缺席后清理副本／resources／测试exe，unit TMP为空，全部材料逐字归档后删除证据临时根。没有重复归档整份程序或运行包，没有日常配置／Shell／已安装app／官方源码修改或业务请求。

本批无新官方服务轮，原第二Stop11345ms／RootAbsent18及协调最终deadline失败保持。下一步固定干净源码／release二进制后按新假设一次性取得两轮完整预算和tail分段，再判断这部分有限收益是否足够；若仍超限按全动作阶段批量找剩余可修项，不删完整来源校验或借额外预算。根中断／动作资格／缺Ready／一般客户端图路径和完整进程集合／刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定候选原全范围保持，目标active，未发布。
