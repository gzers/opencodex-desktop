# S21-POST-02 Windows 全局材质贡献回交

2026-11-04，Asia/Shanghai。W线按IMP-30 / DMD-POST021 / AC-P21-02执行。代码和原生证据已形成；**性能门未通过，完整AC和问题不得关闭**。不合并main、不发布；最终集成、Mac验证和权威回写归L。

## 取用身份

| 项目 | 固定身份 |
| --- | --- |
| 代码分支 / HEAD | codex/post021-windows-materials / 684e85c29f76584de4ab53026dcc315e34b57c62 |
| 共同代码基线 | 7a8441f7e647e4b3540d5907e44ac4f186f90998 |
| 文档贡献分支 / 起点 | docs/post021-windows-materials / c6f0a3f29ae59f6384f61f25dea773061f41bf11；本文件所在提交即文档交付身份 |
| 修复前实际程序 | 正式0.2.21，startup commit 6e2197fd0b8a36dcb8d893b62693c5a2a77151cc；UI/Rust产品输入与共同主线基线无差异 |
| 修复后实际程序 | startup commit 684e85c29f76584de4ab53026dcc315e34b57c62，EXE SHA-256 f408616c94b309e283e1bf2bbedf24bd355efd492b0977857f39565da171154d |

代码提交依次为8cb16415d（标准滤镜与控制按钮回退）、56671d50f（产物/原生检查）、a5b431d22（实底回退优先级）、684e85c29（真实父面与稳定帧探针）。最终改动九个文件。文件/产物及交互合同摘要见[source-manifest.json](source-manifest.json)，进程、EXE和源码绑定见[runtime-identity.json](build/runtime-identity.json)与[启动日志](build/sandbox-app.log)。

## 已确认根因与修复

1. **生产压缩丢掉标准滤镜。** 源码把backdrop-filter写在-webkit-backdrop-filter前。锁定依赖Lightning CSS 1.33.0对等值别名压缩时只保留最后的前缀声明。实际WebView2标准语法支持为true，前缀支持为false，因此正式产物的公共dialog计算值是none，填充却仍为56%透明。同样命中详情吸顶头、折叠按钮和任务卡按钮。不是Windows不支持玻璃、Mica失效或showModal必然不能模糊。
2. **无滤镜回退被优先级压住。** effects.css在不支持滤镜的分支内使用html，基础材料使用优先级更高的:root。进入回退也可能保持透明填充。将后置规则改为:root，实底才真正赢得级联。原生显式注入真实分支，先捕获失败再修复；[原失败](failures/fallback-specificity-failed.json)保留。
3. **面板共用快捷浮层缺不支持分支。** panel-hub.js的样式原样注入，不经上述压缩，正常高/中档声明完整，但无法识别滤镜时只有半透明填充。本次补亮暗实底及hover回退；官方页面、命令不改。

修复把前缀声明放在标准声明之前；任务按钮用保留原有6px的独立语义变量，在低档/不支持时取none。没有提高正常玻璃不透明度，没有改字体、布局、窗口配置、主入口、依赖、版本或同步业务。UiDialog和DetailWorkspace的script/template前后摘要完全一致。新增测试直接运行锁定生产压缩器扫描全部CSS/Vue样式，另有构建后跨懒加载CSS检查，避免只看源码假通过。

原始链见[最小压缩反例](root-cause/minifier-repro.jsonl)、[正式包规则](root-cause/baseline-dialog-rules.json)、[实际运行样式](root-cause/baseline-runtime.json)。修复前按共同基线本地构建，得到与正式包相同的index-CCOMTbHb.css。

## 原生前后证据

同一720×502逻辑像素连接弹窗，正文13px、标题18px、填充rgba(34,36,42,.56)均保持；背景条纹边沿对比度**12.5 → 0**。高档实际消除条纹，透明度未增加。条纹只注入隔离测试背景，不进入产品。

