# macOS 隔离候选十分钟观测

源码 main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27，唯一测试 PID97688；18:05:43–18:15:43（UTC+8），61 次、每 10 秒采样，共 600.064179 秒。场景为浅色 / 高画质 / mesh / 100% 的概览准备态（Node/npm 已发现、OpenCodex 缺失），不替代就绪态。

应用及同 resource coalition 的 WebKit 合计 CPU 累计增加 350.13 秒，平均 58.34876% 单核；应用约 7.89%、GPU 约 34.17%、WebContent 约 16.28%。这是进程累计 CPU 时间差，不能解释成整机占用或证明具体动效 / 调度根因。RSS 为进程 resident 之和，249.83 → 242.55 MiB，峰值 265.97 MiB，不是去重物理内存。

采样进程身份稳定。更新缓存、偏好和调度文件 SHA256 / mtime 不变，但不能据此证明零网络请求。采集器本身 CPU 0.2364 秒，61 次读取合计墙钟 2.7732 秒。after-process.json 为 18:17:44 的独立快照，证明正确隔离根 / 锁仍持有、两个备份文件摘要和日常 PID75764 / 启动时间保持不变。

初末 AX / 截图未连续证明前台 / 可见性；无主线程、网络或系统调用跟踪，10 秒轮询可能漏掉短命辅助进程。尚无受控配对 0.1.9 基线，性能门禁未评定。正式签名、真实周期、更新安装及双平台设备门禁仍待完成。IMP / TASK 保持 in_progress，release_authorized / release_accepted 为 false。

provenance.json 记录源文件和转换。before.png 原件实际为 JPEG，归档仅改为 before.jpg，字节不变。candidate-attributions.json 仅保留 admitted=true 的候选 coalition 记录；未归档其他应用元数据或原始 launchctl 环境。cpu-breakdown.json 由首末样本推导，其他采样 / 方法 / 图像原件按字节复制。checksums.sha256 覆盖除自身外全部文件。
