# 待授权的上游运行包修复交接

问题：Windows正式Codex重启在桌面进程归属查询10秒、app-server归属查询8秒处超时。逐进程CIM GetOwner的RPC成本使整个枚举失败，不能按无进程处理。

已审阅补丁：本仓库test/smoke/post021-windows-final/runtime-windows-owner-probe.patch与runtime-windows-appserver-owner-probe.patch，基于发布@bitkyc08/opencodex@2.84.0。只替换查询：同一原生进程句柄取得owner SID、路径和创建时间；保留当前用户/包路径边界、PID代际、祖先保护、窄匹配、原超时、退出/拉起和未知状态分流。四处变更及2021文件复制摘要已在runtime-patch-review.json独立核对。

Windows源候选原生证据：默认桌面旧48960退出、新92496启动；app-server查询约1039ms，catalog约1986ms，原10秒/8秒限制保持；真实管理器公共重试显示已重启，config.toml、模型路由和contextWindow65547读回，其他账号桌面代际不变。原发布包失败保留，候选不冒充已发布2.84.0。

Mac补丁重放和4项协议测试通过，仅注入exec回调，非Windows原生证明。初次未安装PATH Bun及Mac没有GetSystemDirectoryW导致的失败保留；补测试专用PowerShell路径seam后通过，不修改产品源码或候选依赖。

目标上游：lidge-jun/opencodex；当前账号pull=true、push=false。只读核对最新main仍有GetOwner，且main已增加命令行/兼容重启上下文，不能将2.84补丁未经移植直接投到main。若获准提交上游PR，需保留最新CommandLine/base64上下文及包兼容路径，加入相应回归，再提交独立PR；不声称本批已获上游接收或已发布运行包。现有补丁、哈希及实际原生证据均已准备可审阅。

公开PR/issue会向第三方仓库发出消息，目前没有执行。不得静默改本产品的npm源、覆写上游版本或在用户日常安装上打补丁。取得上游可安装集成包后，应再由隔离Windows默认adapter验证，再进入本产品最终发行安装器/升级/签名/回退门。


## 只读接续核对：上游实际集成分支

已按MAINTAINERS.md核对，外部贡献PR应基于最新dev并以draft提交，不能从main直接开功能PR。固定dev提交0a30c551304927c93c26a96c034bd39a166c7460的桌面及app-server文件仍含逐进程GetOwner，两份Git blob身份和SHA256已验证，见upstream-dev-review.json。因此尚不能用“dev已经修复”解除本批运行包依赖；2.84补丁移植时须保留dev新增的命令行/兼容上下文。尚未创建fork、PR、issue或发送外部消息，等待当前用户明确授权。
