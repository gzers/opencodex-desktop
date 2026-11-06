本轮 Windows 候选2定向补验已通过，可交集成线接收；只关闭本贡献TASK，不代表正式上游运行包或整版发行已通过。

代码分支 codex/post021-windows-final：63f9dbe265568ba608da631aa83739bf5691755a。产品基线 codex/post021-sync-fixes@ca46bc1f3c35988d206e1a2949c708d4f23aab50，apps树 ca89905344d0d29e761bcb8ffa24d22769c9cb4c 未改。文档来源 docs/governance-main@8b2ec843f29a0adcfb46bc6f5575576b65bab4d5 的 IMP-30/WINDOWS-TARGETED 已冻结在 governance-snapshot/。文档贡献分支 docs/post021-windows-final，最终文档SHA以提交回交为准。

实际执行：2026-11-06 08:17:12～08:21:51（Asia/Shanghai），Administrator SID500/session1；Windows Server 2025 Datacenter 10.0.26100，原生 WebView2 Edg156.0.4314.58。由用户在自己的隔离登录会话启动，driverexit0；没有使用凭据提升或跨会话注入。native-entry/ 为本次入口实录，native-attempt2/ 为原生执行证据，native-independent-review.json 为独立复核，coverage-matrix.json 为六项映射。

CI42056342409 五适用job成功，发行build按原条件跳过。原MSVC artifact13431169501，内嵌checkedCommit56141aecef1608eb44d44dd8ff0eaef84b96676f（父ca46及7a844），完整apps树与本贡献一致。EXE15296000字节，SHA256 f62c474c06cb3fe2f77fe23d38e99c0783d86e727dbd538cfe7b4d36dd8c347d。ZIP SHA256 c5acd34fc5db9e178ea59e935cb9fa6f36f81d7a7e6dd5afe91e1b615c454fc4。运行根为 D:/Users/Administrator/WorkSpace/Test/post021-candidate2-targeted-20261105；隔离源码、缓存、npm配置和数据根，实际管理器PID91648/创建代际见原始身份文件。

运行包：2.97.0-post021.ownerquery.2，15406072字节，SHA256 bba03ba0cf02c7bcd1895f63c670cb4a2f4b5272657145e45701b20d5e0f1957。基于原npm2.97与上游dev0a30c551按owner补丁再provider基线补丁构建；第一补丁十后像、补充五前后像、review四runtime后像和构建器摘要均先核验。真实管理器离线预检及托管安装成功，完整预发布版本、managed/offline来源和实际ocx.cmd入口读回，explicitPath为null；2522个安装文件逐摘要匹配。不是只用另建目录或手填路径冒充托管安装。无效not_gzip包确实失败，旧来源/入口/包体全保留。截图01/02及相应JSON齐备。

全新改名 Post021AdminRenamed → Post021AdminFresh2：先确认无pending、无新目标；可信WebView键盘输入，实际保存按钮，稳定渠道ID。先真实取消确认，完整保存字节和pending保留、客户端未退出；再首次实际确认一次。未注入重启成功、没有先失败恢复pending冒充首次。完整保存摘要 a0e6ad819b5e36d99dad85e303a37f1d11417adee1eaaaac1dd35993bf5ab955 在14个文件观察样本和完成读回一致；knownModels.updatedAt仍为2026-11-05T00:00:00.000Z，recentArrivals仍为空，未删字段或放宽字节守卫。旧正式MSIX主进程51156退出，新主进程77168（及同账号子代）真实启动；新客户端引用存在、旧引用消失、pending清除。首次公开成功约46.87秒，是含实际重启/协调的观测耗时，不是单一归属探针耗时；原10秒/8秒保护未放宽。

失败重试单独使用普通保存contextWindow65591：按SID/路径/创建时间/哈希精确停止自有测试服务50452，原确认框报“配置已写入，但重启尚未完成：官方服务当前不可达…无需先打开面板页面”，没有manage backups/os error2；保存保留且客户端77168代际未退出。恢复同一托管候选服务108104后点击同一公开“重试重启”，约38.71秒成功；正式客户端主进程换为64728，65591读回，新旧引用及pending再次正确。截图06/07保留。两个服务实例共4次自有loopback GET /v1/models，零推理，未执行真实同步。

隔离保护：初始及取消/首次重启/服务失败/重试后共五份15119签名随机挑战摘要，两日常配置始终为46355008abfeb9b9390bd3733504f72d2e1c56c4da0600879455b167389aeed4和a485cf85292e680e43b41b0d674efe7cd5cb22109298dd49a25ed4cd170b58c8。四次边界中其他账号关键进程无代际变化；日常代理66908的SID/path/session/PID创建时间相同。只读观察器49432已自行结束，无须清理。测试管理器91648、服务108104及Administrator正式测试客户端64728保留；没有为收口广泛杀进程，后续若清理必须重新核对实际身份，不能照抄历史PID。

根因与失败保留：逐进程GetOwner超时由运行包owner查询补丁处理，SID/路径/PID代际/祖先和原超时仍保护。新渠道引用reload后只采用provider、未采用该provider的发现基线，会在目录刷新重建updatedAt并触发管理器严格字节拒绝；provider基线补丁修正源端采用，管理器守卫不变。完整预发布识别/离线安装由ca46同源管理器实证。原源码定向构建器5项、上游路由2项、saved_runtime_reconcile19项及Windows实际安装内核1项通过；最初Unix-only筛选0项不计通过且原日志保留。首轮Administrator在启动管理器前读取15119日常代理配置EPERM，五份原始JSON在attempt1-native-failed/；改为15119签名只读采集并验证重放/身份/过期负例，未改ACL或降摘要保护。用户此前缺右引号进入PowerShell >> 等待，脚本未启动；与运行包产品故障区分，原先“无启动证据”观察和失败RCP均在history/、prior-receipts/保留。GitHub短时commit_refs_rules拒绝后原分支推送恢复，未绕过规则改公共分支。

可安装交付方式已实证为隔离TGZ经同源管理器offline托管安装。本候选是本地预发布修订，不能把上游正式2.97.0或现有生产安装源写成已修复。生产仍需上游维护者发布或由集成者明确采用维护的离线来源；最终桌面发行原包/升级/签名/更新清单/回退及共享DMD/IMP/AC结论由集成线收口。本贡献未公开PR/fork、未合并main、未发布、未采用候选为日常来源、未改15119日常配置、未做真实同步或推理。旧材质/RDP性能及全量任务未重复，其历史结论和例外仍取既有回执。
