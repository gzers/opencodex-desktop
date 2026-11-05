# 连接后Windows原生补验（未完成）

代码1d40d73e8307a85d8d54b0c619011ef5a974713f已推送。产品重试修复4e6614ee3，后两笔仅原生竞态驱动；apps树a8bb18f9ff2781dbbfe67f91473a298f3cd74ae6。草稿PR #70用于CI/MSVC原生后验，不合并、不发布。

本目录全部实测使用原run41913003716/artifact13366782216，EXE SHA dd6f37cf2f6094cf9254ead08217cf8456e190cb213d3198c1b528172f1a0184、内嵌127543d2d0d9c2725f1a357710cd43bc69e3c44d。它是修复前基线，不能证明新4e661重试实现。日期2026-11-05，RDP账号15119/session2已实际连接并解锁，原生图由sky捕获；没有把CDP截图代替Windows实屏。

## 本轮有效结果

| 项目 | 结果及证据 |
| --- | --- |
| 60444→60446，首载10/30ms离开重开、两次正常重开 | 两端panel.json通过，原生图可见官方2.84面板；观察前后config.json字节保持 |
| Hide未返回即关闭通知 | panel-hide-race.json和panel-race-final-ready/panel-hide-race.json各5轮，通过真实postMessage提交及原生callback记录，public close在Hide callback之前，后续Show均visible-ready；请求/返回值不改，不人为延迟回包 |
| 服务未运行与加载超时 | 服务停止显示“未运行”；健康服务新origin首导航延迟20/60秒时按原8000ms在约8015/8003ms显示加载失败；20秒迟到完成会自动恢复。自动恢复不能当手动重试 |
| 超时手动重试 | 有效失败：60秒初始导航等待响应头期间点击原生“重试加载”，日志有reload requested，但重试再次超时，未取得新/web请求。截图native-panel-manual-retry-restored.png实际是失败（文件名错误）；不把它当恢复 |
| 云端独有对象首次全选 | cloud-first-corrected.json逐ID比较预先独立清单，本地1+云端1渠道、本地17+云端1模板；原生UI显示2/2、18/18 |
| 后续新增不扩选 | later目录实际读到新渠道/模板，原选择仍精确2/3、18/19，新增checkbox未选；cloud-later-no-expansion.json通过 |
| 已有明确范围保持 | 通过真实checkbox取消云端初始渠道/模板，冷reload后保持本地1渠道17模板；cloud-final-readback.json用正式transfer_choices逐项证明。旧可选读回命令transfer_choice_read错误已单独披露 |
| 零真实同步 | 本轮HTTPS固定合成服务33次请求均GET/200、零PUT/DELETE/POST。只读目录/版本与本机范围选择，未点同步/下载采用，不访问真实WebDAV或推理。业务config和渠道布局SHA与预置前一致；模板库仅更正错误夹具字段，非业务采用 |
| 后台停绘恢复 | 原生最小化/还原，5秒前台49draws、后台0、恢复36；strategy随原生可见性改变 |

## 性能仍失败

原预算B=1000/60、p95≤2B、over6B=0、三场景各30秒、高档和玻璃不变。静止/滚动/弹窗p95约181.2/142.2/174ms，over6B约175/116/197；全部actual foreground=true，滚动实际范围0.5～899.5px。原生图与完整失败JSON保留。

Chromium trace约38.7MB，按compressed-originals.json保留原字节SHA及gzip。GPU线程FinishPaintRenderPass最大wall171.07ms、最大threadCPU3.57ms，464段wall合计8701.33ms而threadCPU393.267ms；主线程BeginMainFrame最大19.057ms。这里主要是GPU/合成等待区间，不直接等于着色器执行成本，也不能单独指认DWM、驱动或RDP。trace场景脚本执行了滚动，但结果保存因Node不支持import.meta.dir失败，未将缺失的scene.json伪造成通过。

临时诊断依次去滤镜、去阴影、设置合成层、paint containment、offscreen content visibility、变简单圆角/opaque root，均未消除p95超标。隐藏内容仅诊断时p95约28.2ms，不是可接受修复。独立同EXE进程禁用DirectComposition仍约166.7/166.6ms，禁GPU切至Microsoft Basic Render Driver仍约181.8/166.7ms。禁帧率限制+禁GPU-vsync虽滚动p95约17.8ms，但静止最大1877.7ms且仍有15/2个over6B，拒绝采用。仅禁GPU-vsync仍约167.9/152.6ms。没有把这些开关落到产品、没有降档/弱化玻璃或放宽预算。

RemoteFX约10～12输出fps、编码6～9ms的样段未报告网络/客户端/服务器资源丢帧；这是定位线索，不是产品通过。初次计数器路径转义失败保留，performance-counter-admission-corrected.json才是实际捕获。GPU/DWM与进程归属只读记录在本目录，未停止日常或其它会话进程。

## 新修复与检查

根因假设由受控失败支持：WebView2初始请求未收到响应头时，对同一视图Reload仍排在旧导航之后。4e6614ee3在显式reload且lifetime.ready=false时先关闭待加载子视图，再建受限origin新视图；已就绪正常复用保持，并避免新建后再Reload。旧epoch/generation/退出和浮层守卫保留。尚待同源MSVC新EXE证明修复后恢复，不能据源码宣布已解决。

本地Vue面板8、GNU Rust面板12、Clippy lib通过；产品GNU构建因DLL导出258649超过65535失败，未生成可代验产品。CI41944607861的前端、发布工具通过，两后端仍进行；windows-smoke-evidence在Controlled parent did not become ready失败，原日志及artifact13378630948留存。只重跑该job的请求因父run仍运行被拒，待终态再补。无重启或取消正在执行的构建。最新1d40仅test/smoke工具变化，与e9e7源apps树一致，后续可逐字核对候选apps树，不冒称最新整轮绿。

## 失败归类与清理

native-screenshot-catalog.json逐一说明18张实屏的实际内容：若文件名早于状态、误称timeout/恢复、或未激活捕获到了遮挡前台，均明确不作为该项通过证据。先前invoke不可写、fetch未覆盖Windows postMessage通道造成两次空trace，以及错误树分组展开/过早DOM读取/错误预置submissionReceipts=[]等驱动失败全部保留。有效云端首次前没有scope或initialized，修正字段为映射后仍使用原独立expected文件；没有从scope反推期望。

本轮所有测试管理器/服务已按绝对路径、PID、CIM创建时间、session、owner和原生句柄时间精度核对后停止。connected-cleanup-final.json确认自有root无残留，日常MSIX38560和管理器24140仍存活。本轮外层清理第一次错误使用陈旧LASTEXITCODE误报失败，目标24292实际已安全退出，结果文件留存后跳过，不按旧PID再停。历史误停日常Bun30196事件及旧失败仍保留，不因本轮清理通过认证全历史零副作用。

正式MSIX默认adapter重启仍未做：当前用户包内含日常Codex，未证明一次性账号/会话，不能用本地CODEX_HOME、OwnedClient或原生面板替代。未改日常配置，未执行补偿重启。

后续：等待草稿PR70/MSVC候选，核对run/head/apps树、ZIP/EXE SHA与启动内嵌SHA，在新隔离根复验同一60秒导航阻塞→8秒超时→恢复端点→原生重试≤原超时→配置读回及五轮浮层竞态。launch-fixed.py与panel-server-fixed.ts已在scratch准备但未启动。性能定位/修复继续未完，正式重启须一次性会话条件。TASK保持in_progress、RCP总体failed，无CERT。
