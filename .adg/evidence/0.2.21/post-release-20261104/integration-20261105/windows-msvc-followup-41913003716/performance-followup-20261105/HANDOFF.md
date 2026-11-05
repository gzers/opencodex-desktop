# RDP性能续查：尚无可采用修复

本轮仍按原B=1000/60、p95≤2B、over6B=0、静止/滚动各30秒、高档/原玻璃配方检查。使用原MSVC41913003716/artifact13366782216的EXE，仅本进程带--disable-frame-rate-limit诊断开关；没有落地产品参数或动效改动。面板新修复的MSVC正式通过见native-msvc-retry-fixed-20261105，不扩大为性能通过。

rate-only-replay/process-result.json记录真实进程退出1，完整stderr及失败JSON存在。有效前台静止约32.57秒，rAF7146、WebGL7147次，p95约1.5ms但max3150ms、over6B=10、longTasks10；滚动30秒实际前台，p95约17.2ms、max19ms、over6B=0。整阶段因静止长帧仍失败。这说明取消帧率限制会过量提交装饰绘制，不能仅因p95变小采用，也不能把rAF频率等同于显示器或RDP实际呈现频率。

在同一测试进程临时给实际装饰WebGL drawArrays加60Hz提交间隔（未改原QA rAF采样、CSS玻璃或档位）继续对照。统计实际提交1530、跳过32580，最小提交间隔16.7ms。采样静止阶段结束foreground=false，滚动前准入再次拒绝；这一轮是无效前台样本，不能使用p95或draw计数认证改善。rate-draw-cap-stats.json包含整个安装周期的间隔，最大2848ms也不能当作有效前台性能结论。请求用户提供约两分钟不操作RDP桌面的可用窗口，期间不继续抢前台。

GPU负载只读记录发现本会话DWM以及另一个Active RDP会话的DWM/Edge共享GPU，未停止任何其它会话/日常进程，也不能单凭负载归因。尝试WPR GPU profile前确认not recording，启动返回0xc5585011“Failed to enable the policy to profile system performance”。无录制开启，再查询仍not recording；未修改策略或提权。现有Chromium trace仅定位GPU/compositor等待，仍不足以在DWM、RDP帧投递与驱动排程间定因。

性能尚未修复，不能降档、弱化玻璃、放宽预算、以软件渲染或隐藏内容等诊断替代成品。正式Codex默认adapter仍缺一次性账号/会话，保持只读停点。目标未完成，原始失败保留。
