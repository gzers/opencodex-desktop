# 0.2.24 官方客户端三锁协调

revision: 1；2026-11-11。归IMP-34 P24-04，承接client-artifact-policy-r1及macos-client-originals-r1；代码main6dece428ad3e790b1d4d7efcb9ed070371c45e9b。当前Mac原生协调/锁内原件读取通过，非完整客户端恢复或冻结候选，accepted=false。

## 入口及不变边界

infrastructure/official_client_coordination.rs通过当前RuntimeHandle对应的完整版本化程序准入、官方2.99.0/tgz b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c身份和全镜像树核验，再以所选Bun运行Desktop内嵌client_coordination299.mjs。脚本不是运行包补丁，官方源文件不复制/替换。版本/来源不匹配、损坏程序、已有不安全路径、非canonical或非当前私有事务控制根不回退至其它CLI或系统程序。

HOME、CODEX_HOME、OPENCODEX_HOME分别来自明确当前上下文，不从宿主继承代理/凭据/BUN_OPTIONS/NODE_OPTIONS或其它环境；工具PATH受控。官方resolveNativeProfileContext再次核对实际CODEX_HOME和OpenCodex HOME；官方包声明必须是@bitkyc08/opencodex 2.99.0。辅助进程只调用原样官方resolveNativeProfileContext、withNativeMainExclusiveClaim、openCodexCoordinatorTransaction、withCatalogWriteSerialization，按claim→N→K顺序取得排他；没有再实现另一套文件锁/命名空间，也不运行服务、模型、同步或credential API。

在取得锁前，已有claim/N/K及各sidecar必须为当前uid、600、普通单链接文件；不安全旧对象在进入官方可能收紧mode的函数之前拒绝，不替用户修补权限。官方生成的新稳定锁只释放、不在产品入口删除；只有本轮明确创建的私有测试协调文件由测试收尾删除。官方namespace已有目录的身份/权限检查继续交官方实现，本批没有声称独立native ACL权限审计已由这些POSIX mode检查替代。

## 有界回调与释放

Rust同步作用域拥有唯一直接Bun Child及有界stdout管道，只接受两个固定schema回执held/released。每条最多2048字节，重复JSON键/额外字段/第三条/不完整行拒绝。operation、真实Child PID、上下文摘要及三件实际dev/inode互相核对；失败仅保留固定stage，不把原始异常文本、Key、参数或私人路径放进错误输出。

辅助进程只有在官方三个callback内部都已取得锁时发held；在原finally全部释放后才发released。释放通过私有operation目录内create-only 600标记，内容绑定原operation；不写控制根外的凭据/客户端文件。回调最长2秒，完整操作仍受传入的绝对deadline约束，包括来源核验、导入、工作、释放、原Child退出以及最终版本/全镜像复核，不能给某一步另开新总秒表。过期或helper已退出时Lease::check拒绝，调用结果不能因释放成功把原回调失败改成成功。

**作用域借用不是官方同进程CatalogWritePermit，也不是服务归属/正常退出/文件补偿授权。** 官方permit仍留在官方JS registry内，本批只证明跨进程排他和锁内读取原件。真正客户端写入仍须独立当前生命周期/原生文件身份/权限、写前核对、受保护持久事务和对应协调的完整执行证据；不能只因为check曾成功就允许一个延迟写者在锁释放后写盘。

失败Drop会先请求释放、短暂等候；确需清理时仅可终止本入口持有的直接锁helper，永远不发信号给runtime/service或按名称找进程。该清理不能计为正常服务Stop或协调成功；正式生命周期必须接收独立清理/失败证据，不能用它补通过。控制材料由外层原事务保留，本入口不自行清目录。

## 实际验证

5项定向协议/来源检查通过：固定消息及未知/重复字段、固定失败stage不泄露任意输出、过期调用不访问来源/控制根、synthetic/未知官方归档身份拒绝、实际cat→stdout管道的长度/第三条/不完整行拒绝；最终编译/格式/全目标严格Clippy通过。

另在新/private/tmp/ocxd024-coordination-*/私有profile中以固定完整descriptor e0464990b4ab42a519332ba567f8c866aca712621570c5fc22230eea2d5d5fe3供应原样官方2.99，实际生产协调入口验证通过1项：

| 检查 | 实际结果 |
| --- | --- |
| claim/N/K实际排他 | helper持有时，Rust另开的三个SQLite连接分别BEGIN IMMEDIATE，真实返回busy/locked。没有用文件存在或日志猜锁。 |
| 原件读取 | 在这三锁内实际调用Mac原生Originals::begin，留存的original frame原件及摘要保存。 |
| 释放/重获 | 正常released＋原Child exit0之后，三个独立SQLite连接都可重新取得并回滚事务，文件身份保持。 |
| 回调失败 | 固定失败原样返回，锁仍释放并可重获，不把释放成功当工作成功。 |
| 有界过期 | 工作超过helper窗口返回失败，后继三个SQLite排他均可取得，不以失效借用继续宣告成功。 |
| 旧权限 | 只将本轮新建N锁改为644作反例，产品拒绝且保持644，测试自行还原自己夹具权限后收尾。 |
| 外来HOME | 合成官方journal绑定另一现存HOME，K拒绝，工作callback不运行、journal原字节保持。 |
| 客户端与程序 | config.json、Codex config.toml、catalog原字节保持，无auth.json或runtime-port生成；最后全镜像核验通过。 |

本轮全部夹具供应和检查约94.596秒；首个完整协调/原件操作4705ms，使用显式5秒测试调用deadline。它不是完整正常Stop10秒验收，也不证明所有硬件下都有该剩余时间；正式Stop接线仍必须在原10秒合并预算中验证。官方namespace目录本轮已存在，只创建私有CODEX_HOME摘要对应的数据库，未声称验证全新系统账号第一次namespace创建。实际用户为本机当前uid的隔离profile，不当双SID证明。

原生轮代码来源为a19669114＋本批准确补丁，source.json含实际算法/helper/测试源和binary摘要。原生后只新增pipe reader独立单测，算法/helper原字节未变；5项最新协议检查通过，没有因新增测试重复70秒供应。未修改官方源码或日常配置，不运行真实业务同步或模型请求。OS sandbox实证拒绝外网/日常HOME读取，仅自有夹具和精确私有HOME hash的N/K数据库/sidecar及N采用临时文件可写，没有开放整个账号namespace。

## 留证与未完

证据.adg/evidence/0.2.24/official-client-coordination-r1/保留假设/准入、完整原生结果、真实原件frame、stdout边界、source摘要、检查及manifest。自有测试/程序helper均退出，本轮新数据库再由独立连接取得/释放排他后删除；测试runtime、profile、复制binary和核对后的重复临时回执已清理。未重构.app/DMG或发布。

下一步仍是客户端原生写前身份核对、原件应用/冲突保全、受保护的持久恢复/回退，以及正式Start/Stop的原件/阶段/服务代次绑定。config/Grok/Agent与新CODEX_HOME目录缺席/创建/撤销范围尚未补全；本协调通过不把上一轮catalog/cache/audit未还原改为通过。Windows目前无W0 HOME/Profile/W1后继Git贡献，仍由既定独占写集接入；本入口只在macOS条件注册，不冒充Windows原生生命周期已齐。
