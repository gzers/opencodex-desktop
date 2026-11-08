# 第16轮全量诊断与一次性修正

- Run 42459241250，source 2a3a8d8feb995cc45394ecdee5a30e186533225d，25 pass / 1 fail，accepted:false。
- 01～13、15～26通过；14失败：user1 runtime_start_not_healthy，stage=wait-owned-health。
- 原目录 /private/tmp/ocxd023-sweep-r16-42459241250-download/windows023-sweep-42459241250-1/ 保留；264个manifest成员大小与SHA256全部核对相符。Actions绿色不代表验收。
- 新维护包2.97.0-desktop023.names.2，archive SHA256 fcd2740ecc32ae1e11c1c79d84c801b2c5d5447ae89e070ab060548d61f75ee6，当前程序/两SID按新policy构建。四格原安装器仍是旧诊断输入，不外推最终安装包通过。
- user1省略端口后实际分配50010；healthz HTTP200、HMAC、PID及端口核对通过。管理器保留stopped/unknown。user2自动端口/冲突/重叠/停止协议未完成，不算通过。
- 两真实账户/Profile、精确进程代际、原ACL和Server策略清理/还原均通过，原失败证据不覆盖。

## 同一批处理

1. 官方运行包windows-secret-acl.ts明确采用protected owner-only DACL；process-state.ts使用该原子写入器。Desktop当前读取器复用了创建文件要求的两条User/SYSTEM DACL，错误拒绝合法owner-only记录。读取专用校验接受protected owner-only或原User/SYSTEM，仍要求实际TokenUser所有者、显式FullControl、无继承、无其他主体、普通单链接文件；创建/重开/目录的双主体合同不变，不修复原记录权限。沿用户已授权的认证状态读取边界，不改端口分配或启动语义。
2. 增加真实Windows icacls生成owner-only记录的成功用例，原创建校验仍拒绝该形态；增加Everyone授权与硬链接拒绝。原五条证明/HTTP/消费者/超时用例全部保留，第26项要求七条均实际运行。
3. 原端口夹具providers:{}没有defaultProvider，实际schema会拒绝并触发配置恢复。改为一个明确的纯模拟provider/defaultProvider，固定模型、禁用liveModels，地址仅127.0.0.1:1，显式允许该模拟本地地址；默认端口仍完全省略，固定冲突仍精确端口。没有调用上游/推理。
4. 对生成函数直接调用采用包的真实configSchema，默认与固定端口都必须通过；失败记录保留。临时H/driver在成功或失败后清理。诊断第02项实际Windows Bun也执行同一检查。
5. 失败以后额外记录实际backend激活校验耗时和原5秒status source的受控结果；不增加生产预算、不改变失败结果、不序列化记录密钥。原大范围CLI对照继续保留。

## 本机定向验证

- /private/tmp/ocxd023-r16-attested-status-local.log：6过/0失败；不冒充Windows两条新ACL用例通过。
- /private/tmp/ocxd023-r16-status-replay-evidence-c/：实际固定Node大文件/完整激活证明、实际Bun1.4.4 loopback三次响应和真实status consumer通过，保持5秒；临时I/D/H/服务均清理。早期错误夹具a/b保留，不算通过。
- /private/tmp/ocxd023-r16-actual-fixture-schema.log保留未允许模拟loopback的原拒绝；后继schema-b.log通过实际schema两个夹具并确认临时目录已清理。
- Windows七条原生ACL/HTTP回归及真实两SID完整端口协议待下一轮，绝不以本机模拟替代。
