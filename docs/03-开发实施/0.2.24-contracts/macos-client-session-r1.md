# 0.2.24 Mac启动原件与实际就绪／正常停止代次绑定

revision: 1；2026-11-12。归IMP-34 P24-04，承接completed-stop-binding-r1、macos-normal-exit-r1及native-client-coordination-r1。共享原生绑定实现main4b6943ca71cc240d00feb6cda944e5db06c7fdf5；先行独立helper修复maind85f9d1d3（完整SHA见证据summary）。当前实际绑定链通过，自动恢复/整版验收未通过，accepted=false、非冻结候选。

## 会话与真实输入

macos_client_originals::session::StartupSession按原Originals创建startup-scope.json。原件捕获位于实际NativeLease内，begin还要求当前受控运行来源的原生归属查询明确Stopped；不把runtime-port缺席当安全启动许可。scope绑定原attempt/Original摘要、当前uid、实际Desktop数据根/工作用户目录/OpenCodex HOME/CODEX_HOME路径与各原生目录身份、原程序根及完整CurrentSelection、原捕获时间。原件及scope均create-only、0400、私有原目录fsync和写后核对，不修改官方配置。

NativeLease新增require_context，验证同一runtime Arc、实际profile、HOME和client root；其原生profile/HOME目录持续重核。只取得了另一组官方锁或拥有同一目录字符串，不代表可为当前启动拍快照。后继source只做已准入当前program selection/来源的观察及前后记录一致性，不为每个引用重复全镜像扫描；外层NativeLease的整个操作前/后完整程序树核验仍是执行准入。

bind_ready只在真机新鲜private runtime记录、实际Generation和既有SystemServiceOwnerProofProvider确认的当前用户、管理程序image/argv/HOME归属通过后执行。对同一个原HOME完成HMAC health→ready→health、原runtime记录字节及原进程出生代次读回，再复核同一原生owner事实。只有ready=true、原PID/port/HOME、原版本/程序来源一致时，才封存原Ready frame及startup-live.json；后者只保存原scope/Ready摘要、受保护runtime记录SHA256、PID/uid/出生秒微秒/端口，不保存secret/token/Key或原runtime记录。

References给外层原事务保存attempt/Original/scope、已封各阶段摘要、live摘要。open只接受独立传回的这些原引用；scope/阶段/live文件改动或目录/程序/HOME变化拒绝，不从当前磁盘或扫描目录生成新基线。现有持久JSON本身并不成为活进程证明：seal_pre_stop重新验证实际签名/Generation/原生HOME归属及原记录；没有实际当前进程便不能仅解析live JSON推进阶段。

seal_completed_stop要求本轮实际CompletedLifecycleStop与scope的当前context/D路径、live PID/uid/出生代次和原记录摘要完全一致，并受原Stop截止时间约束；原停止后runtime-port/ocx.pid必须明确缺席，真实NativeLease仍核对同context。原Stopped frame封存后再次检查Stop绑定及记录缺席；AlreadyStopped无新退出/摘要，不能代替这次服务正常退出。尚不构造MutationPermit、不写回客户端文件。

## 本轮实际发现并修正的两个缺口

### 实际客户端目录必须与启动环境一致

首轮native fixture／协调器使用profile/codex，但生产ControlledProcessRunner清空继承环境、只给用户HOME，没有CODEX_HOME，官方原样Mac路径实际为profile/.codex。用原官方getCodexHome只读命令及精确已知私有运行PID的内核三键HOME/CODEX_HOME/OPENCODEX_HOME投影核对证实；未留完整argv/environment。不能把单独传给协调器的client路径当运行进程上下文。

Session现明确按当前受控Mac启动策略核对canonical(profile/.codex)，另传的client路径拒绝。首轮服务实际readiness失败保留；该错不被笼统归因为所有readiness失败的唯一原因。日常进程环境/当前用户配置从未改动，测试失败服务仅原image/record+双challenge同连接HMAC绑定后执行官方POST Stop，reply200/performed、原镜像进程清空；这份清理不是原轮启动/验收通过。

目前缺失.codex目录仍拒绝，不在读会话时猜空目录或隐式创建；正式新安装的目录缺席/创建/撤销合同保持未完，不能要求用户手动补目录代替产品支持。其它客户端根同样须与实际未来启动策略核对，不因这个检查而悄悄取消已定支持范围。

### 首次成功的官方N初始化不能被helper回滚为空

第二轮默认目录修正后真实Start通过，Ready绑定却被官方N拒绝。独立只读检查本轮精确新数据库为0字节/user_version0/无table；实际原官方构造器报CodexCoordinatorLegacyAmbiguousError/nativeResidue。根因是helper在干净目录调用官方N构造器初始化后无条件rollback，留下空N；服务随后合法写入native集成journal，下一次构造便正确拒绝在残留状态上重新发明初始行。

