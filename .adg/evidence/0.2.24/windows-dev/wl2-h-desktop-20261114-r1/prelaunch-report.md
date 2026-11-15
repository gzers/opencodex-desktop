# H 普通桌面入口准备检查点：等待实际启动

2026-11-14 Asia/Shanghai，沿TEAM13/r3/r2。Windows代码f93651d9f42b933c660558e49c22a6d8ceff1b85，tree61547f78188fa3ee9744751ff10c9f2c8bbe448c。承接3d028e82d，不改变四trait/原预算/Unknown拒绝。**当前仅已准备，HComponentExecuted=false、productionExecuted=false、allAccepted=false。** 用户已交互登录marenben，原只读会话清单确认RDP7活跃、SID尾1003；这不替代原生Token/noJob读回。

新代码在实际产品ocxd中加入精确--internal-runtime-owner024分派，当前只支持任务自有profile/ocx024-h-UUID/{I,D,H}内的empty-owner诊断scope。单次私有继承pipe输入经过schema/长度/EOF检查；打开准确currentI的broker和测试manager镜像、SHA和原目录pin，要求原普通SID/session及bootstrap/挂起broker noJob。真实官方Owner、Job和客户端custody均不创建，所有服务请求保持Unknown/Required/noLifecycleAccepted。该宿主不是完整生产broker，也不发出恢复或后继资格。

独立observer保留最初CreateProcessW返回的broker句柄贯穿整个定向case；测试manager正常退出、仅终止准确Child的失败退出后，再fresh连接比较原brokerPID/FILETIME/SID/session/noJob及managergeneration增长，检查旧连接重放拒绝，最后explicit-recovery仅退出无官方Owner的空宿主。测试manager是实际编译库testEXE复制为taskI/opencodex-desktop.exe，**fixtureManager=true、actualGUI=false**，不能将同名文件或这一组件case说成真实GUI退出/生产四链验收。

## 编译、首错及实际输入

- build01 lib exit0/151105ms，新严格bootstrap1项通过；1882项过滤。桌面observer/manager是ignored测试，尚未执行。
- product01 allbins exit101，首错E0601：新增src/bin/windows_owner024.rs被Cargo自动识别为独立bin且无main。原stderr/result保持。小分派改为允许写集ocxd.rs内Windows-only模块，删除本轮自建未提交helper，没有改Cargo清单或新增独立程序。
- product02 allbins exit0/65200ms，MSVC debug。lib及product原始warnings保留，无strictClippy/release通过声明，未改release参数或预算。
- final-source-manifest逐一核对10项源码Git/blob/工作字节；build01库编译35份非binRust与最终一致，product02全36份Rust与最终一致。后补桌面PS/cmd作为实际启动脚本独立归档，不当编译输入。
- 最初untracked binhelper没有被当时sourceglob捕获，原hash缺失；hypotheses02先前以为其字节已归档，此错误已在artifact-provenance-limits.json明确纠正，不补造原hash。E0601原因有原始trace及最终inline差分，不冒全套首失败源闭包。

| 准备制品 | 字节 | SHA256 |
| --- | --- | --- |
| ocxd.exe | 5398016 | 5b222d2fabf3e6f8b237bbe4983144cb5e1bd49881ebad426833ab2d277e3a93 |
| h-desktop-tests.exe | 93417472 | 24f201211140560dfbe5ed558d2462805cadee80fecc517d3df7d1a1705347a7 |

这些是实际MSVC debug产品/库测试宿主，不是签名release候选。public输入逐字节复制并核SHA，根只给当前控制SID/System完全权限及marenben读/执行，out01给予marenben写入；这些ACL仅位于本次新任务Public根，不是日常目录/策略或旧原件ACL。启动脚本没有ExecutionPolicy Bypass、runas、任务/服务、令牌提升或跨会话CreateProcessAsUser；只在marenben自己桌面Shell手动启动。

## 当前完整状态

