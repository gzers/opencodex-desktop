# r7同次Windows完整扫描：原对象留位与原截止传递

本批沿TEAM19／IMP147／r7既定Windows scanner/source/native消费者写集实施。源码9b3eb595f58590eae71c1aa1277a0e793ef6e6ff为原生完整inventory，20775b54e为Source捕获/复验及Stop原截止传递，64bf7aa73a61be2b50469657bbc7515d486eca00明确成功fixture清理；中间已自行消费main634fc376cda19ca039e9fc30d3be83699058414a，仅NAME/shared模型与页面更改，无本线scanner交叉。代码全部推送codex/024-windows-native，文档分支另交原始失败/来源/状态。原PSS24未知依赖未收到新决定，未阻断独立扫描开发。

## 实现和边界

Windows inventory每次重新打开本次真实目录和全部普通文件，保持同一次调用的原句柄至所有worker join及最终fence完成；文件只共享READ，拒绝WRITE/DELETE/同名替换，目录拒DELETE/路径替换。完整枚举前后及最终children名单、同卷/原fileID、原最终路径、文件类型/reparse/硬链接、创建/修改/属性/长度、所含0x1f7安全描述符、ADS均重核；只读API缺权限/不支持/忙/漂移即Unknown，无降级回旧路径scan。源文件需READ_CONTROL及可建立拒写删读句柄，这是本次不可变原对象集合的真实前提，实际安装/生产组合尚待完整产品验证，不从组件通过推断。

每普通文件仍完整读每个byte、同SHA-256、原EOF增长探测和原总量限制；Entry/manifest/tree摘要语义不变，只有根.runtime-image.json继续按原设计不加入自引用digest，该清单本身仍普通文件留pin并在verify读完整bytes和原image摘要核。嵌套同名文件仍普通内容。100,000成员、64深度、512MiB单文件、2GiBaggregate、最多4workers不变；并未把目录pin当跨动作grant，没有cache、metadata代hash或删掉实际操作边界扫描。

初始同一open内重复属性读取经实际P0计量后删除：每次admission各原生属性真实读取，保留原文件handle，最终所有属性与原基线再次真实核验；枚举结束仍核目录原ID/metadata/children，未删除最终安全/ADS/path/children或任意操作边界检查。安全原始SD仅用于同对象同所含组件的稳定性围栏，不是r4投影恢复证明，不能mint Full/Closed；audit SACL不查询、不签其恢复。此局部原生只读实现按r7允许program_image_windows写集，未改backup/safety、hash、runtime_executable或原生安全共享ABI。

inventory_until/verify_until将调用者原截止传到枚举、所有chunk读及最终对象集fence，不另开时间；Source capture/check和Windows Stop原3个调用点传原end/deadline。任何到期失败不返回部分成功集，已创建worker全部join，panic/错误/创建失败也收束。同步Win32调用不可中断时只能实际返回后拒绝超期，不伪报严格OS中断或生产预算通过。Mac/Unix仍原参考算法、默认None，不改其它平台语义；所有旧full scan调用仍存在。

## 假设、失败与定向结果

library01 E0308/raw101：FindNextStreamW真实windows-sys参数是void*，修精确指针cast；未执行测试。library02旧广范围测试包LLVM OOM/raw101，原日志保留。只对独立test包设置debug0/incrementalfalse/codegen256并--lib，library03新原语9通过（固定副本用例ignored）。后加入权限/更多反例时library05 E0277（夹具闭包捕获raw SD非Sync）和library06再次OOM各保原错：SD只对新自有文件，改有界self-relative bytes生命周期；scanner-only构建关闭test包query feature，不再链接不相干原Owner/query/恢复测试，从而降低构建规模。没有改产品根清单、机器策略、页面文件或日常进程，没有改变测试assert/timeout/预算或编造没运行的断言。

library07/08/09分别12过1败/raw101，唯一失败均是新自有junction fixture的cmd路径语法，尚未测到scanner；三个失败目录、stderr及文件摘要保留，未重试删除。改用新自有目录的真实FSCTL_SET_REPARSE_POINT创建junction后，library10真实reparse拒绝通过。权限DACL更改、ADS、硬链接、same-length byte、写/删/rename替换拒绝、原空目录与新增成员、file512MiB/aggregate2GiB原生sparse、depth65、过期与读前取消、错误/panic4worker join、根manifest绑定与截止均明确断言。最终library17 **15通过0失败3ignored/raw0**；3ignored是显式固定副本实验，前后已有相应独立实际执行结果，不拿ignored签通过。

成功自有Scene现在显式TempDir.close核错误，最终15项都真实清理；旧失败3fixture仍原样保留，原固定官方副本只读复用、未删除/重解包或执行。旧删除审批拒绝未重试；原r2/账号/Profile/PATH/机器策略不动。

## 同源固定副本与性能含义

fixed04首实现完整12,213项/623,970,751字节与原参考BTreeMap及tree306e50d0620aa99b61200ec4c0af8c200fb2f12f3b45ee2068dd85dc35f5fd55逐项相等，但4worker新5.968s对旧5.802s，**没有变快**；原样保留。减掉初始同open重复查询、保留所有前后fence后fixed11同一EXE/同一源/两实现：旧4worker6.916s、新4worker5.908s，旧1worker13.134s、新1worker13.082s。均读完整所有文件/byte，摘要与完整row精确相等；顺序/冷热与主机争用未控制，4worker样本约14.6%下降不是可靠因果加速比或生产/release承诺，1worker基本持平。没有通过省读文件/扩大worker或延预算取得差异。

entry14实际Windows生产入口所用完整inventory+manifest函数体，在固定同源副本的原10秒测试窗口内6366.008ms通过；public verify_until/Source/Stop薄转发未由完整Tauri编译，这只是同函数体/副本/debug结果，**不是完整恢复动作10s、Status5s、Start30s或release生产通过**。主机旧OOM仍留，无同前提完整产品重试。

本轻量宿主机械编译实际scanner模块及同一个inventory/完整manifest函数体，原safety path/open/readcore和clean_absolute/hash都由原源码消费，不模拟Win32成功；generated5段逐字核，40份来源快照对最终工作/producer Git仅CRLF等价核。query feature关闭，实际没编译的Owner/Source/选型typed薄转发另列source-artifact-verification.json，不能把宽源快照当全产品codegen证据。TEST EXE SHA c2affdbb8ffcf0fa0ed7b2d4dd92f1a79758c4e5c8fe53c77dc9a00f69280155，不是产品/发布制品。当前100,000原生句柄/最大容量仍只有原固定代码边界，未做满规模native测试，不冒整版容量验收完成。

## 继续项

13项状态见status.json。P0/本次P1组件与deadline原函数体取得进展；完整Tauri/MSVC产品、release/普通用户真实生产组合及全容量仍未通过。beforeResume原6项/正常Resume回归1项证据复用，partial anchor/原host fresh恢复命令/new origin/实际claim-NLK/完整原件返还仍需接线和中断验证。当前r2完整ZIP/44项预检原证据复用，Job24未知类型仍依赖Mac安全决定，exact plan executionReady/approvalRequested=false，未force/restore/原包重跑。75744/21500/40588、三custody原件/三work/anchors/协调/旧seq9不动，Start/Closed/allAcceptedfalse，不业务模型、不W2/双SID/发布。目标继续active，本组件结果不代表总体收口。
