# 精确版本更新说明、核验网页与独立重试

2026-10-14，Asia/Shanghai。代码 feature/0.1.10-release-details@6305747f035945f3563296b42d21edcf5c816464 已提交推送，父提交be7395acfce0ec1e36b83300b576db408f8c35bb，包含父分支已验证的焦点修复。main仍eae85322，PR#1仍open/draft、未合并；caption工具仍独立5a分支，本分支未吸收它。没有启动桌面应用或原生性能测试、切换Windows候选或执行官方FZ50生命周期脚本。

## 已复现的实现缺口

原面板详情固定显示缺省说明，没有向精确版本的发布源查询；管理器release_page仅把版本拼成v-tag网页，未确认对应Release存在。与原IMP-20/分析03中“加载/可读/未提供/失败四态、独立重试、精确版本来源与网页”不符。对be完整UI和其冻结配置运行同一份新增黑盒flow测试，三个场景均实际断言失败：没有说明请求、仍显示猜测的管理器页面、缺少独立重试。原红notes-flow-red-admitted.log保留。首次抽取缺Tauri事件配置导致0测试导入失败，另保留notes-flow-red.log，不计产品红样本。

只读源审计直接读取官方npm2.84.0元数据与对应GitHubRelease：包name/version匹配，repository为git+https://github.com/lidge-jun/opencodex.git；非草稿v2.84.0正文7528字节。管理器非草稿v0.1.9正文1526字节。仅身份/URL/大小/摘要归档于三份*-identity.json；原公开响应外部留存。该审计是urllib来源核验，不是生产Rust HTTP客户端的实机更新测试，也未安装或执行该包脚本。

## 实现合同

- 概览/设置仍共用UpdateCenter，选中具体对象时挂载唯一ReleaseNotes组件。首屏与全部更新视图不调用说明接口；读取说明不调用版本检查、备份或安装。加载文字、纯文本内滚动、实际缺失、独立失败/重试与已核验网页按同一组件呈现。原updater正文优先；说明失败保留已有正文和版本/安装结果。移除原简单拼接网页地址，不信任旧缓存中的猜测URL。
- 后端update_release_details先验证当前已知管理器通道/候选与世代，或面板持久缓存候选，再独立查询；未接收任意来源URL。管理器只查维护仓库的精确v-versionRelease，面板先核实同版本npmname/version/repository再查维护仓库。响应tag、非草稿、精确html_url、stable非prerelease均核对，404为缺失，来源不符/网络/超量为独立失败。不从latest或通用README制造版本正文，不跟随HTTP重定向。没有精确Release时不猜网页。
- 来源与限额放入runtime.defaults.json的updates.release_notes：manager_repository=gzers/opencodex-desktop、panel_repository=lidge-jun/opencodex，单响应262144字节、正文65536字节、总查询30s。全查询用一个总截止覆盖面板两次HTTP；共用既有代理偏好和15s连接上限、默认TLS校验/UA，显式无代理不落回系统，手动支持HTTP/SOCKS5h。reqwest只启用既有socks特性，Cargo.lock字节未变。没有新增周期或自动重试。
- 活动根下cache/updates/release-notes/{manager-stable,manager-beta,panel-latest}.json最多三个缓存槽，schema1，包含对象/通道/候选/仓库/正文状态/核验URL/时间。按候选验证；可读正文24h、真实缺失1h后过期，未来/过期/坏缓存重查，失败不覆盖既有缓存。清缓存不动manager-state/update-schedule.json。所有目录拒绝链接路径，读最多512KiB，写原子0600，磁盘访问在线程池；Windows仍由安装位置data派生活动根或既有自定义根，macOS沿默认根。
- UI同对象/通道/候选/活动根single-flight；切候选/通道/根、冻结根或关闭观察者后拒绝迟到投影。后端每对象串行并持有数据根写入准入，关闭UI不取消已接纳的有界缓存事务；绑定保存不能穿过它。管理器查询后再次核对世代，旧通道缓存只存其自有槽且必须精确版本才可用。
- 说明重试是独立inline反馈，不向通知中心投递或解除检查/安装异常；event-registry补充真实UI反馈调用点。重试按钮移除时立即保留焦点在说明区，用户在等待中移到其他控件则不夺焦。新增组件遵守纯文本与小范围布局，未称实际双平台视觉验收完成。

缓存时间只管用户主动查看的说明；原45s启动宽限、stable24h/beta6h/panel24h、30min状态错误保护、对象级退避、安装/签名/备份门禁均未修改。无更新候选时不请求说明。

## 回归与失败保留

最终前端97文件/501passed，包含本轮说明与共享中心29项；Rust34组/884passed/0failed/6ignored，其中说明12项覆盖精确源拒绝、404与失败区分、单响应/总截止、重定向拒绝、UTF8/文本、缓存零请求/版本通道根隔离/强制重读/时间腐坏与链接拒绝。类型检查、生产构建、fmt、strictClippy(-Dwarnings)、diff检查通过。构建保留4条INEFFECTIVE_DYNAMIC_IMPORT警告；jsdom提示不支持实际canvas，不把模拟DOM/WebGL当原生图形结果。

六个ignored（live网络、钥匙串、npm、WebDAV凭据、helper、Tauri原生主线程）保持，不计通过；0测试组保留。notes-rust-full-final.log.gz是原完整日志无损gzip，原字节摘要与解压roundtrip列于manifest。未归档应用二进制、用户数据根或WebViewprofile。

首轮Rust101为SharedUpdateStatus引用模块错误，已修正；strictClippy首轮参数过多/测试未处理read数量、次轮冗余字段，改为显式Request并修夹具，原日志保留，无allow或降低告警。首轮全UI499通过/2失败为异步请求启动前就卸载的测试与残留mockOnce，明确等待请求开始并重置夹具后全量501通过；断言未放宽。原业务红三项另列，不混同编译/夹具失败。

committed-run-results.json记录17个精确源码文件摘要/父SHA/最终回归；独立与Gitblob校验。最后同630CI38490716335in_progress，frontend/release-tools/windows-smoke-evidence成功，后端jobs在途；不能提前声称完整CI/Windows编译或制品通过。

## 后续真机与门禁

精确CI成功后再核验同SHA隔离候选。Windows先协调结束旧be实例并确认实例数0，才切换新候选；本轮只给来源信息，不自动替换或新开应用。两权限、真实DPI下复验两个入口/对象四态、准确正文与链接、长内容内部滚动、独立重试、关闭重开、焦点和跨候选/根晚到；说明缓存命中无HTTP请求、失败不改版本/安装结果需真实计数和状态证据。旧be原生证据不证明630代码通过。

macOS同机官方0.1.9配对CPU+1.817629百分点/平均RSS+117.008965MiB失败保持，根因尚未定位；本轮没有新原生样本，不以功能单测解除。冷/热每版各至少20次、启动p95增量≤100ms且≤5%、10min CPU≤0.5百分点/RSS≤10MiB、主线程≤50ms、缓存100切页/10重启零自动网络/查询进程，以及真实周期/安装/数据保留/更新/备份/事件/原生/签名门禁均不变。八项TASKrequired_checks未闭合，review:scheduler-performance-0110failed，验收空、IMP/TASKin_progress。核心事实未晋升，无真实迁移、日常替换、自动合并、签名、公开发布或完成证书；FZ50许可仍缺。
