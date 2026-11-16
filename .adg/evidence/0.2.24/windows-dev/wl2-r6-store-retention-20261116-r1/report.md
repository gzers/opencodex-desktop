# r6释放对象分离与首次变更前v3 Store保管

TEAM16/IMP142/r6，源码eee0ad1a49442f6369373d34bf22ea610f1b1af4/tree741d26814a4b122944c15c7de5bf07bf3faec8c7；承接229bcb934/532105f72。Windows独占范围，原trait/预算/Token/8投影/未知拒绝不变，不改官方或日常配置。

| 运行 | 原退出码 | 过/败/忽略 | ms |
| --- | --- | --- | --- |
| library01 | 0 | 5/0/0 | 201792 |
| library02 | 0 | 27/0/0 | 115889 |
| library03 | 0 | 2/0/0 | 125719 |
| product01 | 0 | None/None/None | 226557 |

本轮无非预期原生首错。5项新定向通过；受影响旧kernel ranges/各角色busy/expiry/byte drift、lease分段、v3进度及new capture保管回归27过。静态接线复核发现组件journal只接受D直接子目录，与生产D/official024/windows/UUID不匹配，原代码会在任何客户端移动前拒绝；修严格生产命名空间祖先pin及native UUID同session绑定，再两项定向通过。不捏造生产失败执行；仅实际源码缺陷。旧raw5根移动/空GUI/1314、无变化官方读取/安装器不重跑。

释放后类型分离：NativeLease仍持PhantomData<*mut()>和活OVERLAPPED范围，保持非Send。只有原来全部UnlockFileEx成功、范围空、原4文件/parent/Token/ID/8投影/字节/sidecar读回后，准确原LockFile对象及Bound pins按所有权移入ReleasedObjects，里面没有NativeLease或range/OVERLAPPED，不复制/重开句柄。facts在拆分前后同值；真实线程移交历史receipt+读回+返回后，主线程仍须实际新取得才有活lease，新epoch且单用，不能用迁移历史对象授权重复获取。全程无unsafe impl Send或令牌/ACL修补。

Held原来单独Drop阻止直接移出原pin，现由NativeLease统一保管已登记Held；尚未登记的Held取得失败时先明确尝试其自有partial ranges unlock，再按原错误返回。失败cleanup关闭自有文件不算普通释放成功，收据仍必须显式原生release。NativeLease原check_inner/release方法体token保持基线，原范围/分享/忙立即失败/原deadline/poison语义不变。RoutePin capture/check/declared方法体同样token一致，只把可见范围限定到同Windows原件模块供新Store复用，原共享route解析未改。5份token等价与active marker/无unsafeSend记录归owned-boundary-equivalence。

原生CapturedTrees核心：完整原树+actual journal在序号0先进入Option Store，再发anchor/file意图与native动作；18记录准备3份普通原件与工作副本，20含显式释放后，准确历史文件/pin、全原件树和同journal可以实际跨worker并返回。事件/current失败导致0原件移位，native首次move后失败留下1准确原件custody；所有树/journal被poison，Store仍可定位，不替换/追赶/自动重试。这里是合成native组件，没有fake Source/Owner/NativeAction或production Ready/Closed。

生产v3 CaptureSlot已注册在原WindowsOriginals中。prepare_v3_capture要求原Owner/原ActionWindow/Source/current/image/route/完整NativeLease；拒绝旧v2/native历史混用，D/official024/windows任意旧条目（含文本Closed）均拒绝新会话，不扫描接管/迁移/覆盖。Source与完整双原树/plan/native journal形成CaptureStore后先入原adapter槽，才开始首文件移动；Source/route/未Resume原Owner前后重核，完整工作后像核验后才Prepared→显式释放并留回执。活lease始终在栈上，失败时Store/原句柄/current/前缀保留并显式清理自有活范围。v3已有/失败尝试会阻止旧v2 capture fallback。

新production prepare方法仍未实际调用，原capture dispatcher/Ready及后续v3完整接口尚未切换。生产journal严格D/official024/windows/UUID，准确private祖先/目录pin，manifest/current沿该准确UUID；错目录/非UUID拒绝。直接D/session仅供原组件，actual Source factory不接受。编译器验证CaptureSlot/Source/route/全树/journal/ReleasedObjects自动Send，不能据此签生产通过或强行发活NativeLease。现有公共Start无条件拒绝，不以synthetic Binding/hash代实际Source资格。

剩余：正式v3入口/阶段/全snapshot writable系列、源绑定官方受控副作用/原共享policy、config/auth/Grok非产物声明范围、动态产物/新目录准确退休还原、完整NormalExit/监督/端点/record/source/typed Closed/原history/后继和实际普通Owner-H-S-D。原缺席Codex初始化及安全退根仍blocked，不能留目录/两锁冒Absent。普通MSI/CAP-trust/全部历史清理未代签。不W2/双SID/真实业务模型请求/发布。

产品MSVC custom-protocol debug bins和最终产品输入/testRust对工作/Git源码字节核验，前端32份复用；非release预算/冻结候选。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77731328 | a907409f06fb3c69eb93e64309f84f4c8912a70aec723a067baf155611d6f62b |
| ocxd | 5398528 | ce8fddda08c8091795646051412c8673eced08ec535988b071a54c98e0874edc |
| ocx | 306176 | fcfa35593a48440789c56d031d20b1c09e9428e9337f54865dc11cbe05954258 |

77场景私有内容ZIP核验，D canonical/source snapshot原字节另归raw-scene-records；11处已完成release历史/纯数据场景核全部文件和目录后清理，66失败/pending原场景和29 D日志根保留。入Store原生20事件正例仍缺正式Closed，保持pending。ZIP不是NTFS身份/安全克隆，旧删除拒绝/账户/Profile/Public/旧broker均不重试，overallCleanup=false。
