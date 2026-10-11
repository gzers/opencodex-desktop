# macOS 0.1.9 / 0.1.10 十分钟探索性对照

官方 v0.1.9 源码 de6862d40389d527527b9db8ef52e186a58b625f，与 main@369ab12da55448fbfdc4d0c9bf03970b56fd6c27 的隔离副本在同宿主顺序采样。两者浅色 / 高画质 / mesh / 100%，概览准备态（Node/npm 已发现、OpenCodex 缺失）；偏好文件字节摘要相同。原官方 tar、源码内嵌、原 EXE 与独立 ad hoc 副本身份见 asset-identity / baseline/provenance / verify。原件严格签名失败，测试副本通过不代表正式签名验收。

0.1.9 于 18:29:37–18:39:37（UTC+8）采集 61 次 / 600.022329 秒，累计 CPU 328.09 秒，平均 54.67963% 单核；RSS resident 之和 284.75 → 287.625 MiB，峰 288.171875 MiB。候选已归档观测为 58.34876% 单核，差 +3.66913 百分点。各角色分解见 comparison.json。两者 GPU / WebContent 均有明显开销；不能归因于自动检查。

数字差超过原 CPU 增量预算 0.5 百分点，作为待调查信号保留。没有连续前台 / 可见性、受控预热或重复配对；独立根的缓存 / schema 状态不同，无主线程 / 网络仪器。不能认定因果回归，也不能记性能通过。原冷 / 热各 20 次、CPU / RSS / 主线程 / 零请求与真实周期预算保持。候选原件引用 ../OCX-0110-20261011-369A-MACOS-IDLE-OBSERVATION/，未复制第二份。

0.1.9 快捷键退出未生效，normal-quit / quit-settled 名称表示尝试：快照仍见 PID19382 和持有锁。候选 PID21610 在基线退出未经独立确认时被启动，曾出现两个测试实例。18:45 确认精确基线可执行路径后，仅向 PID19382 发 SIGTERM 清理；forced-cleanup 证明基线消失、锁可取得，不能记正常退出通过。旧基线沙箱 boundary 沿用旧临时目录，0.1.9 没有当前 boundary 拒绝机制；实际 root 独立。

候选 restored-after-baseline 为早期快照，不能计锁就绪；restore-settled 与 single-candidate-after-cleanup 证明正确环境 / 根、锁 metadata PID21610、原两份备份摘要不变。18:48:50 再次确认唯一候选和日常 PID75764 / 启动时间不变，18:49 原生 AX / JPEG 证明准备态恢复。没有通过观察工具重开基线。

只复制测试进程归属、允许环境字段与备份摘要，不导出偏好载荷、原始环境或其他应用元数据；基线 attributions 8 条全部 admitted=true、同测试 resource coalition。AX 以原字节 gzip 保存；JPEG 及 JSON / 方法按字节复制。provenance 与 checksums.sha256 提供独立校验。IMP / TASK 保持 in_progress；release_authorized / release_accepted 为 false。
