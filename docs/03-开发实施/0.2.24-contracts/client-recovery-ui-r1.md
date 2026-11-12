# 0.2.24 客户端恢复收尾确认界面

revision: 1；2026-11-12。归IMP-34 P24-04，承接[显式metadata收尾](client-explicit-finalize-r1.md)。代码main d14125821，先行共享键盘修复ed63c511f。诊断accepted=false；本批提供已实现metadata收尾分支的用户入口，不把部分文件继续/回退、其它平台或整版验收标完成。

## 产品入口与原型

软件“诊断中心 → 环境诊断”在官方Doctor之后、界面自检之前增加“客户端配置恢复 → 检查客户端恢复”。使用现有UiButton/UiDialog和本项目间距、边框、明暗主题。既有唯一总原型index.html由post023-runtime-ui.js加载新的client-recovery-024.js/css，模块只在内存演示，不读配置/文件、调用IPC/真实服务、存储设置或执行命令；不另建第二份可编辑原型。

原型与软件共同覆盖：点击入口才查询，读取中；已恢复配置但缺收尾时“确认完成收尾”；未满足收尾/未知/运行中/无待处理/不支持的准确文案；诊断详情；执行中关闭可重开；成功后服务保持停止；状态刷新失败只重试刷新。没有展示可执行的“继续恢复/回退”按钮，因为对应产品许可尚未实现。原型后续界面不可用来掩盖这些差额。

可收尾文案说明“客户端配置已恢复，尚有一项收尾未完成；确认后重新核对原会话并完成收尾。客户端配置保持不变。”诊断详情折叠显示原阶段、原件/计划/完成记录与会话编号，未把原件SHA、内核锁/进程实现或密钥散进主流程。phase值映射中文；未核实不会写成无待恢复。两个版本此对话框均最大560px，窄窗口自适应；首截图发现原型复用旧宽弹窗后已用专属样式修正，没有影响其它原型弹窗。

## 交互状态所有权与确认

useClientRecoveryStore持有观察与执行promise，页面卸载/关闭只脱离观察、不取消已发确认；执行中重复点击/重新打开/路由重挂不发第二次finalize。查询有generation，窗口关闭后的迟到查询不能复活旧预览。成功且当前状态已核对的窗口再次打开会查询当前会话，不能无限保留旧成功页；刷新尚未确认的成功回执则保留供状态-only重试。

只有pending/restore_prepared、原件/计划verified、完成记录not_recorded或unregistered、有界正确attempt/revision/64hex摘要且后端canFinalize=true才显示确认。用户点击先记录当前live快照的实际D/H，再发送原attempt/revision/digest；后端沿前批独立再准入，前端不发文件路径/内容/凭据或任意恢复决策。

回执必须status=restored、attempt与原请求相同、revision恰好原+1；不匹配/失败清旧预览并提示重新核对，不自动重试写操作。成功回执保持显示，再取一次live实际状态：只有当前UI绑定仍是同D/H、新快照同D/H、client_recovery=not_required、runtime=stopped、无PID/port、canStarttrue/stopfalse/restartfalse且无其它动作，才acceptFinalizedClientRecovery解除本窗口恢复latch。普通迟到not_required状态仍不能解除此前失败latch，原过程动作保护保持。

刷新失败或不同位置/未知/仍运行/fixture来源都保留启动限制；界面准确显示“收尾已完成，当前运行状态尚未核实”，提供“刷新运行状态”，只读刷新不能再次调用finalize。关闭/重开失败刷新保留原成功事实，不把读状态失败冒后端收尾失败。前后端都不会自动启动服务。

## 实测发现的WebKit键盘问题

首次Chromium13场景通过；加键盘核验后的WebKit出现SUMMARY“诊断详情”→Tab→BODY，跳过后续按钮，0场景通过的原失败与两次定位原输出、焦点before/after和截图全部保留。该问题属于共享UiDialog原来仅在首末元素处理Tab，macOS WebKit的默认键盘策略可能在中间summary后跳过buttons而逃到外面。

