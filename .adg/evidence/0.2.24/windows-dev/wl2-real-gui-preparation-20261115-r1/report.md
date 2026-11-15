# Windows真实GUI桌面入口准备（尚未执行桌面验收）

源码0c618cdfde4da37da0ca90e1b0913d7d81f63a29，tree e6c6832e3f02d58c924c3b0409d4575161bb86b6。代码已推codex/024-windows-native。新增Windows内部注册，显式任务环境才启用真实主窗口探针；正常前端和后台setup后核sandbox/无runtime/原普通SID/session、native窗口可见及实际主文档Finished，再用私有pipe认证。无公开命令/权限扩展，不改Vue/共享trait/官方2.99或日常配置。真实官方Owner路由仍未完，不能把空broker中GUI认证称生产收口。

新独立观察者保留broker创建handle，启动真实产品EXE；逐次验证native主窗口属于精确原Child、正常退出/只故障该原GUI/新GUI认证、递增代次/原连接replay拒绝及同原broker/noJob。最后只请求空host显式退出，broker rawExit未知继续null；此逻辑已编译，尚未运行桌面用例。GUI页面Finished不是Vue DOM验收，frontendDomAssertion=false。r3的旧空H/fixturemanager用例不重跑不冒actualGUI。

| 准备验证 | 结果 |
|---|---|
| 隔离Node24.11.1 | nodejs.org固定包与发布SHA256匹配，未做签名验证；仅target工具目录 |
| npm ci --ignore-scripts | 原退出0；无日常Node/PATH修改 |
| vue-tsc / vite build | 原退出0/0，当前前端源未改；32文件1162800字节归档 |
| library01 custom-protocol | 原退出0/251577ms，6passed/3ignored；lib34warnings |
| product01 custom-protocol bins | clean最终0c618cdfd，原退出0/224425ms，lib68warnings |
| 实际桌面GUI | 未执行，待marenben原桌面Shell启动准备好的入口 |

全部96份product执行输入对最终工作字节一致、Gitblob行尾等价。library01测试EXE是c30168689加原始工作差异，52份Rust输入逐字等于最终0c618cdfd，实际build provenance保留，不称clean最终构建、不重复未受影响测试。产品EXE及ocxd均为clean最终提交构建，摘要归product01-artifacts.json，桌面3输入原件归prepared-input-manifest.json。首错null，预期负例不是运行失败。

实际产品Desktop77121024/SHA256b440af3bd6b348e3de80d159cffebc05b05fa86ecb376c62c1b4a69c75d3866a；ocxd5398528/SHA25645d54e076b2571cbae10e16f0f0fda83e34233348085f65a0e0b938c2f2f5cc2；ocx306176/SHA256594e3d4044823acf49543dcd7b6c171e3b1d8c07deb129327692353a4b3b9ee8。observer95040512/SHA256a6f6ccef62405667666f2dccdc8060aa6b9fe3e304864feb71a0f9378fd3d532。debug非release预算/冻结候选/发布。

精确新Public根C:/Users/Public/ocx-h024-actual-gui-20261115-r1/已准备三个输入及一次性launch/同SID自动采集脚本；新根ACL仅controller/SYSTEM全权、marenben读取，out01额外Modify。仅自有新任务对象权限设置，非修现有ACL/策略/令牌。日志逐项同SID兼容现有句柄读取，失败全部保留，不重启/kill broker/改权限。原普通marenben session7实核Active，但控制器不是该交互Shell，需用户在其桌面启动；无提升/模拟/计划任务或breakaway捷径。

完整状态表归final-status.json；B1/B2/B3及O1/O2/S-D/P复用前轮限定事实，CAP/trust非空1314/5不重跑。公共Start及后继仍拒绝。原件声明副作用/持久Ready与PreStop/后继历史释放、新官方broker路由/全H-S-D生产组合及普通MSI/历史清理仍未完。准备目录/工具缓存/制品留用，未新建账户/Profile，未启动桌面GUI/官方服务。全部历史自动审批删除拒绝未重试/绕过，overallCleanupComplete=false。无业务同步/模型请求/W2/双SID/发布。