独立提交d85f9d1d3只对原本缺席／确实0字节、实际官方构造器已准入且本轮三锁工作成功的首次N事务调用官方commit；commit前要求原/当前nativeGeneration均0且currentTxId均null、assertStablePath通过。schema/初始行仍由原样官方构造器产生，没有复制SQL schema/迁移旧格式、没有接管非空旧DB。已有N和失败callback保持rollback/close，记录及锁保护保持。该修正使首次捕获→服务实际启动→后继官方重开可以连续成立，不把先前拒绝改为成功。

## 验证

7项Session引用/类型/未知输入检查与15项NativeLease上下文及相关协调检查通过；后者含一个跨进程探针入口，其余实际检查逐项保留。Session主要验证原scope/阶段摘要独立重开、跨profile/HOME/client/program拒绝、目录代次替换/篡改保全、重复或不安全runtime记录、持久live JSON不能冒充endpoint、出生时间极值及实际客户端目录核对。初次5pass/1fail暴露u64极值出生秒可解析，补i64时间域上界；原失败保留。Clippy PID字符串比较首错保留，改用有界数字解析而非禁lint。最终编译/格式/全目标严格Clippy通过。

原样官方2.99新私有profile的实际轮次分开保存：

| 轮次 | 实际结果及边界 |
| --- | --- |
| attempt1 | readyness失败；客户端路径输入与实际清空环境不一致。原轮完整保留，精确私有服务随后官方停止清理，不补通过。 |
| attempt2 | Start通过，Ready绑定官方N拒绝；实测0字节DB及nativeResidue原拒绝定位helper初始化回滚。原服务再按确切image/record/HMAC官方停止清理，不补通过。 |
| attempt3 | 修正helper首次官方初始化持久化后，实际原件→Start→Ready→引用重开→PreStop→正常Stop→Stopped引用匹配通过；启动含绑定13444ms、停止含绑定7695ms。此来源尚未包含最后加的Ready前后原生managed/HOME归属门，单独保留。 |
| attempt4 | 最终准确源码：加入Ready前后实际原生managed进程归属校验，完整绑定链通过；启动含绑定13675ms、停止含绑定7833ms，原绝对30/10秒保持，正常Stop仍沿原HMAC/OS/kqueueexit0门。 |

attempt3/4复用首轮已校验且未改字节的官方完整运行镜像，每轮重新验证来源/全目录，不重复约70秒供应；每轮新的profile/协调DB/原件/引用，未在失败原件目录覆盖重跑。最终runtime package2.99.0/tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c，descriptor e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3，准确编译源文件及binary摘要留存，最终代码算法与实际测试字节一致。各轮前先有要证伪的假设，无调宽超时/减少原生核验。

**最终仍是passed-binding-only／accepted=false。** 三件原始客户端字节与停止后比较仍false，测试只证明正确会话和真实服务Stop能关联，并未实现文件补偿。该测试由显式原生宿主调用实际Session/协调/进程接口；应用普通commands::execute_lifecycle还没有自动创建或选择Session，也没有当前活动会话持久索引/失败恢复UI或生产MutationPermit工厂。不能用这个技术整链取代日常产品入口完成状态。单PID正常退出也不代替完整控制集合/所有监听、config/Grok/Agent/全客户端树恢复。

## 留证与清理

证据.adg/evidence/0.2.24/macos-client-session-r1/含四轮原错／成功、实际默认HOME解析与限定三键内核上下文、官方N拒绝/零DB检查、各轮准确source/binary、原件scope/live及四阶段frame、独立原引用和全部大小/SHA manifest。成功两轮6件frame分别对外层独立references核对摘要和真实0400/当前uid/单链接条件，不保存runtime secret、admin token或config凭据。此前失败服务的官方清理回执单独保存，不能混入normalStop验收。

自有测试/helper/服务镜像进程已空；精确新协调DB分别独立取得/释放SQLite排他后删除，成功轮N schema仍为官方初始化版本/表事实另存；完整Darwin TCP表framing核对前失败私有主端口缺席。全部副本程序/profile/复制binary和逐字节核对后重复临时回执根已清理，没有官方源码、日常配置/系统Shell、正式安装或其它用户对象改动。OS sandbox拒绝外网/日常HOME，仅loopback、自有根及精确新HOME摘要对应的N/K/history数据库/sidecar及N采用临时文件可写，未开放整片账号namespace，无真实模型请求或WebDAV业务同步，未出新.app/DMG或发布。

下一步是把Session持久引用接到实际应用生命周期当前会话写者、失败／崩溃恢复和真正客户端发布工厂。原始范围继续覆盖config/Grok/Agent/全树、新目录/活跃刷新/跨卷/flags、WindowsW0 HOME/Profile/W1/W2、NAME、System代理、发布验证和固定候选；本批不能以只处理三件或只有测试宿主调用的较窄状态结束整版目标。
