# Windows NativeLease原始字节锁与双SQLite互操作

源码b8ab9b321af3563896b83bd02b087e82ae84d648/tree431e67fd8b4b2d282c06d9d8c557ddea00266385，沿TEAM15/IMP141/r5，公共Start拒绝保持。本批在Windows originals内实现private nonClone/nonserde/nonSend的NativeLease原语及原生private file句柄只读核验，不改共享四trait/HMAC/官方文件、Mac或日常配置。

按固定官方9份源及SQLiteWindowsVFS确认，原claim→N→L→K次序：claim对pending/reserved/shared512独占；N/L/K先按WindowsVFS取得shared510，再reserved1排他，全部FAIL_IMMEDIATELY，不busy-wait、不在生产开SQLite或执行PRAGMA/recovery/init/schema写。保留原文件句柄、父pin、原ID/8安全投影/private TokenUser-SYSTEM/noADS/singlelink/原字节，拒绝sidecar/WAL/未知header/缺席，原范围Unknown拒绝。生产路径仅原Token Profile、原KnownFolder DEFAULT_PATH0x400、SID和canonicalCODEX_HOME摘要派生；HOME/USERPROFILE/LOCALAPPDATA伪值在独立子进程不改变namespace，没有创建真实namespace。

每次check核同原deadline/token/原文件及path，错误poison不再准入，内核锁留到作用域释放，释放自有锁不让过期/失败变成功。真实raw writer写未被SQLite范围覆盖的字节会被检测拒绝，冲突保留，明确不承诺锁阻止任意写入。

| 执行 | raw | 结果 | ms |
| --- | --- | --- | --- |
| library01 | 101 | 59过1败，首错正例测试误给新deadline | 155021 |
| library02 | 0 | 60过：修测试原截止并加失败poison | 103493 |
| library03 | 0 | 窄选7过1helper忽略；新增raw漂移反例 | 125063 |
| library04 | 0 | 窄选8过1helper忽略；固定Bun互操作 | 109003 |
| product01 | 0 | 最终custom-protocol MSVC debug bins | 141575 |

首错coordination_tests.rs:71原InvalidData/raw101，原生产no-extension断言正确；只把正例每次end()改为一个原deadline，未放宽时间/原件/竞争断言，原失败源/log/result/EXE/scene保留。首两轮宽selector重复了未变的54路由状态例，后两轮改窄选，不宣称完全无重复。忽略项是由父测试实际执行的私有namespace child，不冒普通生产通过。private expiry测试先被延长期限检查poison，之后短私有deadline重查不单签真实时间流逝；expired factory、no-extension和失败后禁止重入均有直接反例。

真实rusqlite竞争者对4处BEGINIMMEDIATE均被原生锁挡住，N/L/K普通读可共存、claim普通读拒绝，释放后BEGIN成功；SID/目录/文件原ID/8投影/内容不变。固定bundle Bun115281496字节/SHAcb1d8e6564e655e49d30cead359750a1cbbd12f42651caa4b62244d7bd93c244逐字对原bundle成员。8个独立Bun SQLite子进程只运行自有合成库脚本，不导入官方服务模块；持锁4次原raw73且其SQLitebusy事实真，释放后4次原raw0，仍未写库字节。73为测试helper在实际busy分支设置的原退出码，不能把它冒官方查询码或改写0。

NativeLease::acquire_existing的实际Owner/source/原namespace绑定仅编译，未实际工厂/official init/owner journal receipt或root move handoff/reanchor验收。没有把原语插为生产变更许可证；完整官方副作用、初始化、路径交接及原包H-S-D闭环继续未完，原working unchanged拒绝仍在，Start未开放。

最终116份产品输入/70份最终测试Rust对最终工作字节/Gitblob行尾等价，原前端32份字节复用不变。

| 制品 | 字节 | SHA256 |
| --- | --- | --- |
| opencodex-desktop | 77689344 | b6bf623794a951eb73a4f4ba9d136a38559bb7cd692fd28fdeb2de05ba44ebba |
| ocxd | 5398528 | a6a4e8d7ef4a169a2d7409dd5faa82cd2a7106ba6236549e9124a8a5c48161b2 |
| ocx | 306176 | 660f596eb98688938407863f69b0e4dde289106659ababf38c45a49ea45fa7f4 |

共39保留场景/168文件观察/1278108字节私有ZIP内容核验。已知合成ADS另存观察项，ZIP不是NTFS身份/硬链/安全克隆；原scene均保留，无控制器删除尝试/旧删除拒绝重试，无账户/Profile/Public根/GUI/日常策略变化。首次GUI失败broker40588/Public20文件8拒读、B2 CAP/trust非空、普通MSI及全部旧清理缺口继续，overallCleanupComplete=false。完整矩阵在final-status.json，allAccepted/productionExecuted=false，不业务模型请求/W2/双SID/发布。原生原语Bun子进程不冒官方服务或产品进程0。
