# 第13项：Windows 本机连续负责启停闭环

2026-11-09，Asia/Shanghai。用户后继指令授权Windows沿最新Git直接原因分析、最小修复及定向验证，无需每轮交Mac中转；本文件覆盖旧交接中等待Mac决定修复位置的下一动作。第13项目标为真实隔离普通user1启动确认、成功后立即正常停止及精确清理，原归属、权限、就绪、客户端实际还原与退出断言以及30/10/5秒预算保持。不扩大到双SID、安装器、core-sweep、全量、第18项独立修复、日常配置、真实业务同步或发布。

## 检查上一轮

最新基线代码d3b34a234e32b63365672052772c2c72c719f97c，文档23715445a45313110bf117861003b63cf9797a9d。原批次ocxu1-20261109-214306启动15072ms通过，唯一started回执，PID14788/53668原生身份、监听和健康认证一致。随后Stop在187ms失败lifecycle_stop_unknown；服务原句柄清理前未退出，正常退出码为空，客户端还原未确认。清理已完成，但整体未通过，因此继续原因分析。

## 明确假设与修复前反证

先写假设H1：Stop在发送官方停止前的生命周期绑定或原生Witness阶段因HOME/Job/租约不一致被拒。读原实际路径证据：Desktop HOME为扩展本地盘符路径；维护包生命周期模块用Bun realpathSync.native(home)作为签名内HOME与Job摘要输入。

用该封存程序内Bun1.4.4，在测试自有目录读取同一路径两种拼写，不启动服务。结果为：输入普通路径返回普通路径，输入扩展路径仍返回普通路径；Desktop消费端旧key只替换斜杠/小写，保留扩展前缀，所以两key不同。更新H2：合法等价HOME拼写使Binding.verify_proof及native Witness.live的前置比较失败，足以解释未发POST、服务仍活着的Stop失败。实际完整产品因原日志未记录具体阶段，还需新制品真实闭环证伪。

隔离Rust小crate直接提取产品Binding源码及单元测试（仅排除native Witness模块），补回归用例：签名内普通HOME配Desktop扩展HOME。旧实现2过/1失败，失败为该回归断言；最小修复后3过/0失败，包括原字段篡改、错误Job和签名/请求上下文反例。此小crate不能替代真实native Witness，后续构建会执行完整产品相应测试。夹具首次rmSync目录清理报EISDIR，保留结果后仅修夹具为rmdirSync并核目录消失；另一次offline依赖缺失原日志保留，经127.0.0.1:3060下载固定依赖后取得真实红绿结果，不混作产品运行失败。

## 最小修复及预期

代码61080a1bc644e5abb4ef45794344a7185542b74a：仅在Binding与native Witness两处HOME比较调用已锁定dunce1.0.5的安全等价拼写转换。签名内原始HOME不变、Job名摘要计算不变、HMAC原字节不变；SID/PID/创建时间/ACL/Job成员/租约锁/实际退出与客户端还原仍全部校验，不改运行包，不改预算。回归还拒绝不同目录、设备命名空间、扩展UNC、驱动器相对、尾随点/空格，以及擅改签名tuple的HOME。

代码fdf5d4db551f71cbf493dc97b428f8b9a9364a7a：仅在现有core-build准备中增加lifetime binding与attested Stop两组定向测试，成功才封存制品。提交跳过自动全量CI，显式purpose=core-build；没有选择默认core-sweep。

预期：定向单元测试通过；新同源Desktop在原30秒内确认启动，立即正常Stop，在原10秒内完成官方还原、退出、Job清空、租约和端口/记录消失，再精确清理。任一新失败先保留首错和原证据，按新线索更新假设，不盲重跑。

## 实际结果：本次单用户启停通过，缓存清理有审批阻点

构建42631183420完整成功，Windows产品绑定3项、认证状态/Stop边界9项定向测试全过，其余测试未执行。新artifact13681803354/ZIP304980804字节/SHA256 a44664fb38780898d6ce1907f8c277f61c996d1afd008d9ec1935cf9ed17824a，9成员、源码、policy、lifecycle.3摘要均核验。真实批次ocxu1-20261109-223434仅Start1次、唯一started回执，15351ms确认健康，PID81424/61934与原生归属/监听一致；立即正常Stop1次，8727ms完成实际还原和退出证明，Bun/Node/cmd及driver在清理前原句柄均exit0。账户/Profile/目录全删、残留进程空、ACL恢复。RDP39采样全Active。第13项本机普通user1有界场景通过，已停止进一步验证；不外推关闭整版、双SID或第18项。

只有自建绑定夹具target及两个ZIP.parts缓存目录删除被自动审批拒绝（仅blocked by policy），准确路径和存在状态已记录，不绕过、不声称全部磁盘产物清空。[四项原证据、修复原因及缓存清理阻点](../../../.adg/evidence/0.2.23/user1-start-stop-20261109/ocxu1-20261109-223434/README.md)。

## 派发时记录（保留来源）

构建42631183420已派发一次，实际source=fdf5d4db551f71cbf493dc97b428f8b9a9364a7a，等待定向测试/完整制品。运行包继续lifecycle.3（15413397字节，SHA256 ec14baaee354c6729d1d2545ab9165d207b14a3a4239f354662163087eeb0739）。新artifact/ZIP身份未产生前保持未知。

分析原件暂存D:/ocx13-analysis-20261109-220054，含每版假设、Bun路径观察、Rust修复前/后日志；新运行前将与独立批次证据一并封存，原失败批次不覆盖。Windows知识参考development-knowledge的进程/锁与已锁定dunce源码；其来源资料命令未执行。
