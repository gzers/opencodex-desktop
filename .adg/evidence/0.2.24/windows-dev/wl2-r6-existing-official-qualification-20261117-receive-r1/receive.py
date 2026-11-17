from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent;DOCS=ROOT.parents[4];PUBLIC=Path('C:/Users/Public/ocx-r6-official-existing-20261117-r1/out01')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
for name in ['launch-result.json','broker.stdout.log','broker.stderr.log','same-sid-export.json']:
    raw=(PUBLIC/name).read_bytes();p=ROOT/'raw-public'/name;p.parent.mkdir(exist_ok=True)
    with p.open('xb') as out:out.write(raw)
    rows.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
manifest=load(PUBLIC/'same-sid-export.json');assert manifest['complete'] and not manifest['fileErrors'] and manifest['sid'].endswith('-1003') and manifest['session']==7
expected=[]
for row in manifest['files']:
    rel=Path(row['relative']);assert not rel.is_absolute() and '..' not in rel.parts and rel.parts[0] in ['D','qualification-preparation']
    raw=(PUBLIC/'raw-export'/rel).read_bytes();assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    p=ROOT/'raw-export'/rel;p.parent.mkdir(parents=True,exist_ok=True)
    with p.open('xb') as out:out.write(raw)
    expected.append(rel.as_posix());rows.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw),sameSidSource=row['source']))
