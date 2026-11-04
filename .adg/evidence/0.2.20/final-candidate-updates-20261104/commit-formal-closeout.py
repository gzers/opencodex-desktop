from pathlib import Path
import sys,json,yaml,hashlib,subprocess,datetime,re
sys.path.insert(0,"/Users/ezio/.codex/skills/project-governance/scripts/lib")
from runtime import kernel as k
root=Path("/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main")
ev=root/".adg/evidence/0.2.20"
final=ev/"final-candidate-updates-20261104"
frozen="b1750f5cf78b294b2edb34b9674d8c707ef8f810"
task_id="TASK-OPENCODEX-UI020-CLOSEOUT-20261104"
task_digest="05900c538996b6111920e6f6659e122788896fcf035040f1d6133bcb81122b7d"
receipt_id="RCP-OPENCODEX-UI020-CLOSEOUT-20261104"
cert_id="CERT-OPENCODEX-UI020-CLOSEOUT-20261104"
store=k.RuntimeStore(root)
task=store.load("TASK",task_id)
assert task["state"]=="in_progress" and task["content_digest"]==task_digest
review=json.loads((ev/"final-entry-reconciliation-20261104/final-independent-review.json").read_text())
assert review["result"]=="pass" and review["sourceCommit"]==frozen
stable=json.loads((final/"public-stable-verification.json").read_text())
assert stable["result"]=="pass" and stable["tag"]=="v0.2.20" and stable["sourceCommit"]==frozen
assert stable["releaseId"]==433103683 and stable["latest"]=="v0.2.20"
assert stable.get("mergeCommit")=="28d7f62e994beed57d291c904ab2c775491bc928"
gates=json.loads((final/"final-promotion-gates.json").read_text())
assert gates["result"]=="pass" and gates["sourceCommit"]==frozen
assert gates["productCi"]["conclusion"]=="success"
assert all(item["status"]=="passed" for item in gates["requiredChecks"])
assert k.digest_bytes((ev/"font-final-candidate-bda7c92d/source-freeze.json").read_bytes())==task["baseline_digest"]
for p in [final/"macos-adjacent-ota/ota-result.json",final/"windows-adjacent-ota/summary.json",ev/"font-final-candidate-bda7c92d/native-connection-consumers/result.json",final/"asset-verification.json"]:
 assert json.loads(p.read_text())["result"]=="pass",str(p)
acceptance=json.loads(Path("/tmp/ocx020-acceptance-map-seed.json").read_text())
assert len(acceptance["criteria"])==20
acceptance["status"]="passed"
acceptance["finalIndependentReview"]="final-entry-reconciliation-20261104/final-independent-review.json"
acceptance["publicStable"]="final-candidate-updates-20261104/public-stable-verification.json"
acceptance["sourceCommit"]=frozen
acceptance["nativeEvidenceReuse"]="final-candidate-updates-20261104/unchanged-consumer-source-reuse.json and final-candidate-34a3553f/native-model-history-consumer/native-launch.json explicitly bind unchanged consumer modules to prior bda evidence; new payload gates are separate."
for row in acceptance["criteria"]:
 row["status"]="passed"
 if row["id"] in ["AC-020-19","AC-020-20"]:
  row["evidence"].append("final-entry-reconciliation-20261104/final-independent-review.json")
 for p in row["evidence"]: assert (ev/p).exists(),p
acceptance["scopeLimits"]=["Adopted cfbd90c4b + MCP1da4cabca + user typography only; future mock modules excluded by exact source.","Browser state consumers use explicit fixtures; real file/native/credential/install/OTA evidence is separately identified.","Mac OTA requires exact new payload automatic restart with sandbox, original bundle identity and process-local restoration handling explicitly recorded.","Legacy WebDAV connection passwords retain existing local backend; content-protection passphrase uses separate native keyring.","No daily real WebDAV/multi-device synchronization or future client MCP key-path adaptation is claimed."]
(final/"final-acceptance-map.json").write_text(json.dumps(acceptance,ensure_ascii=False,indent=2)+"\n")
bugs="docs/04-项目资料/01-项目支撑资料/06-问题记录/0.2.20-bugs.md"
dmd="docs/01-需求管理/需求/DMD-OPENCODEX-UI-020.md"
imp="docs/03-开发实施/IMP-OPENCODEX-DESKTOP-28.md"
rel="docs/03-开发实施/REL-OPENCODEX-DESKTOP-23.md"
def read_source(path):
 _,front,body=(root/path).read_text().split("---",2)
 return yaml.safe_load(front),body
