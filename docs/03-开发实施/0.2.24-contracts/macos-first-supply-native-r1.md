# 0.2.24 Mac实际应用首次供应原生检查

revision: 1；2026-11-11。归IMP-34 P24-01/05，承接macos-engineering-build-r1及integrated-preparation-r1。代码main 45f813d799662a36b51fc0033dc9c7cef0a7f527（产品状态修正）及7be8d2658e98882b2c0ceb3a146b8b3977a9c746（原生入口）均已推送。accepted=false，不将工程输入当最终集成候选。

## 验证目标与真实输入

此前工程.app使用OPENCODEX_SANDBOX显式测试身份，按产品合同跳过自动供应，只证明首窗渲染。前三次macos024-first-supply.py使用原已密封工程.app，binary SHA fda508bfbe7ac3e6ee71512c827d1aeb07ecdd0e0280295e8d9034afef26ea20，来源09392b38e+version-source.patch/a58f03cb9，早于80afa4418代理后继。第四次使用45f813d79干净产品源码新构建真实.app，binary SHA d398c4e3f67b8ead28fe17f6595cd96731fc5cab32518302d3e6ec1abfcd00eb，包含代理后继和状态修正；原启动日志明确嵌入完整45f813d799662a36b51fc0033dc9c7cef0a7f527。构建约247.544秒，严格codesign通过；没有重建新DMG，旧39782f66…工程DMG不能冒用为本.app的制品。两者均非冻结候选。

脚本复制.app至自己的/private/tmp唯一Applications目录；不去签名、不改Info.plist/应用字节。使用新私有HOME/CFFIXED_USER_HOME/TMP/CODEX_HOME，不设置OPENCODEX_SANDBOX或其根/边界，运行程序由正常生产启动解析至私有HOME/Library/Application Support/OpenCodeX Desktop/runtime。程序根起初不存在；只初始化当前v2配置根和偏好以禁用自动联网、同步、面板/客户端启动，合成配置无模型和凭据。这是新profile/无runtime输入，不是“所有配置文件也完全为空”的新用户覆盖。

偏好字段归属从当前产品DISK_SECTIONS及固化默认提取，结构元数据匹配当前structure_version=2/inside，字段集完整且无重复，0600原件。原应用bootstrap/owner/路径/单例/供应/校验/激活/环境检查全部照常执行，不通过假current/manifest或mock环境完成供应。

## OS隔离与原生观察

精确sandbox-exec拒绝日常用户根读写、系统Keychain、外网及全部网络；用192.0.2.1连接和日常.zshrc读取只测试内核拒绝，不读取实际内容。没有同名应用操作，不改日常安装或Shell。AX/CG观察器固定实际Popen PID及确切副本exe，跳过系统菜单，不遍历其它窗口/最近文档。

观察并保存实际AXHeading当前阶段和每行已完成/进行中/待执行值；current.json实际出现后继续等真实应用环境就绪页面，不仅凭文件存在宣告通过。核对实际程序位置、current及镜像摘要/官方归档身份、package2.99.0，再用已供应的原Node/npm/OpenCodex只执行--version。版本探针cwd固定私有根，避免继承被拒绝的工作区；无服务启动、模型或同步操作。配置原字节保持且runtime-port记录不存在；观察失败不算依赖通过。窗口截图只截该PID窗口。

## 失败证据与清理

attempt1预先放入preferences却未初始化当前v2数据根，产品以nonempty unrecognized directory拒绝，原件保留；这是测试输入遗漏，非产品应放宽的限制。attempt2补齐v2输入后实际供应及环境就绪已观察，但独立npm探针继承工作区cwd，OS拒绝process.cwd导致exit7；该失败没有转记成功，也未修改产品。后继将探针cwd明确为私有根并保留前件，再复验真实链。

每次结束持有原GUI Popen身份，仅SIGTERM自己的测试进程组，若有残留必须列原PID/PPID/PGID/image，组空才删除自己的副本.app/程序目录/配置/助手根。失败和成功均保留各次summary、命令、AX、阶段及截图（能取得时）；不得删除工程原.app/DMG或用户数据，不覆盖前次失败。自己的测试供应镜像属于明确新建夹具范围，可在验证后整体清理，不能套用为生产repair-history自动删除政策。

## 当前范围

attempt3已通过实际供应→UI环境就绪→Node/npm/OpenCodex版本读回，约86.2秒，配置保持，无服务记录；但是截图发现服务状态仍残留供应前“未发现安装/接入后可启动”，本轮没有把这个显示问题藏进供应通过。后续状态修正和第四次真实.app复验已经完成，原失败与发现保留。本轮只闭合无系统Node、新用户私有程序根的实际Mac供应与UI→依赖版本证据，不包括真实服务启动/30秒预算、状态5秒、正常Stop10秒/客户端还原、产品升级回退/卸载、WindowsW0/W1/W2或名称剩余项。工程输入不是最终集成候选，最终仍需最新干净树固定制品与受影响原生验收。


## 原生检查发现的状态显示缺陷及最小修正

已供应且环境通过后，原状态采集器如果只收到服务查询错误，仍会保留供应前的NotFound。原程序来源此时已经resolved，原缺失结论被事实推翻；不能沿用，也不能把查询失败改成Stopped/Running。main45f813d79补充该一处转换：resolved=true且旧状态NotFound而本次查询失败时变为Unreachable，health未知、PID/端口清空；已知运行态的瞬时失败保留、来源缺失变NotFound和来源不可信处理保持。

19项状态回归包括Timeout/Parse/Unreachable×轻/完整采样，再收到真实Stopped成功和后继卸载回NotFound均通过，格式/严格Clippy通过。第四次真实.app复验等待环境通过且当前hero不再显示“接入后可启动”，历史最近事件保留不当当前错误。实际链约127.647秒通过，77.694秒观察到current、82.105秒Supplied、85.855秒环境门消失，后继真实采样收敛至“未运行”。这是成功采样后的Stopped，不是注入查询失败后的Unreachable；失败回退由上述19项单元检查证明，原生检查不冒充覆盖了该分支或瞬时刷新/5秒预算。Node v26.8.1、npm11.19.0、OpenCodex2.99.0均真实输出。

## 证据结算

证据位于.adg/evidence/0.2.24/macos-first-supply-native-r1/，保留四次summary/AX/截图、可取得的各次runner、真实构建/签名/状态检查及输出路径负例。所有夹具副本.app、runtime、配置和助手根已移除，GUI仅向自己的进程组SIGTERM退出，组内无残留；逐字节核对保留件后已移除重复临时回执，manifest重算。未改日常配置、官方包或安装应用，未运行真实业务同步/模型请求或服务启动停止。私有profile仍是当前操作系统账户，不当真实新SID证明。