mained63c511f将最上层对话框内所有未被子控件消费的Tab/ShiftTab按既有可见可聚焦顺序显式移动，空集合聚焦dialog；只处理top layer，已preventDefault子事件仍优先。没有修改系统设置或测试引擎断言。新增单测包含summary→取消→确认→关闭及反向；原Escape、busy、嵌套、异步菜单和子菜单先消费Escape规则继续通过。此为共享键盘行为影响，已核验其它消费者，不需重新跑未变材质特效或Windows原生双用户。

## 验证范围与来源

96项前端相关回归通过（恢复flow/API、原process-action、共享dialog键盘/玻璃结构、新增渠道及安装恢复消费者）；最后增加unknown completion guard/刷新失败准确文案后，只再验受影响3文件40项、类型和生产build，不重复无影响全组或后台原生。源码边界逐SHA记入source-final.json。

浏览器在实际LogsRoute与真正UiDialog上使用显式DEV审计夹具，不访问Tauri或真实服务；唯一总原型另走纯内存模块。Chromium与WebKit各15场景均通过：软件10组（含1180×760浅色、900×600深色；pending/unknown/active/none/unsupported/error、慢执行关闭重开、刷新失败后只读重试）及原型5组（eligible/pending/unknown/slow/refresh-failed）。可收尾场景逐9次Tab均保持对话框内，Escape关闭零mutation、确认后一次写调用与范围匹配新状态解除latch；每场景无pageerror，几何无横向溢出/视口越界，截图留存。

最后文案/guard变更后，WebKit只重验软件/原型两个refresh-failed受影响场景，通过；后端无变更，没有新原生通过宣称。全源source-final.json明确完整双引擎轮与最后局部修正的边界。人工实际查看过浅色/900×600深色确认页、刷新失败页及修正后的原型确认页截图，未将纯截图存在当无缺项证明。

中间一次Chromium脚本将原型目录拼为01项目支撑资料，10个软件场景过后在原型输入存在检查失败；该结果保留failed，修正为真实“01-项目支撑资料”路径后才15过，不把错误当原型通过。WebKit原焦点失败不覆盖。原型详情字体/背板沿既有总原型视觉系统，软件沿本项目组件，未声称整页/其它现有模块像素相等。

生产前端build/typecheck通过，dist逐JS SHA记录；DEV mock标记、合成会话UUID、window.__clientRecoveryFixture均不进入生产bundle，真正client_recovery_finalize命令与界面在bundle存在。dist是标准前端工程输出，不是新.app/DMG或冻结候选。没有本批真实Tauri按钮→原生finalize整包实测，后端对应前批原包/另进程6631ms证据分列，不用浏览器mock冒新native验收。

## 证据、清理及后继

.adg/evidence/0.2.24/client-recovery-ui-r1/保留假设、源/原型依赖摘要、测试/build结果、全部浏览器原失败/焦点轨迹/截图、最终双引擎与局部增量summary、生产mock排除检查与manifest。原型入口index.html有其它用户未提交工作，本批只提交新恢复模块/css与post023-runtime-ui两行loader，不改写/提交其index或其它原型；观察时总indexSHA留证，不伪称为本批独占。

所有9个可视轮次（含失败）均正常关闭browser/退出自有Vite，精确本批audit脚本进程缺席；Vitest已退出，无本批native服务/外部协调DB/测试Profile/日常或系统配置写入。纯原型只内存示例，正常dist保留；证据复制逐SHA核对后清理重复临时根。没有真实模型/WebDAV业务请求、新.app/DMG或发布。

下一步继续部分恢复继续/回退的原生授权与界面，一般图/全客户端/首次缺根/外部Agent/大树/跨卷/活跃刷新/完整生命周期集合与旧revision4首错，以及WindowsW0/W1/W2、NAME、复杂System代理、官方发布验证/README/固定候选。已实现收尾UI不代替这些范围，目标active。
