# 只读WPR分配配置准备

工具494fc830已在feature/0.1.10-windows-working-set-pages提交、推送并读回，只新增两个test/smoke文件和CI步骤，产品b4不变。由本机WPR导出配置生成单独的OcxGpuAllocation.Verbose.File：两个collector各256KiB×64缓冲，总配置32MiB；保留进程/映像/虚拟分配/内存信息关键词、VirtualAllocation与ProcessCreate栈，以及系统导出D3D11、DXGI、DxgKrnl的原关键词、级别、栈和启动状态参数。它收窄独立诊断范围，不修改原验收指标、预算或bd2回放约束。

本机18项检查通过，含WPR实际识别/解析生成文件和六个缺关键词/分配栈/图形provider、改变GPU关键词、XML外部实体与超容量输入的拒绝检查。首次push CI38920950285/attempt1的Windows方法作业117040410248成功，18项与实际checkout494逐项核对；制品摘要/大小/CRC、六份提取文件及原作业日志均验证。CI方法字节是Git LF的CRLF checkout转换，单列实际摘要。核验时整轮CI仍在运行，只准入只读方法，不倒称整轮或真实录制通过。

WPR日志是解码转存的UTF-8命令输出，生成profile和原WPR导出按文件字节保留。源码只引用实现提交及摘要，不在文档分支再存一份实现。独立核对本机/CI生成XML的缓冲、三provider参数与来源相同；文件顺序或换行不同不冒称逐字相同。

32MiB仅为collector缓冲配置，不是全进程内存上限，也不证明ETL文件大小、事件无丢失、分配栈可解码、目标代际绑定或会话独占停止。当前生成器没有录制入口，记录executionAdmitted:false、eventLossVerified:false、fileGrowthBoundEnforced:false；真实录制/生命周期方法仍需准备和验证，不能因只读准备通过就占用前台。

23:07:43读回native0、已恢复原目录ID/创建时刻、偏好/HKCU指针及TASK摘要保持，无产品启动、ETW录制或查询回放。RSS+20.040155MiB超+10MiB、原退出失败、bd2实际失败窗口、严格恢复、Mac依赖与其余门禁继续开放。TASK revision2/in_progress/pending，DPI专项取消，不合并、关闭、签名、晋升或发布。
