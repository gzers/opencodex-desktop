# Windows 光层定位后分配

代码 `feature/0.1.10-windows-query-etw@d94c215d869c0560717d79dc02881ce8858afe88` 已提交推送。本补丁消除复现出的旧容器中间缓冲分配，尚未证明8be候选的平均RSS +33.249280 MiB失败已解除。

## 诊断发现与修复

组件在onMounted中更新Teleport目标后立即创建WebGL画布；目标的DOM移动尚未完成，初次尺寸来自旧的 `.motion-mark`。后台Edge使用Git中正式019/8be的真实组件、着色器和CSS，配以简化壳层及相同已安装Vue/Pinia依赖，记录canvas宽高属性实际写入。最初简化壳层尺寸与原生不同，原结果保留作方法观察；补齐 `.desktop-stage/.app-shell/.side/.main` 原壳结构后才形成匹配原窗口的1496×1342结果。两者都不是完整应用、Tauri EXE或RSS门禁实测。

相同1180×760 CSS视口/DPR2的正确壳层下，019最终1426×832；8be依次写入1426×150、1426×960、1496×960、1496×1342，其中前两次位于旧容器，后两次位于最终路由。临时延后创建变体及最终真实工作区源码均只写1496×150、1496×1342，全部发生在最终路由。高/中档、浅/深的最终源码8组观察中均只有一个画布，移除组件后均为0。宽高分别赋值产生的两次设置仍存在，没有据此声称仅一次驱动分配或测得节省的内存字节。

生产修改仅在DOM更新完成后创建网格缓冲；Logo仍同步建立，统一外观策略、着色器、颜色、最终尺寸、DPR、覆盖范围和动画逻辑不改。提前卸载时跳过待执行的创建，mesh模式watcher在布局未完成前不抢先分配。Vue nextTick的用途按官方DOM更新flush合同核验，来源记录在证据中。

## 回归与成像

新增两项回归在8be实现上失败：首次getContext必须位于最终Teleport目标；DOM移动前卸载不创建WebGL上下文/程序，也不遗留动画或背景节点。首次把整个scene也延后的方案造成两项既有即时Logo测试失败，原日志保留；最终保留scene同步，仅延后mesh，没有删改既有断言。

最终本地UI99文件/535项、类型检查、生产构建通过。实际最终工作区源码的后台Edge对照中，浅色与深色静态图各2360×1520，RGBA逐字节相等、差异通道0、最大差0、像素摘要相同。高档仅核对分配路径和销毁，不用不同采样时刻的帧数宣称性能或动画通过；静态像素一致也不冒充全部原生视觉矩阵。

首轮探针将Windows路径直接插入模块，触发八进制转义语法错误，未形成有效渲染结果；已保留失败方法/日志，再修正URL路径并单列正确壳层和最终源码结果。后台诊断未启动OpenCodex Desktop原生实例、修改日常安装或占用前台。

## 源码与验收边界

原件在 `.adg/evidence/OCX-0110-20261016-WIN-MESH-ALLOCATION/`，包含各阶段方法、真实宽高写入、原始截图、逐像素比较、修复前失败及最终本地日志。源码文件摘要绑定到真实Git对象；旧中断轮、8be有效失败样本和Mac性能失败均保留。本发现只证明存在额外初始化步骤，没有原生GPU分配栈/内存转储，不能归因为全部32.33 MiB WebView差异或保证新EXE进入10 MiB预算。

同SHA push `38824163566` / PR `38824151573` 沿各自首次CI执行；07:35:40两轮仍在进行。完整终态和实际checkout/tree日志核验通过后，才允许单次请求新候选。PR5已按最终范围更新读回，仍draft/open、未合并/无自动合并。GitHub连接器更新被403拒绝后，先用现有本机Git身份读取原值，再只作一次正常PATCH并核对回读；没有改变账户权限。

08:04完整PR `38824151573` 已success并完成原始日志独立核验：五个必需作业全部成功，main专用build在特性PR中按工作流正常skip；前端99文件/535项、Mac35组923passed/6ignored、Windows35组648passed/7ignored。实际PR checkout `f2c5f1bd8ed827897255e2aaf174c23a3de7fbf9` 与head d94的整个tree均 `f8e4654cdd13ae40fe7ae29bb61f5b0d5638d668`，未假定EXE字节相同。16份冻结原件及独立verifier在 `.adg/evidence/OCX-0110-20261016-WIN-D94-CI/admitted-pr/`；首版把main-only skip误拒绝的准入脚本失败日志保留。这里只用一轮完整同源CI作为候选构建准入，没有合并不同轮局部job来替代整轮成功。另一轮push的Windows已完成，Mac工具链仍未结束，继续保持该轮pending，终态不反写此快照。

按上述准入只请求一次候选 `38827408691`，固定source d94、run_attempt1、purpose=candidate。08:13:05 package仍进行，query-etw和retention预期skip；下载、实际安装载荷和原生补测尚未完成，不能把候选请求当成制品通过。后续原件在 `.adg/evidence/OCX-0110-20261016-WIN-D94-CANDIDATE/`。

08:18:10伴随push `38824163566` 终态为failure。唯一失败job是Mac backend，GitHub check-run注释为“The hosted runner lost communication with the server”，另有macOS arm64容量提示；job最后状态停在Rust工具链安装，cargo fmt/Clippy/test均pending、未执行，不归为源码测试失败，也不把该轮改标成功。作业原日志下载HTTP404“specified blob does not exist”，缺失如实保留；注释只说明runner失联，实际根因未确定。未重跑或取消任何作业；此前一轮完整PR成功及由其准入的单次候选保持各自身份。8份原件、终态、jobs/check-run/annotations与独立verifier归`.adg/evidence/OCX-0110-20261016-WIN-D94-CI/push-runner-failure/`。截至08:20:17候选仍进行，未启动宿主产品。

019冷/热各20、p95≤100ms且≤5%、CPU≤+0.5pp、平均RSS≤+10MiB、主线程50ms及完整请求/周期预算不变。DPI仍待验，真实更新/来源、安装保留、树表/事件、历史bd2失败窗口、严格单EXE恢复交接和Mac CPU/RSS继续未闭合。TASK-182/IMP-20不关闭，不合并、签名、晋升或发布。
