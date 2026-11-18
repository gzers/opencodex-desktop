# TEAM68 r7 独立读回脚本配对

主线 040f411e6d48d97bde6e7b2e6c22860e547b6173，治理 55dff7d7b6aad0357d310757981641c0f38d5a03，TEAM68／IMP207。源码 a8d7a08002c3af21b2c7ad8f9bcc06cda9d107d0 只改测试 launcher 三处。待证伪判断是合同指定的四处替换即可得到固定配对，同时保留 ACL 与所有其他暂存输入。

启动器 30,158 字节，SHA-256 6b553204560d59ba7a292f64c322404028eceb1a0899661f5f9dc9d76953e990。观察脚本原字节附于本目录，6,394 字节，SHA-256 f3a0c5ed9f8ba698d0b5d3e37d1b1209b4b88fa5ee4714a5fe06c1ff000aba5f。CMD 继续使用 85f56dd180ee907b70e68ed531d90a4e5a480cb104264bdad4890053e5da60f2。暂存前核 Git blob、规范路径、目录成员与 ACL；获取两个目标的独占句柄后再次读回旧摘要，写入后刷盘并在原句柄上验证新摘要，释放后再次核全量成员、摘要和 ACL。只有两份指定脚本字节变化。实际 PowerShell 5.1 解析及摘要门通过，混用旧 launcher 被拒绝。

首次准备在清单路径分隔符转换处 raw1，Parameter 'oldValue'。此时尚未写入。修正后准备、暂存和 PS5.1 核验均 raw0，stderr 为空。观察时 marenben session 7 活动，range-r1/out01 为空，私有夹具根未创建。没有执行 Host、DiagnosticBefore、Explicit 或 DiagnosticAfter，没有新增暂存临时文件。本批未触碰旧现场，也未进行清理。
