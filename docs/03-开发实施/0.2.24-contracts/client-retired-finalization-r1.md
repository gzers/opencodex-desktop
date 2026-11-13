# 已退休根目录的显式恢复收尾 r1

revision: 1；2026-11-13；IMP-34 P24-04；功能main848ca8c15、专用原包诊断maincbc6174de17ef658f34d361ee4a6c734bff8460c；accepted=false，非固定候选。本记录是已实施共享／Mac路径及fixture证据，原包显式恢复验证尚待结果；已通过 [正常双轮](client-generation-native-r1.md) 不因此重复或回写。

## 可用场景和确认

当前独立Head为RootRetirementPrepared，但根已经移动到该计划唯一archive、Retire意图及读回都存在，完整Ready／PreStop／Stopped与Live引用可核验，且当前logical `.codex` 缺席时，可以新预览确认“完成收尾”。根尚未移动、缺阶段、缺readback、writer marker仍在、当前Grok或来源变化都不进入此动作，也不自动移动根、补receipt、修权限或执行回退。

复核按独立索引的source／origin／Plan推导Retire intent/readback精确字节和摘要，不从磁盘任意新摘要当作基线。要求exact released archive tree、当前Grok及Original恢复关系、每帧与素材／scope／Live／provenance及两stable-lock身份，历史上若已有RootAbsent，其completion digest必须相同。确认以独立domain绑定完整pending Head、history摘要、source、origin、Plan／readback、archive身份／tree／Grok、scope/Live及HOME；它是确定性确认指纹，没有时间TTL，也不构成执行权限。

root预览只提供canFinalize、3phase、completionEvidence=unregistered与新digest，其他动作关闭；前端沿既有单层确认入口精确接受此root phase，其他证据组合拒绝。公共请求、结果字段和status=restored未增加。动作只创建一次新的stop10恢复deadline，并以上层原deadline为上限，所有重核消费剩余时间；绝不延长旧normalStop或把旧失败晋升成功。

## 独立恢复证明与提交

RecoveryCompletion字段私有、无Clone/serde／公开构造器；仅由这次held Index／request复核、完整当前程序准入、原Live记录绑定的当前服务／launcher／port／HOME进程和manager缺席、两stable-lock真实exclusive获取与正常释放核验后构造。它不能转换为AuthorizedReadback、MutationPermit或原正常退出证明。源文件不回放、不修改客户端或原包，动作只关闭同计划metadata。

source/candidate/postcommit原完整事务审计保留，并在首次SQL前、SQL后／COMMIT后核当前程序选择／进程／锁／deadline；写入或后续拒绝不伪称事务回滚。若实际next已提交且仍可完整认证，追加同计划pending再返回拒绝；无法安全追加时保留现场并明确失败，不返回成功。

初始actualdiff审查指出P1：完整image扫描之后只检查时间，可能漏掉扫描期间当前根／Grok／进程变化。已修为扫描后执行check_closed，重新exact Head完整审计、history/lineage／full retired review与fingerprint、程序选择、当前OS缺席与双锁释放、原deadline；首次和重复路径都使用。补审解除P1，未代执行测试。

精确重复原确认只可对应current RootAbsent紧邻的pending predecessor；再次完整核当前资料／programbytes／OS／locks后，返回原已关闭revision（仍等于原request.revision+1），不增加revision。历史中18后当前pending19不能当alreadydone，原确认19在收尾后20才可关联。不同／陈旧／换domain确认拒绝，前端不自动重试mutation；普通既有目录finalize沿原路径，预算从动作开始计入预览读取。

## 已完成验证与仍待原包

五项新fixture测试通过：完整现存证据稳定确认、合成未准入runtime不能仅凭review执行、根未移／readback缺失不提供入口、计划／intent／receipt／各phase／scope／Live／archive／Grok／root／namespace变化拒绝；原close到期留下9→10 pending的完整旧历史保持而新确认改变；实际双SQLite竞争拒绝并可正常释放后重获。fixture不签生产Stop权限，也不证明新的原包恢复预算已过。

完整原件320＋协调29＋owner20＋命令4共373项不同后端测试通过，原有忽略保留；前端恢复flow/API94项通过。严格all-targets Clippy -D warnings、fmt、vue-tsc、diff-check通过。最初encode(slice)编译错误保留，仅改传slice引用，编码格式和解析规则不变。完整回归与最终test-only图像漂移／快照语句的构建范围在source receipt明确，release原包入口将核固定最终源码，不冒称较早debugbinary包含后来测试语句。

专用cfgtest＋ownedProfile＋显式flag验证入口在真实原服务Stop／原件恢复／根move/readback后刻意中断close，保留原Stop失败；接着从真实preview经public finalize，要求新close<10s、重复request<10s无新增revision。actual审查P2发现初始入口提前返回缺最终源字节快照，已加after-retired-finalize／after-retired-duplicate各7个tracked路径存在性、SHA/base64及原Option字节逐项比较／alltrue断言；之后另注入完整image扫描后的合成Grok变化，必须拒绝并保留该变化。补审确认P2解决，原包专用轮尚未执行，不能预填通过。

证据 [.adg/evidence/0.2.24/client-retired-finalization-r1](../../../.adg/evidence/0.2.24/client-retired-finalization-r1/manifest.json) 包含开跑前假设、全部命令／压缩原日志、P1/P2与修订、11源及debugbinary范围、独立verifier和清理。Node/Vite留下119个task-owned缓存文件约1.12MB，已原字节压缩核对、摘要留存后清理，不混充业务证据；unitTMP随后为空，全部材料逐字归档后删除证据临时根，工程target保持。

已固定cbc6174de和32关键源开始release库宿主构建，后继只验证这项显式恢复／重复确认／扫描后拒绝，不重跑已通过正常双轮，也不发布。根未移继续／回退、缺Ready／正常阶段不足、创建根取消仍需独立处理；一般客户端图路径／完整进程集合与刷新／NAME／复杂代理／WindowsW1W2／官方交付验证及固定app/DMG/MSI/NSIS等原完整范围保持，目标active。无日常配置／Shell／已安装app／官方2.99或真实WebDAV／模型请求变更。
