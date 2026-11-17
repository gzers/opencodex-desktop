from pathlib import Path
import base64,hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent;DOCS=ROOT.parents[4]
PUBLIC=Path('C:/Users/Public/ocx-r6-profile-preflight-20261117-r1')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest()
files=[];read_errors=[]
for p in sorted((PUBLIC/'out01').iterdir()):
    assert p.is_file() and not p.is_symlink() and p.stat().st_size<1048576
    try:raw=p.read_bytes()
    except PermissionError as e:
        read_errors.append(dict(path=str(p),type=type(e).__name__,message=str(e)));continue
    target=ROOT/'raw-public'/p.name;target.parent.mkdir(exist_ok=True)
    if target.exists():assert target.read_bytes()==raw
    else:
        with target.open('xb') as out:out.write(raw)
    files.append(dict(path=target.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw),source=str(p)))
export=load(ROOT/'raw-public/same-sid-export.json')
assert export['complete'] is True and export['fileErrors']==[] and export['sid'].endswith('-1003') and export['session']==7
exported=[]
for row in export['files']:
    rel=Path(row['relative']);assert not rel.is_absolute() and '..' not in rel.parts and rel.parts[0] in ['public','preparation']
    raw=base64.b64decode(row['base64'],validate=True);assert len(raw)==row['bytes'] and sha(raw)==row['sha256']
    p=ROOT/'same-sid-decoded'/rel;p.parent.mkdir(parents=True,exist_ok=True)
    with p.open('xb') as out:out.write(raw)
    if rel.parts[0]=='public':
        prior=ROOT/'raw-public'/rel.name
        if prior.exists():assert raw==prior.read_bytes()
        else:
            with prior.open('xb') as out:out.write(raw)
            files.append(dict(path=prior.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw),source='actual same-SID task export/'+rel.as_posix(),controllerDirectRead=False))
    exported.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
launch=load(ROOT/'raw-public/launch-result.json');pre=load(ROOT/'raw-public/prerequisites.json');result=load(ROOT/'raw-public/preparation-result.json')
assert launch['rawTestExit']==0 and launch['preparedExistingRoots'] and launch['dailyPathUnchanged'] and not launch['elevated']
assert pre['outerJobAbsent'] and pre['profilePin'] and pre['actualKnownFolderProfile']=='C:\\Users\\marenben'
assert pre['elevationType']==1 and pre['session']==7 and pre['sid'].endswith('-1003')
assert all(r['state']=='Absent' for r in pre['roots'])
assert result['preparedExistingBaseline'] and result['firstError'] is None and len(result['rows'])==10
assert not result['officialInitializationProven'] and not result['originalAbsentRetirementProven'] and not result['actualProductionExecuted'] and not result['startOpen']
required=[1,2,4,16,32,64,128,256]
for s in [pre['profileSecurity']]+[r['security'] for r in result['rows']]:
    assert s['scopeId']=='windows-bounded-access-v1' and not s['fullSecurityProven'] and s['auditSaclAssurance']=='excluded-unknown'
    assert [v['requestedMask'] for v in s['projections']]==required
    assert all(v['nativeStatus']==0 for v in s['projections'])
for i in range(10):
    for suffix in ['intent','result']:assert (ROOT/f'same-sid-decoded/preparation/prepare-{i:03}-{suffix}.json').is_file()
