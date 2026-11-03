# 0.2.20 验收纠正与弹窗状态复核（2026-11-03）

采用原型仍为 cfbd90c4b + 1da4cabca。安装验收修正为 PR #45/abe20e9f；浏览器夹具和状态验收为 PR #46/52692a54。未改正式产品代码或日常配置。

## 安装器报告的确定性缺陷

Release 41589977450 的源提交为 efbe46e8，工作流虽为 success，但原始 installer-verification.json 只有 MSI/default、MSI/custom、NSIS/custom 三格，缺少 NSIS/default。保留原件为 release-41589977450-three-cells.json，不接受为四格完成。

确定原因是 PowerShell 变量名不区分大小写：foreach 的 $kind 覆盖参数 $Kind，使末尾的all条件不再成立。PR #45 改名为 $installerKind，并在 success 前校验所请求 Kind/Mode 的全部单元格；漏格、重复、失败及超范围结果均被拒绝。没有采用中断前待验证的 ALLUSERS/MSIINSTALLPERUSER 参数方案。

真实 PowerShell 7.5.4（macOS）运行新回归时，改前生产脚本因“Installer loop overwrites a requested matrix parameter”失败。改后 12 项合同检查通过，包括拒绝真实三格假绿报告、9种 Kind/Mode 组合及缺格/重复/失败反例。Windows CI 快检用 PowerShell 7.6.9 也通过同12项，原始回执见 matrix-contract-windows.json。另有55项发布工具/Windows验收回归通过，20个smoke脚本语法通过。完整原生 CI 和正式包安装矩阵仍待完成；CI backend job 不生成 MSI/NSIS，不能据它证明包体矩阵或注册表清理生效。

## 草稿资产与测试包不一致

draft-assets-snapshot.json 保留取回时的8项服务器摘要；其 targetCommitish 为32bb1cb4，tag和被测源码为efbe46e8。更强的字节证据：

| 包 | 工作流实际测试 SHA-256 | 草稿 API SHA-256 |
| --- | --- | --- |
| MSI | f3a278feb35ec15eac88c9cf44c700ba7a93cf051b8f0145561b418bb0477506 | 6e93a20a30beb416013e1322a978e1e7e4d8dd71eb91b22621e665f880eb1f73 |
| NSIS | cefc22e2b4dcad5885cc6e5d6ef98473c71a95053f36f6c69640a416e866414b | a9bb102ce653fb53eb3c020ab7111d877a6779cdd89a9f8a2ebabe0198ed5e6c |

二者均不同。草稿资产不能承接该CI结果。后续须固定最终源码，重建干净草稿，核对最终上载字节/签名，再通过独立四格工作流验证实际下载包。普通账户smoke现在可显式接收包体ExpectedCommit，避免将新验收脚本HEAD误认为旧包源码。未公开stable、未执行真实同步。

此前“NSIS排序污染是已证根因”“CI全绿证明registry清理/四格通过”等表述均证据不足：排序更改后的三格通过只证明custom MSI在那一轮通过；先前AppSearch重定向的完整机制仍未确定，不能倒推出hive来源或安装范围原因。

## 浏览器弹窗适用状态

DEV夹具遗漏discardUnattempted接口，导致受保护导出或取消预览调用不存在的native IPC并出现清理失败toast。旧浏览器检查仍报通过。新增“正常流程不出现清理失败”的断言在原夹具失败，原报告保留于 cleanup-red。PR #46补全显式内存discard适配，已完成导出保留结果用于重试；正式构建不加载这些DEV夹具。

Chromium/WebKit各26项导出/WebDAV断言、26项导入状态断言，共104项通过；40张截图，pageerror为0。每引擎1180×760、900×600，覆盖：默认不加密/加密字段/生成结果、连接字段与测试/取消、原页同步预览、概览导出原页停留/Escape；空文件、受保护文件同窗解密、错误口令不采用、正确解密后预览、默认备份、Escape取消不采用、重开清除文件及解密输入。导出复测宽屏亮色/窄屏暗色；新增导入状态均亮色。三档材质/全部缩放、本机真实文件、备份恢复、真实WebDAV、原生菜单和其余弹窗状态未由这些结果覆盖。

目录中的报告与截图按引擎/流程对应。已目检WebKit宽屏连接窗和窄屏导入预览，标题/动作栏可见，窄屏内容在内部滚动。原生macOS及全部35个历史调用点仍未完整验收；本轮不关闭S20、AC或整体RCP。