| 场景 | 修复前 | 修复后 |
| --- | --- | --- |
| 实际正常连接窗口 | [Windows原生截图](before/webdav-dark-high-native-0.jpg) | [最终源码原生截图](after/webdav-dark-high-final-native-0.jpg) |
| 同场景高对比条纹 | [原生截图](before/webdav-dark-high-stripes-native-0.jpg) / [原生WebView2 PNG](before/native-dialog/dark-high-connection.png) | [最终原生WebView2 PNG](native-final/dialog-matrix/dark-high-connection.png) |
| 详情吸顶/折叠及其它配方 | [16类基线](before/native-gallery/dark-high-materials.png) | [16类最终结果](native-final/gallery-matrix/dark-high-materials.png) |
| 真实通知与设置行 | — | [实际生产宿主](native-final/production-matrix/dark-high-settings-feedback.png) |
| 弹窗内菜单 | — | [原生截图](after/dialog-select-native-0.jpg) |
| 嵌套守卫 / 200%滚动 | — | [守卫](after/nested-guard-native-0.jpg) / [滚动](after/scroll-200-native-0.jpg) |

PNG由**运行中的桌面EXE所属WebView2**经loopback CDP捕获，非无头浏览器；JPG来自Windows Computer Use。原生输入也由该工具执行。一次失焦捕获返回了遮挡目标的同名基线画面，已排除；[后台图](native-final/lifecycle/background-native-client.png)使用目标WebView2独有CDP target，不把错图作为证据。

## 表面盘点与覆盖矩阵

[源码消费者清单](surface-inventory.json)枚举30类选择器及37处实际UiDialog声明；类名命中只是盘点，不代表每个宿主均逐一原生操作。完整数值见[COVERAGE.json](COVERAGE.json)，原件在native-final。

| 分类 | 原生覆盖 | 亮/暗×高/中/低 | 额外验证 |
| --- | --- | --- | --- |
| dialog / AppModal | 实际连接窗口；查看、导入、导出预览、新增渠道、节点检测、安装、更新中心七代表宿主 | 连接六格通过；高中真实消除条纹，低档实底 | X/Esc/遮罩、嵌套、busy、beforeClose、输入、焦点、滚动 |
| 通知/Toast/任务/保存提示 | 实际NotificationCenter、ToastHost、TaskProgressHost、SaveResultNotice；信号为隔离内存输入，未真正保存/重启/检测 | 六格通过；16类采样补具体滤镜 | 倒计时/暂停/不抢焦点由未改脚本及1275项回归支撑；本轮未另跑完整8秒重启链 |
| 下拉/快捷浮层 | 实际UiSelectMenu壳层Teleport与dialog内定位；panel-hub原样源码 | 六格通过 | dialog菜单六格＋50%/200%高低档，首个Esc只关菜单；hub承载边界见下 |
| 卡片/面板/控件/吸顶 | 16类生产CSS采样、实际设置行/详情头/折叠/吸顶tabs/按钮/选择器 | 六格通过 | 修复前两详情面和通知按钮反例保留；实际详情滚动 |
| 无滤镜路径 | 直接将生产不支持分支体注入隔离原生窗口，用户仍为高档 | 暗色高档故障注入通过；亮暗低档通过 | 16类＋真实连接窗；hub另验证无滤镜/hover回退。不是强设低档冒充故障 |
| 生命周期/缩放 | 真实前台→失焦→恢复；隔离进程重启读回high/mid/low | 应用100%基线，补50%/200% | 后台2秒光场0次绘制；不改用户档位；原生滚轮在200%使scrollTop 0→96.75且X可见 |

明确不强改玻璃：外壳/侧栏/标题栏、页面.main沿既有分类；系统文件选择器与官方内嵌页面不接管理器配方。概览.motion-actions已有性能约束，使用分层填充并明确不模糊，保持原分类。主/危险/ghost按钮、实底输入、表格及内嵌通知行依设计，不因“全局”变成全部透明。

同步预览业务流程未运行：无连接、未请求日常WebDAV、未调用同步。公共UiDialog与同一传输组件的真实导入/导出预览已验；合入L同步修复后，由集成线在隔离服务补同步预览及四项业务复验。节点窗口只查看一条内存禁用渠道materials.invalid，未发检测请求。

## 检查结果和适用环境

- 前端全量**194文件 / 1275测试通过**；typecheck、production build、22个滤镜规则的产物扫描通过。既有动态导入/包大小警告保留。
- 面板后端定向**10/10通过**；最终EXE本地编译通过。未改Rust源码。既有工具链无rustfmt，cargo fmt明确未成功；未改工具链或放宽检查。完整MSVC CI/Clippy待集成线补。
- git diff检查、九路径写集、字体tokens、同步业务、原生配置及script/template不变核对通过。
- **性能门失败，不能写通过。** 同条件空闲各30秒：基线/修复概览p95为150.5/155.3ms，详情滚动153.6/155.2ms，弹窗覆盖概览164.7/157.9ms；均超过原2B=33.33ms且有超过6B帧。无长JS task，后台停止绘制通过。只代表本机远程会话观测，不能证明物理桌面性能无回退。见[PERFORMANCE.json](PERFORMANCE.json)及完整间隔数组。