assert load(ROOT/'same-sid-decoded/preparation/result.json')==result
status=dict(contract='TEAM16/IMP142/r6',sourceHead='f5fd8211e5e6aeafc0c025887175c1f5905ab5af',rawNativeExit=0,nativePassed=1,ordinarySid=pre['sid'],ordinarySession=7,ordinaryElevationType=1,noOuterJob=True,exactKnownFolderProfile=pre['actualKnownFolderProfile'],requiredProjections=required,originalClientRootsBeforePreparation='Absent',syntheticExistingBaselinePrepared=True,createdObjects=result['rows'],dailyPathUnchanged=True,ordinaryProductionExecuted=False,officialInitializationProven=False,originalAbsentRetirementProven=False,typedClosedQualified=False,startOpen=False,allAccepted=False,rawPublicFiles=files,sameSidDecodedFiles=exported,firstError=None,controllerDirectReadErrors=read_errors,receiverToolError='Initial collector attempted direct private preparation JSON read; original PermissionError retained. Exact original bytes received through already-created same-SID bounded export with hash verification; no permission changes or native rerun.',cleanup=dict(accountProfileRetained=True,taskAndSyntheticBaselineRetained=True,removed=0,historicalRejectedDeletionRetried=False,overallCleanupComplete=False),next='Prepare existing native coordination lock files and fixed official selection under same SID; source-bound native Owner/lease+sideeffects+canonical Stop+successor/Status and real GUI manager continuity still required. Public gate unchanged.',W2=False,dualSID=False,published=False)
with (ROOT/'result.json').open('x',encoding='utf-8') as out:json.dump(status,out,ensure_ascii=False,indent=2)
with (ROOT/'report.md').open('x',encoding='utf-8') as out:out.write(f'''# 普通桌面已有根测试基线实际结果

原源码f5fd8211e/测试EXE d76a8de7448bf5b71219fa3e9737889899dc913ca35bed482434de3a861519aa，在marenben SID1003/session7真实桌面执行1过0败/raw0。elevated=false/type1、原KnownFolder Profile、noOuterJob、Profile及10个create-new对象固定8投影均成功；auditExcludedUnknown/fullSecurityProven=false保持。User/Machine PATH摘要不变。

准备前.codex/.grok/精确任务base均Absent，按原SID建立10个无凭据测试对象，20份intent/result和initial-absence/result完整导出。{len(files)}份原Public文件、{len(exported)}份原SIDbase64解码件按长度/摘要核，Public两JSON与解码件原字节相等。它是existing-root后续夹具基线，原先Absent事实未消除，不是官方初始化、NativeLease或originalAbsent安全退根/Closed/生产证明。原对象留供后续native捕获；无清理/旧拒绝重试。

生产Start仍拒绝；下一步补齐同SID实际稳定协调域与固定原包选择/原Owner，再做lease+副作用+canonical Stop/Status/后继及真实GUI官方owner组合。无本轮首错，无官方/日常配置/PATH/策略更改，无业务/模型请求，无W2/双SID/发布。
''')
p=DOCS/'docs/03-开发实施/0.2.24-contracts/windows-dev-handoff.md'
heading='## 2026-11-17 r6普通桌面已有根基线实际准备结果'
assert heading not in p.read_text(encoding='utf-8-sig')
with p.open('a',encoding='utf-8') as out:out.write(f'\n\n{heading}\n\n实际marenben SID1003/session7/elevatedfalse/type1/noJob/原KnownFolder Profile执行f5fd8211e准备入口1过0败/raw0，固定8投影在Profile和10个create-new无凭据对象均完整nativeStatus0，审计ExcludedUnknown/fullSecurityProvenfalse未改。User/Machine PATH摘要一致。证据.adg/evidence/0.2.24/windows-dev/{ROOT.name}/；{len(files)}份Public原日志及{len(exported)}份原SID导出逐字长度/摘要核，20intent/result+initial-absence/result齐备，原现场保留。\n\n准备前.codex/.grok/base真实Absent单列；此后仅作为existing-root测试基线，不签官方初始化/原缺席Codex无空窗退根/生产Closed许可。尚缺同SID真实协调域固定原包/原Owner，继续完整原包lease/副作用/canonical Stop/Status/后继与真GUI官方owner，不重跑旧空GUI。Start/allAccepted/production/typedClosedfalse，旧首错与所有历史清理/普通MSI/CAP-trust及其它未完保留。无本轮首错，无删除/旧拒绝重试，无官方/日常配置/PATH/账户策略修改、业务/模型/W2/双SID/发布。\n')
print(json.dumps(dict(rawExit=0,objects=10,publicFiles=len(files),decoded=len(exported),ordinaryNativePreparationPassed=True,actualProductionExecuted=False)))