## Current callsite inventory

Current source has 37 static Dialog/Modal component references excluding DEV; current-dialog-inventory.json records exact files/lines. Three ModelTransfer windows and the old SyncRoute direct UiDialog are retired, and six new component/host references are included. These are references, not 37 unique runtime modals. The current ledger maps each reference and preserves remaining native/state work.

## 2026-11-03 原生macOS偏好文件流程

native-preferences目录保存真实Tauri IPC与界面流程证据。原包来自本机既有558b9aec arm64候选，严格codesign校验通过后复制到/tmp/ocx020-native-20261103；副本更换test-only bundle ID并ad-hoc重签，仅为区分日常应用。身份、原/副本二进制摘要及嵌入源码见identity.json；本次不是最终tag包体验收。

HOME、数据根、OpenCodex目录及npm缓存均指向该/tmp容器；应用显示缺少Node环境，仍可进入同步配置页。CLI是仅接受version查询、拒绝执行其它命令的惰性夹具。未运行真实服务、API或WebDAV，也未改日常配置。只结束了预先记录的61145号QA进程。

实际执行并按字节验证：

- 仅选择桌面偏好，打开导出预览创建一个未尝试冻结计划。取消后两个冻结文件均删除，exports没有文件；preview-before/after-cancel报告为证。
- 确认明文导出，生成一个1,258字节的.ocxdconf并记录1项“通用偏好”完成回执，保留export-identity与原生截图。包内没有凭据。
- 通过QA界面将主题改为dark，再由原生文件选择器选择刚导出的包。共享导入窗显示通用偏好更新、默认备份选中。第一次取消，preferences.json的SHA-256完全不变，没有新增范围备份。
- 第二次确认更新，落盘appearance.theme从dark恢复为system；持久导入/导出回执均complete。唯一范围备份中的原始preferences字节与确认前SHA完全一致。

仍待验证：备份groups还列出extensions、扩展配置前像不存在，而UI只选择了桌面偏好。包的manager含空扩展配置，可能存在未选范围携带或应用；本轮只证实偏好写入及其精确前像，未据此验收全部范围隔离/恢复。该差异回写S20-03/14继续查证。原生加密、凭据、Skill/MCP、备份恢复、所有入口/主题/缩放及最终正式包仍未由此完成。

PR #45已在全CI通过后合并为ce9ea6e5；PR #46仍待完整Windows CI。安装包和OTA门保持未完成。

## WebKit模型页复测

models-webkit记录32/32结构断言通过，6张截图，覆盖900/1180/1440宽度、亮暗及150%/200%缩放；仅模型列表，不是全部模型弹窗验收。900宽截图有托盘命令通道不可用toast，属于DEV无原生通道的状态，保留原图，不据32项结构通过宣称完全视觉一致。全路由audit在/tmp/ocx020-all-routes-20261103运行中，尚无最终报告；进程句柄81557待回收。

## S20-03/14范围问题已确认并进入修复（PR #47）

回读冻结cfbd90c4b的sync-config.js第31行，桌面偏好定义为“主题、语言与可移植显示偏好”，字段白名单排除执行绑定与本机操作选择。生产UI却把preferences复选项同时映射至manager.extensions，含整份扩展启用配置；这解释了原生偏好包和范围备份中额外的extensions组。

PR #47/1c2ca000修正新导出及WebDAV的偏好选择，已记住的旧extensions标志不再扩大该范围；普通旧文件/下载如含整份扩展设置，须在确认前重新选择为可见范围，extensions=false。投影失败仍dirty并禁用确认；Skills/MCP继续由实际对象选择。原冻结重试保持原操作意图，不借这项修复重写已开始操作。

两项新增UI回归在改前均失败（旧导出extensions=true，旧文件未重选）；改后40项ModelTransfer定向测试通过，177文件/1095项全UI测试通过，typecheck/build/Clippy/rustfmt通过。真实Rust文件链路读取含preferences+extensions的旧包，再以preferences=true、extensions=false投影、范围备份并采用，验证125%显示偏好写入且backup.groups只含preferences；分别验证目标不存在扩展配置时不创建，目标已有配置时字节完全不变。该用例通过，日志随本目录归档；尚未据此声称新UI的原生文件验收通过，PR CI和最终候选复测仍需执行。

## 全路由审计回收

