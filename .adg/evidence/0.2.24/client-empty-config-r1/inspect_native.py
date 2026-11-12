from pathlib import Path
import json,sqlite3,shutil,subprocess,time,hashlib,os,base64
base=Path('/private/tmp/ocxd024-empty-config-d8t6p91e');run=json.loads((base/'native/runner-summary.json').read_text());root=Path(run['temporaryRoot'])/'fixtures';profile=root/'profile';data=profile/'.codex-desktop/desktop';client=profile/'.codex';out=base/'independent';out.mkdir()
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
conn=sqlite3.connect((data/'manager-state/client-session-index.sqlite').as_uri()+'?mode=ro',uri=True);head=json.loads(conn.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(row[0]) for row in conn.execute('SELECT document FROM events ORDER BY revision')];conn.close();(out/'index.json').write_text(json.dumps({'head':head,'events':events},indent=2));refs=head['references'];vault=data/'manager-state/client-originals'/refs['attempt'];shutil.copytree(vault,out/'registered-originals')
assert not (client/'config.toml').exists();assert not (profile/'.grok').exists()
summary=json.loads((base/'native/native/summary.json').read_text());assert summary['result']=='passed' and run['exitCode']==0
assert head['state']=='restored' and head['revision']==8 and len(events)==8 and len(refs['phases'])==3
for n,k in [('original.json','originalSha256'),('startup-scope.json','scopeSha256'),('startup-live.json','liveSha256')]:assert sha(vault/n)==refs[k]
for phase,h in refs['phases']:assert sha(vault/(phase.replace('_','-')+'.json'))==h
plan_path=vault/('restore-'+head['restorePlan']['id']+'.json');assert sha(plan_path)==head['restorePlan']['sha256'];plan=json.loads(plan_path.read_text());assert plan['schemaVersion']==5 and plan['emptyConfig']['stopped']['state']=='file'
old=vault/('restore-'+head['restorePlan']['id']+'-old-3');assert old.read_bytes()==bytes(plan['emptyConfig']['stopped']['bytes'])==b'\n'
frames=[json.loads((vault/n).read_text()) for n in ['original.json','ready.json','pre-stop.json','stopped.json']]
find=lambda f:next((e for e in f['tree']['entries'] if e['path']=='config.toml'),None)
assert find(frames[0]) is None and find(frames[1]) and find(frames[2]) and find(frames[3]);assert find(frames[1])==find(frames[2])
assert find(frames[3])['identity']==plan['emptyConfig']['stopped']['identity'] and find(frames[3])['sha256']==sha(old)
assert find(frames[3])['protection']==plan['emptyConfig']['stopped']['protection']
for f in frames:assert f['grok']['state']=='absent'
for p in vault.glob('tree-content-*.blob'):assert sha(p)==p.name[len('tree-content-'):-len('.blob')]
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_COMPLETION_READBACK_ROOT':str(root),'OCXD024_REARM_GATE_ROOT':str(root)}
for label,name in [('full-readback','infrastructure::macos_client_originals::completion::tests::inspect_owned_completion_receipt_without_rewriting_it'),('final-state','commands::client_recovery::tests::inspect_owned_client_gate_after_rearm_without_service_actions')]:
 cmd=['/usr/bin/sandbox-exec','-f',str(root.parent/'fixture.sb'),str(root/'tests'),'--exact',name,'--ignored','--nocapture','--test-threads=1'];start=time.monotonic()
 with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:p=subprocess.run(cmd,cwd=root,env=env,stdout=stdout,stderr=stderr,timeout=30)
 (out/(label+'.json')).write_text(json.dumps({'exitCode':p.returncode,'elapsedSeconds':round(time.monotonic()-start,3),'binarySha256':sha(root/'tests')},indent=2));print(label,p.returncode,flush=True);assert p.returncode==0
raw=(out/'full-readback.stdout.log').read_text();projection=json.loads(next(line[line.index('{'):] for line in raw.splitlines() if '{' in line));assert projection['independentCurrentHashesMatchReceipt']
for field in ['codexTreeBase64','grokTreeBase64']:(out/(field[:-6]+'.json')).write_bytes(base64.b64decode(projection[field]))
shutil.copy2(root/'rearm-final-live-status.json',out/'live-state.json')
proof={'accepted':False,'result':'passed-directed-first-use-empty-config-restoration','startMs':summary['startElapsedMs'],'statusMs':summary['statusElapsedMs'],'stopMs':summary['stopElapsedMs'],'originalConfigAbsent':True,'readyAndPreStopExactRecordedFile':True,'stoppedWhitespaceImageBoundToPlan':True,'movedOriginalStoppedObjectArchived':True,'finalConfigAbsent':True,'originalGrokAbsencePreserved':True,'independentCompleteNativeTreesMatchReceipt':True,'finalIndexState':head['state'],'finalRevision':head['revision'],'officialRuntimeUnmodified':True,'noBusinessRequests':True}
assert proof['startMs']<30000 and proof['statusMs']<5000 and proof['stopMs']<10000
(base/'verified-native.json').write_text(json.dumps(proof,indent=2));shutil.copy2(Path(__file__),out/'inspect_native.py');print(json.dumps(proof,indent=2))
