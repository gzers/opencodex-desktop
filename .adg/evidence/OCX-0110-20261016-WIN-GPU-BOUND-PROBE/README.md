# GPU绑定读取器的受控执行检查

工具分支`feature/0.1.10-windows-gpu-resources@768e2318`已推送读回，ETW待录制分支仍固定54927d2c。本机37项检查通过，包括真实PowerShell根/子代绑定、保留句柄、角色与计数器筛选、正常退出和代际消失读回；未启动桌面产品。受控进程未取得GPU实例，缺失实例不解释为零显存，亦不证明原生产品入口已执行。

首次push CI38864269467于15:17:42读回完整success，五必需作业成功、main-only build正常skip；原run/jobs API与摘要逐项核对。三作业原始日志尚未完整下载核验，不能声称原日志准入。`code-provenance.json`保留收集时的`sourceCiPassed:false`，终态由新增读回说明，不重写历史。

初版离线校验把同一UTC瞬间的七位和六位小数作为不同字符串，失败及原校验器保留；修订按100ns整数核代际，原快照/退出/清理字节不改。`verify-probe.py`核对原件、Git方法字节、实际绑定、全CI状态和清单。后续人工窗口的真实产品快照及恢复尝试另存`OCX-0110-20261016-WIN-823-RECOVERY-GPU`，不倒改本准备结论。

Windows平均RSS+29.221361MiB、历史Windows/Mac资源失败、bd2真实失败窗口及全部原门禁保持；本方法检查不代替600秒性能、单个GL缓冲归因或发布验收。TASK revision2/in_progress/pending不变，不合并、关闭或发布。