全量浏览器audit为427/428，0页面异常。唯一失败：三档CPU内存中script-time比较high=11ms、mid=3ms、low=14ms；总task-time则high=539ms、mid=25ms、low=95ms。原报告保留于all-routes-after-qa。相同脚本不改阈值定向复跑该用例4/4通过，script-time为high18/mid5/low11ms，原报告保留于cpu-tier-rerun。两轮都保留，不把后一次局部通过改写为全量428/428。本轮DEV夹具/用户状态修正不等于性能全面验收。

## PR #47/#48当前代码的原生修复复核

native-scope-1c2ca000记录PR #47原生自定义协议调试构建（嵌入当前生产UI），独立bundle id和/tmp数据身份。通过真实原生选择器读取上一轮生成的旧包，初始包含空extensions，但UI自动重选生成的incoming计划manager.extensions=null。确认前偏好字节保持，确认后只恢复system主题，未创建extension-config.json；唯一范围备份groups为preferences、前像SHA精确等于更新前dark偏好。这补充了UI→IPC→真实文件/备份链路；仍不是最终release包验收。

相同构建的加密导出默认关闭，手动开启后双口令不一致显示错误且exports为空；匹配测试口令生成protected包，manifest.sections为models/preferences，不含extensions。原生导入错误口令报“口令不正确或受保护配置文件已损坏”，偏好和原备份保持。正确解密后取消“更新前备份”，确认后偏好确实恢复system，新增的备份记录enabled=false、groups/before为空，没有资源前像。但窗口退回“选择文件/读取配置”而未关闭。失败截图及已完成的数据/备份回执均保留；不把数据成功当界面完成。

该确定缺陷由unlockFile成功后未清除pickOpen导致。PR #48/18597c8d只在解密成功后清除此标记，错误时仍停留原解密表单。新增窗口状态断言改前失败（残留两个picker动作），修复分支177文件/1092测试、typecheck/build通过。

native-integrated-f1f26d8f是本地汇合PR #47和#48的源提交f1f26d8f，完整UI测试1095项及build通过、原生custom-protocol构建。全新/tmp身份从真实原生选择器读取protected显示偏好包，解密后确认导入：共享窗口正常关闭、仍在同步页，回执1项complete可查看和关闭。扩展配置不存在，备份只含preferences及精确原始前像。

继续从设置→数据与备份→恢复所选范围执行真实恢复：导入后的system主题恢复为dark，同时保留导入后人工改为125%的界面缩放；扩展文件仍不存在。再生成恢复预览后在同一沙箱文件模拟外部修改theme=light，确认恢复被明确拒绝（“恢复预览后内容改变，请重新核对，未覆盖”），文件SHA不变。原生截图、前后摘要与范围备份原件均存档。只结束本轮已记录的68441号QA PID。测试未运行日常真实同步、读取用户凭据或改日常配置。

PR #46已全CI通过并合并为90c39700；PR #47/#48仍待全CI和最终集成release候选验收。上述隔离调试构建、单资源原生状态不能关闭全部37个调用点、双平台/正式签名包安装或OTA门。

## 原生模型弹窗适用状态补记

同一f1f26d8f隔离构建以新PID71336重启，记录新增渠道空表单/必填校验/Escape取消，Agent未选模型/保存禁用/取消，模型目录未配置/0结果/确定禁用/取消返回Agent表单，批量检查17个内置目标/执行按钮禁用/关闭，外部任务scope预览/默认不允许新增/关闭未复制或提交，GPT内置版本删除和默认控件禁用/搜索无结果/头部关闭，匹配使用无引用/0型号/过滤/Escape关闭。

native-model-dialogs.json及model-*.png持有边界；这些不代表其余执行成功/错误状态已通过。打开外部任务创建了本沙箱的本地任务计划，不作无磁盘写入声明，未调用外部Agent、复制或提交。记录两项差异：未配置目录原文英文，版本搜索焦点内Escape未关闭；继续按S20-19/20修复。新渠道保存、模型执行、默认指针改变及版本删除均未进行。只停止预先记录的QA PID。

## PR #49 原生模型反馈修复复核

