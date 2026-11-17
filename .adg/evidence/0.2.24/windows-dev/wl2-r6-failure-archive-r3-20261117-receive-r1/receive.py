from pathlib import Path
import hashlib,json,subprocess
# Temporary receiver is staged only into its exact evidence round after run.
DOCS=Path(__file__).resolve().parents[5]
ROOT=Path(__file__).resolve().parent
PUBLIC=Path('C:/Users/Public/ocx-r6-failure-archive-20261117-r3')
load=lambda p:json.loads(p.read_text(encoding='utf-8-sig'));sha=lambda b:hashlib.sha256(b).hexdigest()
v=load(PUBLIC/'out01/archive-result.json')
assert v['archiveVerified'] and not v['fileErrors'] and not v['forcedCleanupExecuted'] and not v['nativeNormalStopProven'] and not v['nativeOwnerAdopted'] and v['originalObjectsRetained'] and not v['startOpen']
assert v['sid']=='S-1-5-21-587408064-856152790-2583328096-1003' and v['session']==7
for kind,pid,time in [('broker',54692,'134393577413203335'),('launcher',50580,'134393577919362322')]:
    before=v[kind+'Before'];after=v[kind+'After'];assert before==after and before['pid']==pid and before['filetime']==time and before['session']==7 and not before['exited']
assert v['archive']['path']=='C:'+chr(92)+'Users'+chr(92)+'marenben'+chr(92)+'ocx024-r6-private-failure-archive-20261117-r3'+chr(92)+'failed-existing-root-original-bytes.zip'
names=set();total=0;groups={}
for f in v['files']:
    rel=f['relative'];parts=rel.split('/');assert rel not in names and all(p not in ['', '.', '..'] for p in parts);names.add(rel)
    assert parts[0] in ['failed-task','synthetic-codex','synthetic-grok','synthetic-coordination']
    assert 0<=f['bytes']<=536870912 and len(f['sha256'])==64 and all(c in '0123456789abcdef' for c in f['sha256'])
    total+=f['bytes'];groups[parts[0]]=groups.get(parts[0],0)+1
assert total==v['totalSourceBytes'] and total<=2147483648 and len(names)<=20000
assert groups['synthetic-codex']==4 and groups['synthetic-grok']==1 and groups['synthetic-coordination']==3
assert {'failed-task/D/qualification-result.json','failed-task/D/qualification-start1.json','failed-task/I/ocxd.exe','failed-task/I/opencodex-desktop.exe','failed-task/I/ocx.exe','synthetic-codex/config.toml','synthetic-codex/auth.json'}<=names
stage=load(DOCS/'.adg/evidence/0.2.24/windows-dev/wl2-r6-existing-official-qualification-20261117-r1/staged-input.json')
for r in stage['files']:
    if r['name'].endswith('.exe'):
        x=next(f for f in v['files'] if f['relative']=='failed-task/I/'+r['name']);assert x['bytes']==r['bytes'] and x['sha256']==r['sha256']
assert not (ROOT/'result.json').exists()
(ROOT/'.gitattributes').write_bytes(b'* -text'+bytes([10]))
rows=[]
for name in ['out01/archive-result.json','archive-r6-failed-preresume.cmd','archive-r6-failed-preresume.ps1','archive-r6-process-facts.cs','archive-r6-content.ps1','input-manifest.json']:
    raw=(PUBLIC/name).read_bytes();p=ROOT/'raw'/name;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(raw);rows.append(dict(path=p.relative_to(ROOT).as_posix(),bytes=len(raw),sha256=sha(raw)))
result=dict(contract='TEAM16/IMP142/r6',archiveSourceHead='c0d42f8dcc72d56b84fb5bbea050efe2d66ee65f',failedProductionSource='c870bd81fcbd65310ff79d262c9c49fc5a2c6f62',archiveVerifiedInOriginalSID=True,archiveIndependentlyReadByController=False,archive=v['archive'],members=len(names),totalSourceBytes=total,groups=groups,directories=len(v['directories']),publicRawFiles=rows,originalBroker=v['brokerAfter'],originalSuspendedLauncher=v['launcherAfter'],firstProductionError=v['firstError'],actualProductionExecuted=False,forcedCleanupExecuted=False,nativeNormalStopProven=False,zipIsNTFSMetadataClone=False,originalObjectsRetained=True,startOpen=False,allAccepted=False,scopeNote='Original ordinarySID helper verified complete source and ZIP bytes/hash/membership beforeafter, independent receiver verifies exact complete receipt shape and known c870 product hashes. Private ZIP not independently reopened by controller; no identity/security cloning claim.',remaining='Loaded c870 debug broker lacks cleanup control channel and holds original Owner/Job. Rebuilt process cannot adopt them. Need explicit controlled failure-cleanup/new-independent-fixture plan; no same-session retry or inferredClosed. Other original fullscope failures/unknowns remain.',W2=False,dualSID=False,published=False)
(ROOT/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2),encoding='utf-8')
(ROOT/'report.md').write_text(chr(10).join(['# r3普通桌面完整失败归档结果','',str(len(names))+'文件、'+str(total)+'源字节、'+str(len(v['directories']))+'目录在marenben SID1003/session7完整归档及逐member/源hash/前后membership核验成功。原broker54692/Node50580同FILETIME前后仍存活、未恢复未停止，原首错os2保留。', '', '私有ZIP '+v['archive']['path']+'，SHA256 '+v['archive']['sha256']+'。接收端核全部receipt字段/分组/原c870产品3摘要，未独立重开私有ZIP；ZIP不是NTFS身份/权限克隆。原失败scene及所有原对象留，无删除/kill/策略更改/原包重跑。','',result['remaining']])+chr(10),encoding='utf-8')
print(json.dumps(dict(members=len(names),sourceBytes=total,groups=groups,archiveVerifiedInOriginalSID=True,brokerRetained=True,originalObjectsRetained=True)))