实际环境：**Windows Server 2025 Datacenter 10.0.26100，WebView2 156.0.4314.58，Intel UHD770 / ANGLE D3D11**。原生HWND DPI=192（OS200%），DPR=2，应用100%基线。只读Win32明确**SM_REMOTESESSION=true**；另有Parsec和Microsoft Remote Display适配器。不表述为Windows10/11或本地物理显示器通过。原件见[OS](environment-os.json)、[原生DPI/远程会话](native-environment.json)、[实际WebView进程](webview-process.json)。

最终包由既有Rust1.99.1 GNU工具链生成debug/custom-protocol EXE，嵌真实production Vite dist。项目只为MSVC目标嵌Common Controls v6 manifest，本地GNU复制件启动前只添加同一manifest资源；产品源码/配置不改。保留[前后EXE摘要](build/local-gnu-manifest.json)。不能当作MSVC release原包、安装或升级验收。

## 失败、未完与集成要求

1. **补物理Windows / 正式MSVC release性能验证**：当前远程环境连原包也不满足预算；门槛不调整，W主TASK/RCP保留失败，无关闭证书。
2. **Mac合入同一最终源码后复验共享材质**：包括该WebKit标准滤镜支持、三档、实底回退及退出/焦点。W未声称Mac通过。
3. panel-hub原样源码已在管理器原生WebView2六格/故障注入验证；**真实官方子WebView承载待集成复验**，本轮不启动日常Codex或冒充子面板已运行。
4. 合入L后在同一最终提交补隔离同步预览、四项同步业务及跨端回归；本线不改POST-01/03/04/05，不关闭AC-P21-06，不改旧证书/清单/DMD/IMP。
5. 无滤镜路径使用明确分支注入。未声称覆盖未知驱动在“计算值仍为blur而GPU静默不绘制”时的自动检测。已确认的Windows现场路径是产物缺标准声明，已原生修复。

完整检查见[CHECKS.json](CHECKS.json)。独有TASK/RCP位于.adg/work/objects的POST021-WINDOWS-MATERIALS/STICKY/HUB-20261104。三个TASK保持in_progress等集成；主回执failed（性能），子回执只表达各子写集定向通过，不关闭整项，未生成CERT/ICERT。

失败原件保留：生产压缩四反例、回退优先级反例、测试路径前缀/父面/过渡/注入位置修正、初次隔离目录未建导致拒绝、GNU缺manifest拒绝（由manifest说明承接）、远程性能失败。failures目录不能作为最终通过。

## 复现入口

在代码贡献分支使用锁定依赖和Node24：

    npm --prefix apps/desktop/ui ci
    npm --prefix apps/desktop/ui run test -- --run
    npm --prefix apps/desktop/ui run build
    node apps/desktop/ui/qa/materials-build.mjs

运行实际Windows EXE时进程级启用OPENCODEX_SANDBOX=1；将SANDBOX_ROOT/BOUNDARY及HOME/USERPROFILE/APPDATA/LOCALAPPDATA/TEMP/TMP/CODEX_HOME/OPENCODEX_HOME全部指向新建隔离目录，设置OPENCODEX_WINDOWS_SMOKE_CDP_PORT。目录先存在，不继承日常浏览器数据。reproduce/launch文件保留本轮绝对输入，不直接当作跨机脚本。

    node apps/desktop/ui/qa/windows-materials.mjs http://127.0.0.1:9422 D:/isolated/evidence dialog-matrix D:/isolated/candidate/sandbox

同一入口支持gallery-matrix、fallback、production-matrix、scale、hosts、hub、performance、performance-modal和restore。先用真实IPC核对四个活动目录绑定均在给定隔离根，路径不符即拒绝。默认断言失败非零退出，写*-failed.json。额外baseline参数只记录旧包反例，不作新包通过。

原生OS输入由Computer Use执行，native-final/inputs/01至21为只读DOM回执和原生截图；不是CDP合成按键代替原生输入。日常安装进程始终保留，不写日常配置、不真实同步、不重启日常Codex。
