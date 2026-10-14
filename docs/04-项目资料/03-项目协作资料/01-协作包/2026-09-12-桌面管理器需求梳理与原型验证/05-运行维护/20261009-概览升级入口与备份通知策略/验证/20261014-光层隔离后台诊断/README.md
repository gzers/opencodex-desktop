# 光层隔离后台诊断：无生产修复或性能结论

2026-10-14，Asia/Shanghai。用户已明确“Windows仍在验收，Mac继续后台工作”；Mac不得开始原生应用/CPU/RSS配对测试。此轮仅源码、旧归档截图/采样条件和无头Chromium中的诊断CSS实验，不启动桌面应用、激活窗口、结束Windows实例或操作真实数据。实际Windows窗口状态由该端取证，不由本实验推断。

## 原性能事实与源码差异

既有官方0.1.9/de6862d4与deb740候选为同macOS27.0.1arm64、AC、18逻辑CPU、高/mesh/浅/100%、概览准备态，顺序各61样本/600秒；CPU增量+1.817629百分点、平均RSS+117.008965MiB仍超原预算。原源/环境与独立重算见20261012-deb740-十分钟性能对照。首末AX/JPEG只能证明捕获时外观，缺连续前台、网络/主线程和多组随机配对，不能判具体原因。

本轮6路径Gitblob/摘要比较：Logo几何与SVGscene在官方0.1.9/旧失败候选/当前d3b83b38完全相同；meshGlow、RuntimeMotionMark、base/materials在deb740与当前d3完全相同。0.1.9→deb的相关新增差异为背景光独立Teleport、正文/状态区层叠/覆盖尺寸及Logo锚点观察；这只是下一原生受控消融的来源线索，不证明布局导致内存或CPU增加。现有光层必须覆盖顶部/侧边留白，不能撤回需求求绿。

## 实验方法与得到的证据

复用已有playwright-core1.64.0和Chromium-headless-shell1248（实际browser.version见results），临时Vite仅绑定127.0.0.1随机端口，一次一个新无头页面/临时profile，页面使用现有DEV审计fixture；没有安装新依赖。页面请求仅允许本机Vite，其他HTTP请求阻断（实际没有外部请求）。覆盖setup/ready×mesh/css四场景、1180×760逻辑视窗/DPR2、高档/浅色；不代表设备物理DPI或WKWebView。

首轮在合成rAF推进后采原样，再临时加`.motion-overview { isolation: isolate; }`，生产文件字节未改。四场景的main/光层/Logo等八个节点位置尺寸完全一致，SVG帧与mesh画布尺寸相同；光层包含main正文边界，未退回padding内部。首轮仍有60–98个不同像素，主要位于控件/通知区；不是全图逐像素相同。

第二轮在独立页面关闭CSS动画/transition、隐藏notif-layer，然后固定同SVG帧比较并读LayerTree。四场景仍几何/SVG/画布相同，setup两场景30个像素不同、ready57/60个像素不同，总3587200像素；最大单通道差13/14，差异范围与摘要见controlled-pixel-verification.json。这些有限静态快照只说明本浏览器下该样式没有改变记录的几何，不能据此认定全部材质/主题/窗口/交互已等价。

第二轮setup-mesh/css和ready-mesh的前后层数14/19/24相同；ready-css开始层快照缺失、结束29层，不当层数变化证据。所有记录的LayerTreepaint事件为0：缺适用绘制仪器与采样窗口，不能称无绘制/重绘消除。120个合成rAF在同一JS任务同步执行，批次毫秒数不是实际帧时间/主线程任务p95，不参与预算判定。没有基线浏览器CPU/RSS配对，不拿无头合成层代替原生WebKit/真实10分钟测量。

因此没有建立性能因果、没有测得资源改善、不修改生产isolation或画质/帧率/尺寸来宣称通过。需要将来协调Windows实例0和新的Mac前台窗口后，按原预算做真正单变量/重复配对再决定修复。

## 失败与清理范围

首临时脚本因newline字面量解析失败，浏览器/服务器尚未创建，原脚本及headless-run.log保留；修正仅临时诊断脚本，第二log成功。两成功实验使用finally关闭自己的无头浏览器与Vite，不结束或操作产品实例。首轮截图和第二轮受控截图分别保存，未覆盖旧原始样本；结果注明DEVfixture/合成rAF。没有复制生产源码、桌面二进制、数据根/用户载荷或WebViewprofile。

原performance required check仍failed，八项checks未整体闭合，TASK/IMPin_progress、空验收。冷/热各20、启动p95增量≤100ms且≤5%、10min CPU≤0.5百分点/RSS≤10MiB、主线程≤50ms、缓存100切页10重启/真实周期及全部双平台门禁保持。FZ50许可未收到不执行；不迁移真实数据、替换日常版、合并PR、签名/发布、核心事实/通道晋升或创建完成证书。
