# 0.2.20 原型一致性审计证据（2026-11-02）

本轮只有问题登记和只读审计，不开发、不发布。问题与解释唯一入口：[0.2.20清单](../../../../docs/04-项目资料/01-项目支撑资料/06-问题记录/0.2.20-bugs.md)。软件是已发布v0.2.19/cb4c9c9b7；原型是Git导出cfbd90c4b，另读已确认MCP增量1da4cabca；并行工作区总原型改动没有混入。当前记录不是0.2.20产品验收证书。

## 方法与范围

- input-manifest.json固定10份输入的来源、角色、摘要、大小和类型：8个生产代码/测试文件、冻结sync-config.js和用户截图。用户原截图保存在问题附件0.2.20，未改图中内容。
- compare-sync.mjs：用项目已有DEV内存模型夹具，补充API返回值只在浏览器内存，分别采集Chromium/WebKit、1180×760浅色与900×600深色。两尺寸均取file-sidebar、export-with-credentials、extra-plaintext-confirm、file-after-reload、webdav-sidebar、webdav-connection，及冻结原型导出，共每引擎14条结构观测和16张图。
- observe-flows.mjs：内存注入本机新版、云端新版、加密、未知变化，检查同步预览/独立口令窗、概览同步路由和导入初窗。每引擎7条观测。没有执行最终确认，不测试真实文件、密码、服务器或备份。
- findings-observed.json：38次差异观测均复现旧行为，是发现问题的证据，不是38项通过验收。包括未知变化时确认可用。
- comparison-contact.jpg为图像索引；原始PNG保留。样例对象数量、凭据开关与mock数据不完全同源，不能按整屏像素差判缺陷；产品额外控件与流程按源码和显式行为判断。
- routes/route-audit.json及16张图：执行仓库原有qa/audit-routes.mjs，固定原型路径，采集overview/panel/models/extensions/sync/logs/settings/tray八路由。两侧数据状态未完全对齐，只用于结构盘点；面板无原生WebView，托盘是网页样例，不证明原生通过。标题/按钮相近不能证明整页一致。
- application-dialog-inventory.json：源码静态扫描23个文件37处引用，其中2个dev文件2处排除后为21个生产文件35处组件引用。动态AppModal、自定义modal和平台选择器另在清单登记，扫描不声称穷尽所有运行状态。

## 失败与限制

首轮compare-sync浏览器驱动重载时DEV参数route=models使页面回到模型页，等待同步侧栏超时，记录在chromium-first-driver-timeout.log。更正驱动显式重导航后复跑，未改产品或放宽原断言。Chromium基础对照无页面异常；WebKit基础对照在原型侧保留一条“ResizeObserver loop completed with undelivered notifications.”，未过滤；两引擎流程观测与8路由结构采集无页面异常。此警告不直接判为已发布产品错误。

浏览器加载的是发布源码的DEV渲染和显式夹具，未操作日常原生应用；真实文件/导出/导入/同步/备份/Windows和macOS原生键盘、系统菜单仍待后续验收。没有将旧功能测试、发布安装验收或本轮mock图片作为全量生产对齐完成的证据。

审计脚本位于docs分支证据目录，是本次检查方法，不是新增产品或测试实现。固定端口52422～52425，只绑定127.0.0.1；进程在finally中关闭。复用需检查独立临时目录与输入版本。SHA256SUMS校验归档字节，目录关闭Git行尾转换；正文解释和问题生命周期归0.2.20清单。
