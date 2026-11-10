# Windows W0阶段贡献接收复核

2026-11-11。本对话按IMP-34接收Windows分阶段Git贡献。本轮接收原生后端/模块注册，不在Mac伪造Windows执行，不代替W1或最终W2验收。

## 固定来源及核对

- 共同基线main 0828a3955615df0a2a3e52fb96ab8249f2e04d1b；贡献代码最终b2e89665d2864ea8e29bbaa91ac7843dff7ac4cb；实际收取文档0eef59a0114f9aea6a33d71e2b680192476939c8。早期ls-remote曾显示第一批4ca34d937，实际fetch后已含第二批；接收先固定实际取得的提交，不能沿用陈旧远端列表。
- 最终净树仅13个W0独占源码/测试宿主文件，未碰共享根/Schema/父模块/安装器/CLI。第一批误带target的历史未并入main，按最终净树取实际源码，逐文件与贡献Git原件比对；本机只在父infrastructure/mod.rs增加cfg(windows)模块注册。集成提交main 2d68a28bc。
- 34个原证据文件、3批manifest所有条目长度/SHA逐项一致；第一批4项及第二批普通/提升各11项的原结果保留；实际可执行Rust/PowerShell/Cargo等来源文件与对应执行提交逐字节匹配。唯一文本差额是test/windows024-native/.gitignore的Git LF与Windows CRLF，已分别核对两种长度/摘要；它不参与执行，不把差额改写成全文件逐字节相同。
- 初次按陈旧4ca34d937收取的两份变动原件在receiver-owned历史目录留存后承接0eef最新原件；初次不完整接收报告另名保留，不拿零批次结果当通过。Windows原件/原失败未改写。

## 接入边界

贡献提供TokenUser/session/elevation、私有目录/记录/路径文件ID、原创建句柄、Resume前Job绑定、完整受控成员/原退出码和IPv4/IPv6监听读回。独立宿主直接引用apps实际源码，普通/提升同一个SID不等于双SID；各项事实不代替共享HMAC/ready、配置事务、总预算或正常Stop整链。

本机已注册模块，尚未把产品进程/状态/Stop调用接到OwnedRuntime。模块README的“未注册”是贡献时状态，此集成记录说明父入口已接；不因此改Windows独占源码或声称生产链已完成。Windows CLI/PATH/安装器W1依旧等实际共享接口，不先猜持久格式；W2最后固定候选按真实不同SID/控制台与RDP等受影响项验收。

## 清理仍未完

Windows回交明确4个早期ocxd024-process-68932-{0..3}失败小目录仍存在，内容已归档cleanup-readback.json；其递归删除被当地自动安全审查以blocked by policy拒绝。此处保留原拒绝，不换工具绕过，不记已清理；成功测试自有进程/Job/句柄和目录的清理证据与这4项分开。没有重启官方服务/日常客户端、创建账户/Profile或修改官方2.99。

本机证据.adg/evidence/0.2.24/windows-w0-receive-r1/保留接收核验；原Windows证据仍在.adg/evidence/0.2.24/windows-dev/。此时可复用W0原生技术/组件证明，产品调用和未完清理继续沿IMP-34保留，不另跑旧r5可行性链。