| 项目 | 实现 | 测试 | 限定 |
| --- | --- | --- | --- |
| session | read-only inventory | marenben active RDP7 observed; user confirmed login | runtime nativeToken/noJob not yet observed |
| actual-ocxd-internal-mode | Windows-only inline dispatch; actual native empty host | MSVC product02 exit0 | empty scope only; no official NativeSession/Owner/custody created; services alwaysUnknown |
| private-bootstrap | inherited pipe, one-shot framed boundedJSON+EOF, ordinary/currentTaskI/profile/noJob admission | new strict-frame pure test1passed | actual desktop invocation pending; test has syntheticconfig, no native admission pass claimed |
| original-creation-handles | observer holds actual suspended host process/thread; noJob/token/image beforeResume | compiled ignoreddesktopobserver | not yet run in marenben; no PID adoption or reconstruction |
| fixture-manager-continuity | normal/hold/after-failure managers, original brokerhandle checked eachstep; only exact fixtureChild may be killed | compiled ignoredtests only | manager is library test EXE copied as taskI/opencodex-desktop.exe, explicitly not actual GUI; noGUI/production acceptance |
| same-connection-replay | one freshchallenge per managergeneration, requests consumed, oldrequestdenied | new live desktop case pending | prior r3pipe tests reused; actual Hhost round not yet passed |
| empty-explicit-exit | explicit-recovery exits only diagnostic host with no serviceOwner; no brokerforce | compiled; pendingactualmanager | not normalOfficialStop; failure archive/recovery script prepared, no processcleanup executed |
| build-first-error | E0601 fixed by inline dispatcher in allowed ocxd.rs | product01 exit101 retained; product02 exit0/65200ms | removed untracked helper not included in initial snapshot; exacthash missing disclosed |
| library-check | strictbootstrap duplicates/oversize/truncate/extra/invalidUTF8/missingfields refusals | build01 exit0;1passed/1882filtered | desktopobserver and manager remainignored until userrun; no ordinarySD tests repeated |
| desktop-launch | exact staged artifactmanifest/hash, ordinary marenben/session7 guard, raw stdout/stderr capture | PSparser passed, input copy hashes matched | must run from marenben ownShell; not launched from current AI controller. out01 empty at preparedsnapshot |
| O-and-full-P | unchanged r3 refusal/custody gap | reused/not-run | known ordinarySD failure not repeated; no officialruntime/realbusiness/modelrequest |
| cleanup | one newPublic taskroot staged; taskonlyACL; actualaccount unchanged | no testprocess/runtime/account created yet | Public intentionallykeptforuserrun; old rejecteddeletion/Profile/schema gaps unchanged, no retry |
| W2-dualSID-release | not authorized | not-run | only sameSID Hcomponent candidate; no release |

## 下一步、现场和清理

已给用户可直接执行的C:/Users/Public/ocx-h024-desktop-20261114-r1/launch-h024-desktop.cmd。必须从marenben的Win+R/桌面Shell普通权限启动，不从当前另一个session/Job包裹的AI控制器派生，也不把Explorer名字或启动成功当noJob证明。实际nativebootstrap若不满足门禁就保持拒绝，只记录新环境事实；不会切换到Administrator或重建同样权限账户重复SD1314。

当前out01在准备快照时为空，还没有nativeToken/noJob、挂起broker、managerreplacement或cleanup实际结果。后续用户启动后用新的raw结果/独立后继证据追加回交，不将此准备检查点追溯写成已通过。

本轮账户/Profile删除0、官方进程0、实际测试broker创建0（准备快照）；只创建了新的Public输入/证据目录供用户运行，完整inventory见public-prelaunch-inventory.json。后续observer只访问任务自有I/D/H，不读写marenben实际.codex/.grok/PATH/日常配置；保留任务base，不删现有账号。若case失败先归档实际任务D，保留普通账户与Profile；脚本仅向准确空宿主发一次显式恢复，不全局杀PID、不强杀broker或借其它工具清理。旧自动审批拒绝/历史Profile/Public/schema清理缺口不重试、不关闭。

原普通完整SD缺口、ClientOriginals同卷原件保管/current引用/恢复、完整family角色、真实GUI/官方生产Start/Status/Stop/Restart仍未完；本H准备不消除它们。未W2/双SID/发布、未模型或真实业务同步。
