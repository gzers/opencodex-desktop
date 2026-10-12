# Windows 关闭隐藏与顺序重启

2026-10-12，精确 `main@afff490` EXE，原根单实例。证据归 `.adg/evidence/OCX-0110-20261012-WIN-CLOSE-HIDE/`。

## 真实关闭按钮

PID29936、主HWND852848，原生DPI192。偏好keepProxyOnClose=true、stable、auto-check false。通过Computer Use实际点击原生标题栏关闭按钮后，窗口从可用清单消失；独立Win32观察visible=false、minimized=false，原PID仍存活。生产app_foreground=false，云母退为solid/inactive、DWM attr1。

此时实际document.visibilityState仍为visible，与此前系统最小化的hidden场景不同，原件如实保留，不把原生hide与WebView visibility混为一谈。没有运行面板代理，因此不证明代理保持运行。

## 恢复限制与有意结束会话

Computer Use对已返回窗口的重新绑定返回“window id 852848 was not found”，直接激活原窗口对象返回“window is not a usable app window”。没有借其他窗口或直接Win32/show API绕过；没有实际托盘输入，恢复不计通过。错误是工具对隐藏窗口的限制，不据此报产品托盘故障。

随后明确结束该测试场景：向原监督器写stop.request，独立确认旧PID29936与监督器40736均已退出、候选进程数0，保存终态finishedAtUtc=09:26:00.561Z。此为受控清理，不是正常退出验收。

确认终态后才复用同一根、同一EXE启动新普通令牌实例，PID41328、监督器44628、exec session91566、主HWND787306，创建时间09:26:29.739Z，未与旧应用重叠。令牌elevated=false/type3，源码与启动日志匹配；新窗口可见且foreground Mica。不可将这个新窗口称作旧窗口恢复。

偏好SHA仍为8498f7cf…，themeNeedsImport=false、system/stable/auto-check false；原固定备份保留。此前同候选已解除通知与轮换旧异常的resolved状态跨启动保持。日常0.1.9 EXE SHA仍为43e06bc3…；没有本机安装器、包生命周期脚本、真实数据迁移或日常替换。

19个原始文件及顺序/身份/状态断言冻结并重验通过。只通过关闭隐藏及本次顺序受控重启和持久读回；托盘恢复、正常退出、运行代理保持、管理员/全DPI、安装更新、配对性能及真实周期继续待验。原TASK/脚本确认、预算/既有性能failed不变，IMP in_progress，不关闭或发布。