def write_source(path,metadata,body):
 (root/path).write_text("---\n"+"\n".join(key+": "+json.dumps(value,ensure_ascii=False) for key,value in metadata.items())+"\n---"+body)
now=datetime.datetime.now(datetime.timezone.utc).isoformat()
resolution="S20-01～20按采用原型及字体增量验收完成；同一b175签名原包完成两平台安装和相邻OTA并公开v0.2.20 stable，限制与原始失败保留于REL23和原问题清单。"
completion="全部20项AC与六项审阅门有分层证据；冻结b175源码、Windows四安装、Mac/MSI/NSIS实际相邻更新及公开stable完成，原型/消费者遗漏和证据错挂已纠正。"
dm,db=read_source(dmd);im,ib=read_source(imp)
assert dm["state"]==im["state"]=="in_progress"
dm["summary"]="0.2.20采用原型与字体增量范围；20项验收、双平台原包/实际升级及稳定发布均有最终证据。"
dm["next_action"]="依据已完成验收形成最终关闭回执与证书。"
im["summary"]="0.2.20冻结源码与原包已完成全部适用验收并公开stable；保留原始失败、模拟/原生边界及未来包排除。"
db=db.replace("全部已采用入口与35处组件调用必须有检查状态及证据", "全部已采用入口、37处组件引用、26处动态调用及另列自定义宿主均有检查状态及证据")
db+="\n\n2026-11-04最终验收："+resolution+" 正式关闭为"+task_id+"、"+receipt_id+"与"+cert_id+"，逐项证据在.adg/evidence/0.2.20/final-candidate-updates-20261104/final-acceptance-map.json；不以通过数量代替入口覆盖。\n"
ib=ib.replace("正在实施，尚未完成正式验收/发布；具体改动和剩余工作回写0.2.20-bugs.md第9节，本文件不另持有第二份问题状态。", "已完成本版适用范围验收并公开v0.2.20 stable；具体逐项结论和历史失败保留在0.2.20-bugs.md第9节，本文件不另持有第二份问题状态。")
ib+="\n\n2026-11-04最终收口："+completion+" PR67 merge 28d7f62e994beed57d291c904ab2c775491bc928与tag b175源码tree相同。详情见REL-OPENCODEX-DESKTOP-23及原问题清单；正式关闭由RuntimeStore的TASK/RCP/CERT承接。\n"
write_source(dmd,dm,db);write_source(imp,im,ib)
# Update current source summaries before lifecycle closure; frozen TASK AC remains unchanged.
for kind,oid in [("DMD",dm["id"]),("IMP",im["id"])]:
 old=store.load(kind,oid);updated=k.sync_source_bound_object(old,root,expected_digest=old["content_digest"]);store.put(updated,expected_digest=old["content_digest"])
