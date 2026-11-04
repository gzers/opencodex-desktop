# POST01 / POST05 交接

2026-11-04。产品源码已稳定，本工线不再修改。正式任务 TASK-OPENCODEX-UI021-INTEGRATION-20261104，本写集对应 AC-021-01 / AC-021-04；未关闭正式任务或宣称整版发布通过。

## 文件与实现

代码根：/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes，分支 codex/0.2.21-fixes。完整15文件清单、SHA-256及检查见 [receipt.json](receipt.json)。

- SaveResultNotice.vue：取消自动聚焦；保留8秒及主动悬停/键盘/隐藏/确认暂停。确认 X / Escape / 遮罩 / 取消只关确认，公共窗自动归还焦点后续计；到期不重启，稍后收起；旧请求和旧计时不关闭新通知。
- ChannelsView.vue：选择列＋渠道/模型/启用/操作。名称160–220逻辑px、操作250px；检测二字按钮位于操作列，无节点下拉列或额度。
- ChannelNodeDialog.vue / ChannelNodeSelect.vue：复用现有 UiDialog，查看型无底栏。单节点/一键检测、停止、选草稿分开；关闭观察不停止。渠道实例保留在过滤列表与详情分支外，搜索、详情和模型页标签切换后重开仍有结果；结果更新不换DOM、焦点不丢。
- NodePicker.vue 保留详情节点编辑与检测，停止保留完成结果，当前项留勾选；useNodeMeasurements.ts 补渠道/provider绑定、同步失效和旧停止错误防串写。
- 未保存改名时按稳定 channel.id 使用已保存 provider 的凭据归属；实际保存来源变化后旧结果失效。没有在界面取出或展示Key。
- 对应8份测试及 qa/geometry-visual.mjs 已迁移。公共UiDialog/AppModal、ChannelEditor、其它modal worker文件、后端和同步文件均未由本写集修改。

## 最终检查

| 检查 | 结果 | 证据 |
| --- | --- | --- |
| 受影响单测/真实Vue消费者/组件归属 | 19文件177项通过 | [日志](affected-regression-final.log) |
| Chromium / WebKit交互 | 各136项通过、各12截图，0页面错误/外部请求 | [Chromium](chromium/report.json)、[WebKit](webkit/report.json) |
| 冻结原型→软件几何 | 各20项通过，四组列表/检测配对截图 | [Chromium](chromium/prototype-parity/report.json)、[WebKit](webkit/prototype-parity/report.json) |
| 既有表格/模型几何回归 | 各47项通过 | [Chromium](geometry-chromium.log)、[WebKit](geometry-webkit.log) |
| 最终typecheck / Vite build | 均通过 | [typecheck](typecheck-final.log)、[build](vite-build-final.log) |
| 本写集diff --check | 通过 | [回执](receipt.json) |

POST05/09采用cf5b50b4fb86e12b64cad209440813159ee06241，119文件逐一核验原manifest摘要；临时只读快照不是第二份活动原型。POST01采用G0 checkpoints/g0-core/manifest.json合同，没有采用整个脏原型的未来功能。

1180×760 / 900×600、浅/深及50% / 100% / 200%组合：原型名称列220px，软件在160–220px内自适应；两侧操作列250px（WebKit个别样本有0.03125px舍入）。对照证明采用范围的几何与行为，不主张不同mock数据逐像素相同。

先保留旧反例再修复：原通知/下拉16项、列表2项；另有provider/停止竞态2项、过滤/详情销毁2项、未保存改名凭据归属1项。日志路径列在receipt.json.counterexamples，中间失败不覆盖成最终绿日志。

## readonly与保护

公共窗分开readonly（禁止选节点）与measurementDisabled（禁止检测）；明确允许的只读连接可检测、不改草稿，已有测试。独立只读画廊默认无IPC，允许检测需明确measurementDisabled=false。

生产列表保持原protectedChannel边界：openai/chatgpt/openai-multi/combo及OAuth/forward不能因迁移新增通用检测、连接修改或切换权限；采用原型native也排除通用测速。没有把允许只读检测的公共窗一律禁测，也未放宽既有官方/专用认证保护。

## 主线交接

本写集无未完实现项。主线统一整版UI/后端回归、原生macOS/Windows、隔离保存/重启和发布验收，以及共享DMD/IMP/TASK回写；本回执不代替这些验收。

只在早期共享快照跑过一次全UI（1197通过、10失败、3条showModal桩异常，详见ui-regression.log），之后按分工未重复全量。中间typecheck的两处未用导入已由主线修复，最终为绿。

未执行git add/commit/push；未测试真实渠道、读取日常Key、修改日常配置或重启日常Codex。共享HEAD可能由其它工线推进，receipt中的HEAD仅为观测值。变更状态见owned-status.txt，跟踪文件差异见owned.diff；新增文件以工作区实件及哈希为准。
