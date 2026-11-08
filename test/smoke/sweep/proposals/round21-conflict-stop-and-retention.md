# 第21轮：真正就绪通过，固定冲突诊断与管理器停止仍未完成

Run42474583331，sourceab5a9a3d8，26过/1失败，accepted:false。272份manifest成员大小/SHA256全部相符；summary SHA256 af13ab259b299afbb8f84a0ee1830a2eba971ce25069841f0d56cf3864c100f2。原目录/private/tmp/ocxd023-sweep-r21-42474583331-download/windows023-sweep-42474583331-1/保持。01～13、15～27过，14失败，不将局部过当成完整端口通过。

A实际管理器省略port，自动55581，在原预算内running/healthy、health挑战证明/PID/port与names.2一致；实际config.toml写入本次端口，只有空客户端目录由夹具创建。B显式占用A端口没有自己的runtime record，但没有公开冲突诊断；其原process_action却返回started，app.log只有start confirmed。终止协议后清理尝试公开manager stop，CDP返回未完成，原进程仍健康，最终严格账本兜底清理和两账户/Profile/ACL均通过。不能因此认定manager stop通过。

源码核查：ControlledProcessRunner对Start/Restart在spawn后立即Started，退出观察接收端被丢弃；stdout/stderr为null，异步非0启动结果无法通知消费者。process_action仍为同步Tauri命令，Stop等待在调用线程执行，可能阻塞WebView。前者为明确源码事实；停止的具体原版耗时尚待直接CLI对照，不依据这轮超时猜测放宽预算或改产品。

## 同期正式候选与测试夹具冲突

正式候选42469646550（5cba）NSIS/MSI已构建。MSI默认与自定义原生普通用户首屏、精确EXE校验通过，默认卸载保留哨兵已通过。随后同账户custom清理测试要求.codex-desktop原本不存在，与前一默认格的保留哨兵相撞，拒绝覆盖既有根。原拒绝保留；安装套件未通过、未进入两SID，不外推后续NSIS成功。真实账户/Profile已移除，原Server策略精确还原。

修正只归test/.github：default卸载先核对哨兵hash证明保留，然后由测试工具清理本次新建的精确哨兵及空父目录。前置根必须不存在；删除前完整树仅允许root/desktop/sweep-retention/随机UUID.sentinel四对象、无链接、内容一致。遇到其他内容保留报错；不递归删除、不修权限、不清日常配置。success回执单独记录testFixtureCleanup，不能替代productRetention proof。实际文件系统成功/既有根/额外文件/内容变化拒绝等检查、原矩阵与发行装配检查均过。

## 原版冲突/停止直接诊断

只在显式ordinary-lifecycle-023模式：原普通账户执行器启动原版A（空Codex目录、静态loopback provider），实际ready后另新H用同一A固定端口启动原CLI，保留原stdout/stderr和30秒诊断结果，核对A记录代际不变、B无运行记录；再运行原CLI stop观察原10秒内是否退出。不会手动删除记录、关闭集成、杀A或扩大生产预算；整个独立诊断110秒运输上限，不算原软件验收。普通执行器精确代际进程、账户/Profile、ACL清理仍必须通过。取得实际冲突文案/退出与停止结果后，产品生命周期变更须按用户最新范围另述理由和确认。尚未修改apps产品代码。