assert sorted(expected)==sorted(p.relative_to(PUBLIC/'raw-export').as_posix() for p in (PUBLIC/'raw-export').rglob('*') if p.is_file())
launch=load(ROOT/'raw-public/launch-result.json');result=load(ROOT/'raw-export/D/qualification-result.json');action=load(ROOT/'raw-export/D/qualification-start1.json')
assert launch['dailyPathUnchanged'] and launch['brokerRawExit'] is None and launch['brokerRetained']
assert launch['brokerPid']==54692 and launch['brokerCreation']=='2026-11-17T10:55:41.3203335+08:00'
assert not result['actualProductionExecuted'] and not result['nativeProductionSequencePassed'] and result['nativeOwnerRetained'] and not result['startOpen']
assert 'windows024_originals_required' in result['firstError'] and '(os error 2)' in result['firstError']
assert action['elapsedMs']==9842 and action['outerBudgetMs']==30000 and action['stopBudgetMs']==10000 and action['result'] is None
t=result['trace'];assert t['stage']=='created-suspended' and not t['creatorResumed'] and not t['originalPhaseRetained'] and t['family']==dict(active=1,members=[],root='Running',total=1)
assert t['binding']['pid']==50580 and t['binding']['filetime']=='134393577919362322'
image=load(ROOT/'raw-export/qualification-preparation/image-created-verified.json');assert image['sharedFullImageReadback'] and image['members']==8660 and image['unpackedBytes']==611461049
prepared=load(ROOT/'raw-export/qualification-preparation/prepared.json');assert prepared['prepared'] and prepared['credentialFreeProviders'] and prepared['autoRefreshDisabled'] and not prepared['nativeLeaseGranted']
status=dict(contract='TEAM16/IMP142/r6',sourceHead='c870bd81fcbd65310ff79d262c9c49fc5a2c6f62',sourceTree='d4297896548b098b46007c10686fba39b72c17a6',time='2026-11-17T10:56:36+08:00',ordinarySid=manifest['sid'],ordinarySession=7,noOuterJob=True,dailyPathUnchanged=True,rawFiles=rows,rawExportComplete=True,preparationPassed=True,sourceImage=image,firstError=result['firstError'],originalErrorUTF8Preserved=True,launcherErrorTextMojibake=True,firstAction=action,nativeLauncherRetained=t['binding'],broker=dict(pid=54692,filetime='134393577413203335',exited=False,rawExit=None),actualProductionExecuted=False,actualCustodyMutationExecuted=False,originalPhaseCreated=False,clientUnchangedMustStillBeFreshlyVerified=True,typedClosedQualified=False,successorQualified=False,startOpen=False,allAccepted=False,cleanup=dict(brokerAndSuspendedLauncherRetained=True,ordinaryBaselineAndSourceAndLogsRetained=True,fullPrivateTaskArchiveStillNeeded=True,historicalUnknownBroker40588Untouched=True,historicalRejectedDeletionRetried=False,overallCleanupComplete=False),sourceFinding='StartContentScope uses RuntimeVersionPaths.package_root (npm prefix) for src/lib/redact.ts etc, missing install::PACKAGE_SUBPATH. Same defect found in diagnostic CLI/sentinel/callsite path consumers. Fix all Windows source-script references; keep prefix/node_modules/bun/bin/bun.exe unchanged.',hostFinding='Executed debug qualification host intentionally parks after result and exposes no control channel. Original retained Job cannot be reached by a rebuilt external process. Do not adopt owner/normalExit from PID/JSON or retry same failed session. Need full same-SID archive and a concrete cleanup/next-scene decision before another native lifecycle run.',W2=False,dualSID=False,published=False)
with (ROOT/'result.json').open('x',encoding='utf-8') as out:json.dump(status,out,ensure_ascii=False,indent=2)
report=f'''# r6原包轮首次失败实际结果

源码c870bd81f，在marenben SID1003/session7/noJob下实际运行。固定bundle8660文件611461049字节完整解包与sharedFullImageReadback通过；三N/L/K原缺席setup和无凭据H配置准备完成。首Start动作9,842ms/原30,000ms内返回windows024_originals_required，原始UTF8首错：{result['firstError']}。

原Node50580/FILETIME134393577919362322已create-suspended但creatorResumed=false；原Job1active/1total/无member，尚无OriginalSession/custody变更/官方服务执行或Ready。broker54692/FILETIME134393577413203335仍原进程保留，rawExit=null不补0；后续Status/Stop/后继/Restart均未执行。User/Machine PATH摘要不变。

{len(rows)}份原日志/同SID原字节导出全部成员、长度及SHA256核；启动器5.1默认编码造成nativeFirstError展示乱码，原native UTF8 JSON未改且正确，修复需显式UTF8读取。源码定位：StartContentScope把npm prefix当包目录，四审计源缺node_modules/@bitkyc08/opencodex；D角色CLI/sentinel/callsite也有同类引用，需批量修正，Bun路径本属prefix保持。不是包缺失/权限/预算错误，不改官方源。

完整私有task/native现场归档仍待原SID；已运行宿主park后无控制通道，重编程序不能获得旧Job/Owner。保留失败及原挂起launcher，不能新建原生资格或以旧broker40588/外部PID收养/强杀补Stop。下一实机轮须先明确该现场的受控清理与新独立测试基线，不能直接重跑失败会话。公开Start/allAccepted/实际生产/typedClosed=false；其余初始化/原缺席安全退根/普通MSI/CAP-trust/历史清理未完保持，无W2/双SID/发布。
'''
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(report)
p=DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md';heading='## 2026-11-17 r6原包轮首错与未恢复launcher保留'
assert heading not in p.read_text(encoding='utf-8-sig')
with p.open('a',encoding='utf-8') as out:out.write(f'\n\n{heading}\n\n实际c870bd81f在marenben SID1003/session7/noJob运行，固定包8660文件611461049字节/shared完整image及无凭据配置/NLK setup通过；首Start9842ms/原30s返回windows024_originals_required、oserror2。证据.adg/evidence/0.2.24/windows-dev/{ROOT.name}/；{len(rows)}份原字节/同SIDexport完整核。原Node50580/FILETIME134393577919362322仍create-suspended/creatorResumedfalse/原Job1active1total，无OriginalSession/custody变更或官方服务/Ready，Stop/后继/Restart/Status未执行。原broker54692/FILETIME134393577413203335保留rawExitnull，PATH摘要一致，旧40588不动。\n\n源码定位StartContentScope把RuntimeVersionPaths.package_root npm prefix当包目录，四审计源缺PACKAGE_SUBPATH；D角色CLI/sentinel/callsite同类需批量修，prefix的bun实际路径不改。启动器默认编码导致副本错误展示乱码，原nativeUTF8首错完整保留。宿主首轮park后无后续控制通道，新程序不得从PID/文本接管旧Job；完整同SID私有现场归档与原失败对象受控清理/新独立夹具仍待，不自动重试失败会话或补Closed。公开Start/allAccepted/production/typedClosedfalse，其余全版未完/历史清理原样，无官方/日常配置/PATH/策略/业务/模型/W2/双SID/发布。\n')
print(json.dumps(dict(rawFiles=len(rows),source='c870bd81f',firstActionMs=9842,osError=2,officialResumed=False,brokerRetained=True,fullArchivePending=True)))
