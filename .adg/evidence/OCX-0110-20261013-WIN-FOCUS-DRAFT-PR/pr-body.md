返回“全部更新”会移除当前聚焦的返回按钮，导致焦点落到 BODY，更新中心仍标记 aria-modal。本变更在视图渲染后，仅当弹窗仍打开且焦点已外逸时，恢复同名控件或弹窗本身；仍在弹窗内的焦点及关闭后的焦点保持。

新增管理器/面板返回全部视图及同名关闭控件保留回归。Windows版本探针另增加仅测试编译的四阶段耗时、夹具就绪文件状态及CI宿主参数诊断，原2秒探针、4秒收尾、5秒夹具预算和CI并发不变。

验证：前端最终同测试blob红3失败/8通过、绿11/11；完整前端95文件/483通过，类型检查和构建通过。head be7395acfce0ec1e36b83300b576db408f8c35bb 的CI38352525092及候选run38355803684成功；Windows主库505通过/4忽略，完整cargo目标601通过/7忽略。候选NSIS/MSI隔离runner安装、首屏及卸载通过。同run ZIP摘要/CRC和本机只读提取MSI EXE的3a26098e…身份已核实，候选尚未在本机运行。

保持草稿：原d975b23 CI38349316806失败原因尚未隔离，原件保留。本机仍为eae8532的唯一隔离实例与既有真实24小时周期；焦点修复须在原周期拥有者终态/cleanup及零实例核验后进行同SHA原生复验。普通/管理员、物理DPI/云母/标题栏、安装/data/卸载保留、完整更新/备份/事件、原预算0.1.9同机配对性能及最终源真实周期仍开放。TASK182原件、FZ50显式脚本许可和独立测试环境尚缺，既有性能failed不清除。

证据按归属保留在docs/governance-main：IMP-OPENCODEX-DESKTOP-20及OCX-0110-20261013-WIN-UPDATE-FOCUS-REGRESSION、WIN-UPDATE-FOCUS-CI-FAILURE、WIN-FOCUS-CI-FULL-LIB、WIN-FOCUS-DIAGNOSTIC-CI、WIN-FOCUS-CANDIDATE、WIN-CYCLE-TRANSITION-GUARD。未降低门禁、合并、关闭、迁移真实数据、替换日常版或发布。