dm["state"]="completed";dm["next_action"]=None;dm["resolution"]=resolution
im["state"]="completed";im["completion_summary"]=completion
write_source(dmd,dm,db);write_source(imp,im,ib)
rm,rb=read_source(rel)
rm["state"]="verified"
rm["summary"]="v0.2.20已公开stable/Latest；八资产、三更新签名、独立四安装和Mac/MSI/NSIS相邻实际更新及最终字体/入口审阅完成。"
rb="""

# 0.2.20 原型一致性与默认字体修复发布

用户已授权治理、开发、测试和stable连续推进。范围为DMD-OPENCODEX-UI-020、IMP-OPENCODEX-DESKTOP-28及原0.2.20-bugs的S20-01～20。采用cfbd90c4b、MCP增量1da4cabca及用户确认的默认字体调整：正文/导航13px、控件12px、副标题15px、弹窗18px、页标题22px，小字与所选缩放语义保留。首次成功云端目录全选，之后新增不自动勾选。

[v0.2.20已公开stable/Latest](https://github.com/gzers/opencodex-desktop/releases/tag/v0.2.20)。冻结产品源码b1750f5cf78b294b2edb34b9674d8c707ef8f810，PR67合并28d7f62e994beed57d291c904ab2c775491bc928；Release ID433103683、八资产ID/字节/摘要均与已验新候选相同。原PR65/66由PR67整合覆盖并关闭；最终tag/source b175包含所需产品与QA提交。旧bda候选因UpdateCenter共享缩放失败撤回为私有validation/0.2.20-bda7c92d归档，原八资产/失败/原安装OTA证据保留；最终不采用其包体。

完整产品CI41699143981的最终结果与此前普通账户首次启动失败原件分开保留。该失败窗口有响应但沙箱日志未生成、托盘Access denied，未推定未经证明的根因；复验保持原阈值。最终直接源码/本机与双引擎检查、所有必需CI事实见final-ci-completed.json及final-promotion-gates.json。官方合同41671597006 attempt2两项通过，其Rust/协议实现与最终b175完全相同，明确复用未变模块，不把诊断日志当产品变化。

构建41699584553完成双平台原包/签名/上传，Mac job成功，Windows后置原生菜单点击检查超时的原始失败留存。相同原包的独立四安装验收通过后承接安装门，未重建已上传字节；不将原Release job写为绿色。八上载资产、三份tag公钥签名/sidecar、五平台manifest、匿名下载全过；Mac原封包deep/strict、嵌入源码与LICENSE/NOTICES、DMG完整性通过。Mac原始EXE SHA 544de246de1a498b8c681c0bd6ce254c7043ffe4b5fe754d67d3477a8fe4370b；Windows原始EXE SHA 7f6788eb996e9ff5ed3a2283650567b35ccabcf24bb747fd40cc1a2c3afaed77。Windows类型标记只允许唯一MSI/NSS变化，仍比较完整预期字节和原始构建身份。

独立41738559767的MSI/NSIS×默认/自定义四格真实安装/首屏/卸载/数据保持全过。Windows41739113409实际0.2.19→0.2.20两格完成签名下载、普通退出保旧、明确安装与自动新进程、原配置/合成凭据读回和pending清理；安装后真实WebView2更新中心100/150/200%按钮高34/51/68px，四边与关闭可达。自动OS新进程观察后，Windows原生安装器重启经Shell不继承测试变量，故使用相同已安装EXE显式隔离重开作文件/窗口读回；边界如实记录。此前探针误用尚未检查快捷入口、无运行环境的概览维护卡、普通按钮role选择器及一轮初始CDP超时均保留，最终修正后严格断言通过，不将失败改写为通过。

Mac实际相邻OTA使用原封0.2.19身份/签名和规范/private/tmp路径，进程级ApplePersistenceIgnoreState YES只处理本机已有AppKit旧恢复提示，不改系统设置。两次真实旧版UI保存及精确前像后，经既有beta端点下载签名安装，确认重启后同路径新进程自动挂载b175/0.2.20并继承隔离环境；直接在同一新进程读回原渠道，9份业务/偏好/新备份前像摘要保持、handoff消费、比例100%保留。最终轮没有二次readback副本，区别旧bda历史轮次。最终原封升级包在同一自动重启进程另实屏验证概览/设置两宿主200%放大、最新版当前唯一版本与原位Escape；整包逐文件等同上载签名载荷，日常安装未替换。

逐入口最终口径为37处Dialog/Modal组件引用、26处动态调用、3处另列UpdateCenter/既有AgentPanel宿主引用，不是66个唯一窗口。独立复核补齐模型助手、更新中心和最终字体在模型/扩展/次要窗口的布局；发现共享缩放漏项后先修复再重打新包和全部包体验收。历史恢复草稿已通过真实原生窗口/IPC将64000→128000放入草稿并返回模型页，已保存策略与业务config保持；记录目录为显式seed的已校验legacy消费者夹具，不冒称真实官方运行时保存。原型窄屏探针纠正的是browser viewport未改变固定产品window的测量方法，不改原型源或借未来模块补功能。

正式证据位于.adg/evidence/0.2.20/final-candidate-updates-20261104和final-entry-reconciliation-20261104。原生连接/导出/备份/范围恢复等此前bda层证据只有在测试模块逐字节相同条件下复用，包体安装、签名和OTA必须引用最终b175原包。20项AC及六项review由RuntimeStore的TASK/RCP/CERT关闭；文件范围、来源、绑定与人类事实回写均校验。

兼容与回退：交付macOS Apple Silicon和Windows x64；保留公开v0.2.19原包/签名与本机保护备份作为手动回退来源，不承诺自动降级、不移动稳定tag。沿用macOS ad-hoc与Windows无Authenticode方式，更新载荷另有真实密码学签名。WebDAV连接密码沿用既有本机文件后端（Mac0600/0700、Windows DPAPI），内容保护口令独立使用原生安全存储，不能混称全部系统钥匙串。底层文件权限错误可能显示系统详细英文原因。真实客户端MCP键路径、多Agent扩展/未来对话与连接图不在本版采用范围。没有读取/发布日常秘密、改变日常配置/安装或执行日常真实WebDAV/双设备同步。
"""
rb+="\n公开stable于"+now+"核验，原tag与Latest匿名manifest及八资产身份保持。正式关闭"+task_id+" / "+receipt_id+" / "+cert_id+"。\n"
write_source(rel,rm,rb)
p=root/bugs;s=p.read_text();s=s.replace("**目标版本：0.2.20；当前阶段：治理与开发中，完整验收/发布尚未完成。**", "**目标版本：0.2.20；当前阶段：已完成采用范围验收并公开 stable/Latest。**",1)
s=s.replace("## 5. 全应用覆盖表（全部登记；非全部验收通过）", "## 5. 全应用覆盖表（0.2.19审计底稿；0.2.20最终结论见第9节）")
s=s.replace("## 6. 弹窗逐项入口台账", "## 6. 弹窗逐项入口台账（0.2.19审计底稿；最终映射见第9节）")
s=s.replace("## 9. 0.2.20开发实施与剩余验收（2026-11-02）", "## 9. 0.2.20开发实施与最终验收（更新至2026-11-04）")
s=s.replace("以下“已实现”不等于关闭；全部required_checks仍待完整回执。", "下表为最终适用范围结论；后续按日期保留各阶段原始失败与纠正，不把历史待验语句当当前状态。")
s=s.replace("授权包括开发、测试、提交/合并与stable发布；尚未发布0.2.20。", "授权包括开发、测试、提交/合并与stable发布；现已公开v0.2.20 stable。",1)
start=s.index("| S20 | 当前实施与已取得证据 | 剩余验收/工作 |")
end=s.index("\n证据在[implementation-20261102]",start)
lines=["| S20 | 最终结论与证据 | 范围与边界 |","| --- | --- | --- |"]
human_basis=[
 "前后端旧人工复核门槛均移除，路径、文件限额及凭据身份校验保留；正式包混合资源可直接预览。",
 "预设与清空入口移除；当前搜索表头选择、隐藏勾选、半选、空与禁用状态均已覆盖。",
 "旧all值物化具体对象，依赖与顺序按可见范围承载；桌面偏好只含四项显示字段，不隐含扩展整份配置。",
 "文件、WebDAV及导入选择独立记忆；新设备首次成功目录全选，其后新增不自动选，失败不初始化。",
 "凭据绑定对象身份和MCP来源，同名不同类型可分辨；未选秘密不导出、不覆盖接收端已有值。",
 "默认不加密，手动双口令保护冻结内容且无第二确认；实际自定义目录、同日多包及打开目录通过。",
 "内容保护开关与连接密码独立，保护口令进入原生安全存储，普通元数据只有引用；轮换与旧密文读取覆盖。",
 "文件选择、解密、范围、变化与备份共用窗口；普通接收直接落盘，旧计划及明确draft保持独立兼容合同。",
 "正式包真实本机HTTPS认证失败/成功、保存重开、留空密码及权限拒绝均验证；候选目录显式seed，启用消费另测。",
 "概览与同步共享宿主；打开、取消、Escape、完成与结果回看原页保持，最终原生导出和连接均实测。",
 "持久结果对应原操作，部分失败沿原ID和原备份重试；通知与清理状态、两次导出各自回看及实际目录通过。",
 "未知变化阻止确认，密钥脱敏、Skill具体文件与双侧来源明确；MCP不选来源准确跳过，选Codex包内内容一致。",
 "普通导入真实写入配置并可重开，无第二次保存；未保存编辑重基准、并发拒绝、显式draft分别覆盖。",
 "实际所选资源备份与权限/摘要、同操作原始前像保持；真实分组恢复拒绝过期预览并保留后来独立字段。",
 "共同内容基线、提交序列及双隔离文件根/HTTPS条件请求证明收敛，传输不抬高内容时间；未冒称日常双设备同步。",
 "四项显示偏好白名单、账户身份、当前连接保留；导入连接需单独启用，真实客户端MCP键路径适配仍归未来包。",
 "取消清理未尝试冻结计划且不生成交付文件；已开始计划保留原ID，部分完成、备份失败与重试边界有真实文件证据。",
 "旧失败保留；冻结源码完整CI与显式官方/HTTPS/原生检查通过，平台子进程与外部夹具逐项说明，不把取消/跳过算通过。",
 "正文13、控件12、副标题15、弹窗18、页标题22与原型同源；完整依赖同步矩阵及最终字体非同步窗口补证通过。",
 "37组件引用、26动态调用与另列自定义宿主逐项映射；原生承载单列，未来mock排除有来源，错挂证据已纠正。"
]
for index,row in enumerate(acceptance["criteria"]):
 text=human_basis[index]
 lines.append("| "+row["s20"]+" | 通过；[AC对应证据](../../../../.adg/evidence/0.2.20/final-candidate-updates-20261104/final-acceptance-map.json) | "+text+" |")