模型目录未配置时显示英文、版本搜索输入框内Escape未关闭，已由be695fc5修正。新增两项回归均在旧代码失败；修复后177文件/1097项UI测试、typecheck/build通过。隔离原生custom-protocol构建使用/tmp/ocx020-model-feedback-native-20261103和独立bundle ID；实屏确认模型选择器显示中文安装设置指引，未配置时确定仍禁用；版本搜索输入后按Escape关闭，源页GPT筛选和版本管理焦点保持。native-model-feedback-be695fc5保留截图、身份和完整AX。已停止自有PID73728，没有模型执行或版本变更。此为当前代码定向复核，最终release包与其余调用状态仍待验。

## PR #50：共用键盘关闭与菜单优先级

PR #48/#49 已全 CI 通过并合并为 0dbb781e4、3120d90a4。继续原生检查发现：be695fc5 的模型明细在别名输入内 Escape 不关闭；中间 377fd4bb 虽能关闭输入窗，但 AX 打开选择菜单后焦点仍在外窗，会关闭两层。最终 e1674de7 通过隔离原生复核：Escape 先只关 Fast 菜单，再从别名输入关闭明细并丢弃局部输入；合成 config.json 的 SHA 与前像完全一致。7 项键盘回归、178 文件/1104 项全 UI、typecheck/build/native 构建通过；PR #50 等待完整 CI，尚不是最终包验收。两次原生失败和最终复测均保留。

新增 model-dialog-states 驱动覆盖新增渠道双步与发现错误/等待/取消/迟到结果、复制/删除取消、渠道批量配置/精确采用预览、手动重复 ID、明细菜单/忙碌/错误/取消、型号批量、保存预览及解析失败、Agent 嵌套选择器、版本搜索、匹配使用、批量 Agent 前置条件、版本比较与匹配预览。四组分别为 1180浅高100%、900暗低100%、1180暗中200%、900浅中50%。WebKit344/344、32截图、0页面异常/外部请求；只证明显式 DEV 适配器下的渲染状态。

S20-19/20 新增待修：Chromium200% 模型详情导航链接区被操作按钮挤到不可用；自动滚动不稳定，显式导航入口也被本体拦截。两个原报告各有252条已通过检查，随后 runtime timeout，不能记为全量通过。改前 Chromium 新渠道搜索 Escape 超时前7条通过也不能算成功。相关 report/failure/截图在本目录 model-dialog-*，继续修复，不跳过尺寸。原型仍 cfbd90c4b+1da4cabca；全部 S20/AC/整体 RCP 保持未关闭，正式包重建/四安装/实际 OTA 继续待验。

## PR #51 高缩放导航修复与双引擎/原生复核

9ce6672e（PR #51，基于 #50）修复详情导航独立伸缩区被压至0，以及fitNavigation漏算主滚动区padding/clientTop造成固定后逐帧滚动。侧栏链接与辅助操作共用滚动区，主保存区独立保留。原型cfbd90c4b要求内部可滚导航、主操作可达和固定边界，未改采用范围。新增合同回归改前把scrollTop推至1800、修复后为0；滚动原件记录fitNavigation不断追加13px。浏览器导航尺寸检查改前links=0/oneLink=34失败，修复后共享滚动区保留完整行。

最终完整矩阵Chromium/WebKit各352/352、各36截图、0页面异常/外部请求，报告completed=true；178文件/1105项UI、13项详情/工作区、typecheck/build及7项Windows smoke工具回归通过。指针点击前显式核对目标在视口且elementFromPoint命中；没有使用force/DOM点击绕过被遮挡控件。前面中间版252条通过后timeout、菜单失败、几何失败和连续滚动追踪均保留。导航补丁单独修复链接区后仍有固定遮挡的中间轮次也保留，不将局部通过改写为全量成功。

navigation-native-9ce6672e为全新临时HOME/独立bundle ID的原生custom-protocol构建，持久200%/dark/mid偏好。实屏从模型模版进入GPT编辑，通过侧栏匹配规则打开匹配预览；空ID禁用检查，输入后Escape关闭并返回未修改编辑器。未调用检查/生成/发现，config.json SHA与合成停用渠道前像一致；已停止自有PID83944。原生200%证据不能代替Windows/最终包或其它数据状态。

Windows smoke新增真实WebView2 keyDown/keyUp：输入Escape取消明细、别名不进入草稿、AX等价外窗焦点时先关内菜单、第二次关外窗、文件/备份不变；仅候选0.2.20要求新合同，公开0.2.19基线不误判。PR #50/#51完整CI与此Windows原生新增检查仍在进行；发布资产及安装/OTA门继续未完成。
