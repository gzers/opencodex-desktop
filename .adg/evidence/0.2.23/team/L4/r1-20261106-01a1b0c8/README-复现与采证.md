# 复现与后续账户采证

本目录是L4首批证据。代码源在codex/0.2.23-windows@42d2a4a1770a5be1050dec3de641efab77a65917；目录只保存日志、回执与生成辅助，不复制产品源码，不提供另一份实现。

1. 在Windows取得上述代码SHA。完整crate先以L0指定MSVC/SDK环境执行编译及相关测试；本轮GNU依赖环境及libtest清单适配只用于本机原生逻辑验证，不能顶替MSVC/安装器。
2. 若需单独构建native helper探针，在源码树外新建Cargo driver。所需依赖为windows-sys=0.61.2（Win32_Foundation、Win32_Security、Win32_System_Threading、Win32_UI_Shell）和serde_json=1.0.151；[[bin]]的path指向代码树test/smoke/windows023-user-scope-probe.rs。本目录native-driver保留本次实际driver清单和lock；其中本机绝对path需替换成接收方真实代码路径，不复制helper实现。
3. 编译时仅在该命令进程环境设置OCX_WIN23_SOURCE_COMMIT=上述完整SHA，OCX_WIN23_HELPER_SHA256=实际代码树windows_user_scope.rs的SHA256。探针内嵌两字段，采证脚本会核对本地源码/版本及二进制摘要；不得把旧probe用于新源码。
4. 在账户持有人的受控通道执行如下参数化命令，使用分配给该账户的预期SID以及全新输出目录。不会请求账户密码。

~~~powershell
# 参数由本次接收方填写；不修改PowerShell profile、机器环境或系统账户。
& <代码树>/test/smoke/windows023-user-scope.ps1 -ProbeExecutable <已核验的探针EXE> -ExpectedProbeSha256 <64位SHA256> -ExpectedSourceCommit 42d2a4a1770a5be1050dec3de641efab77a65917 -ExpectedSid <该账户预期的完整SID> -OutputDirectory <新的绝对证据目录>
~~~

仅在明确需要对侧验证时加KeepFixtureForPeer；否则脚本删除自己唯一创建的windows023-fixture-*子目录，保留receipt/probe/stderr。另一账户可指定PeerPrivateFile，路径必须是保留的windows023-fixture-*/windows023-synthetic-secret.txt，只验证合成文件访问拒绝，绝不传日常配置路径。若显式保留fixture，后续按实际拥有者/精确路径记录清理回执，不能自动清理其余证据或旧数据。

这条脚本只证明该账户native helper的SID/OS目录及private DACL，不启动管理器/服务、不作DPAPI测试、不证明GUI并行或端口归属。第二账户通道及完整同源候选到达后，还须依WINDOWS任务完成两SID并行、同SID跨会话/提权、IPC/凭据/进程/端口及NSIS/MSI适用项。