s=s[:start]+"\n".join(lines)+"\n"+s[end:]
s=s.replace("当前已有分层浏览器、真实文件、原生IPC/安全库及Mac实屏证据；最终候选完整覆盖仍未通过，不写“全量原型一致/可发布”。所有正式验收项目保持未完成。", "最终分层浏览器、真实文件、原生IPC/内容保护安全库、Mac实屏、Windows原生/四安装/实际OTA及公开stable证据已通过；具体限制仍逐项保留。正式20AC与六项审阅门由RuntimeStore关闭，不把原型打样页或未来模块纳入已实现范围。")
s+="\n\n### 2026-11-04 最终逐项验收与稳定发布\n\n"+resolution+" 同一release433103683/八资产由prerelease晋升stable，无改tag、换公钥或重打包。公开tag与Latest匿名manifest与八原始资产身份保持；支持macOS Apple Silicon与Windows x64。\n\n入口最终清单由37处Dialog/Modal引用、26处动态调用与另列UpdateCenter/既有AgentPanel自定义宿主构成，不把引用数当唯一窗口数。独立复核纠正遗漏宿主、历史恢复草稿错挂和最终字体泛用证据，补证及保留失败见final-independent-review.json。当前S20与SY逐项对应在final-acceptance-map.json，冻结AC文字未修改。原型浏览器viewport与产品window尺寸的探针差异及实际产品可达性分列，不暗改采用原型。\n\n全部旧失败、跳过与环境纠正原件保留。原生Mac同路径自动挂载与同一进程读回的边界、旧连接密码文件后端与内容保护原生安全库之别、真实客户端键路径未来依赖均按REL23记载。用户日常业务配置/秘密与日常安装未改，不承诺本轮未执行的日常真实多设备WebDAV。正式回执"+receipt_id+"与证书"+cert_id+"承接本次关闭。\n"
p.write_text(s)
# SY rows reference the exact frozen S20 acceptance mapping, rather than a count.
sy=[]
for line in s.splitlines():
 if line.startswith("| SY-"):
  cells=[x.strip() for x in line.split("|")[1:-1]]
  ids=re.findall(r"\d{2}",cells[-1]);sy.append({"id":cells[0],"entry":cells[1],"acceptanceIds":["AC-020-"+x for x in ids],"status":"passed","evidence":"final-acceptance-map.json:criteria"})
