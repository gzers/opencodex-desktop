# Administrator账号只读预检准备

用户授权本机Administrator用于验证，并指定D:/Users/Administrator/WorkSpace/Test。当前执行身份仍EZIO-MINI/15119，正式Codex根38560及其子进程均归15119；只读检查未见Administrator正式Codex。Administratorconsole为历史已登录会话，并不自行等于无日常配置的可丢弃环境。密码未写入脚本/日志/仓库，未执行认证或切换登录。

代码ed47d11f9/42c031c3f新增formal-msix-preflight.ps1，默认要求Administrator，显式TestRoot输出新目录，拒绝越界/重解析点/覆盖；只读列当前账号包、owner/session/PID创建时间、祖先链和现有配置存在性/字节数/摘要，不读出配置内容，不执行启动、停止、重启或配置改写。D:/Users/Administrator/WorkSpace/Test/POST021-PREFLIGHT.ps1已创建，不覆盖已有文件；需在Administrator自己的桌面执行，当前15119提权shell不代替不同owner。

真实WindowsPowerShell5.1检查：默认Administrator负例在15119拒绝且不创建结果目录；当前账号显式只读运行exit0/正式包1/既有Codex状态true，不能用于隔离重启；显式测试根正例exit0；新Test入口在15119拒绝。原始日志随附。预检不是正式adapter重启验收，actual Administrator账号结果尚待，不能据工具准备关闭AC。用户已答应协调GPU安静窗口，具体时段也未给出。

TASKin_progress/RCPfailed，无CERT。保持不合并main、不发布、不真实同步、不改15119日常配置和进程。后续先核Administrator结果，再在可证明隔离根/测试实例中完成产品Save→默认restart→新进程→配置readback。