assert len(sy)==26
acceptance["syncEntries"]=sy;(final/"final-acceptance-map.json").write_text(json.dumps(acceptance,ensure_ascii=False,indent=2)+"\n")
ledger=ev/"implementation-20261102/dialog-callsite-ledger.md"
ledger.write_text(ledger.read_text()+"\n\n## 2026-11-04 最终状态\n\n采用范围入口与字体最后补证通过；详见final-entry-reconciliation-20261104/final-independent-review.json及final-candidate-updates-20261104/final-acceptance-map.json。0.2.20已公开stable/Latest，同一b175原包完成安装与实际相邻OTA。历史段落仅为当时记录，当前以正式TASK/RCP/CERT为准。\n")
progress=ev/"implementation-20261102/RCP-progress.json";j=json.loads(progress.read_text());j["prior_required_checks_before_closeout"]=j["required_checks"]
j["historical_source_before_final_closeout"]=j["source"]
j["source"]="Final frozen b1750f5cf78b294b2edb34b9674d8c707ef8f810 / PR67 merged28d7f62e; fullCI41699143981success; exact8assets433103683 and3signatures pass; MacactualOTA +Windows41739113409 +fourinstallers41738559767pass; publicstableLatest0.2.20verified. Official2/2 bda contract reused only foridenticalRustsource, earlierfailure retained."
j["release_candidate"]["firstReleaseInstallerCheck"]="Original Windows Release postuploadinnerselecttimeout retained asfailure; sameuploadedbytes independent4cells41738559767passed andfinalCIordinarynativepassed. Rootcauseunproven, no thresholdorassetchange."
j["release_candidate"]["actual_windows_ota"]={"run":41739113409,"result":"pass","evidence":"final-candidate-updates-20261104/windows-adjacent-ota/summary.json"}
j["release_candidate"]["actual_macos_ota"]={"result":"pass","evidence":"final-candidate-updates-20261104/macos-adjacent-ota/ota-result.json","sameAutomaticProcessReadback":True}
j["release_candidate"]["independent_installer_cells"]="MSI/NSIS x default/custom4/4pass, exactsourceandbuiltSHAverified"
check_evidence={name:".adg/evidence/0.2.20/final-candidate-updates-20261104/final-acceptance-map.json; .adg/evidence/0.2.20/final-entry-reconciliation-20261104/final-independent-review.json" for name in task["required_checks"]}
check_evidence["review:release-and-ota"]=".adg/evidence/0.2.20/final-candidate-updates-20261104/public-stable-verification.json; macos-adjacent-ota/ota-result.json; windows-adjacent-ota/summary.json"
j["required_checks"]={name.removeprefix("review:"):{"status":"passed","evidence":path} for name,path in check_evidence.items()}
j["status"]="succeeded";j["published"]=True;j["release_hold"]=None;j["release_candidate"].update(draft=False,prerelease=False,status="published stable, same frozen assets",stable="v0.2.20")
j["formal_receipt"]=receipt_id;j["formal_certificate"]=cert_id;j["tests"]["official_contract_final"]="Exact bda run41671597006 attempt2,2/2 passed; earlier failed rounds remain historical."
progress.write_text(json.dumps(j,ensure_ascii=False,indent=2)+"\n")
# The RCP binds authoritative human facts after writeback; machine projections follow closure.
impacts=[{"target":target["id"],"path":target["path"],"after_digest":k.digest_bytes((root/target["path"]).read_bytes()),"summary":"Final adopted scope acceptance and stable release facts written back."} for target in task["fact_writeback"]["targets"]]
baseline_commit="dc8732eb31f0c15e4cce1ba1f291a8855904bd78"
tracked_at_baseline=set(subprocess.check_output(["git","ls-tree","-r","-z","--name-only",baseline_commit,"--",".adg/evidence/0.2.20"],cwd=root,text=True).strip(chr(0)).split(chr(0)))
actual_diff=set(subprocess.check_output(["git","diff","-z","--name-only",baseline_commit,"--",".adg/evidence/0.2.20",dmd,imp,bugs,rel],cwd=root,text=True).strip(chr(0)).split(chr(0)))
new_evidence={str(p.relative_to(root)) for p in ev.rglob("*") if p.is_file() and str(p.relative_to(root)) not in tracked_at_baseline}
changed=sorted(actual_diff|new_evidence|{x["path"] for x in impacts}|{str((final/"formal-closeout-result.json").relative_to(root))})
receipt=k.build_receipt(task,receipt_id=receipt_id,result="succeeded",changed_paths=changed,checks=[{"name":name,"status":"passed","evidence":path} for name,path in check_evidence.items()],acceptance_results=[{"acceptance_id":row["id"],"status":"passed","evidence":".adg/evidence/0.2.20/final-candidate-updates-20261104/final-acceptance-map.json:"+row["id"]} for row in acceptance["criteria"]],fact_impacts=impacts,completed_at=now,summary=completion)
closed,stored_receipt,cert=store.commit_task_closure(task,receipt,expected_task_digest=task_digest,certificate_id=cert_id,verified_at=now)
k.verify_closed_task(closed,stored_receipt,cert)
for kind,oid in [("IMP",im["id"]),("DMD",dm["id"])]:
 old=store.load(kind,oid);updated=k.sync_source_bound_object(old,root,expected_digest=old["content_digest"]);store.put(updated,expected_digest=old["content_digest"])
result={"result":"pass","task":task_id,"taskState":closed["state"],"receipt":receipt_id,"receiptDigest":stored_receipt["content_digest"],"certificate":cert_id,"certificateDigest":cert["content_digest"],"demandState":store.load("DMD",dm["id"])["state"],"implementationState":store.load("IMP",im["id"])["state"],"acceptanceCount":20,"requiredChecks":len(task["required_checks"]),"publicStable":"v0.2.20","baselineDigestUnchanged":task["baseline_digest"]}
(final/"formal-closeout-result.json").write_text(json.dumps(result,ensure_ascii=False,indent=2)+"\n")
(final/"SHA256SUMS").write_text("".join(hashlib.sha256(p.read_bytes()).hexdigest()+"  "+p.relative_to(final).as_posix()+"\n" for p in sorted(final.rglob("*")) if p.is_file() and p.name!="SHA256SUMS"))
print(json.dumps(result,ensure_ascii=False))
