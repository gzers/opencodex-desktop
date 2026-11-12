from pathlib import Path
import os,json,subprocess,shutil,sqlite3,time,hashlib
base=Path('/private/tmp/ocxd024-first-clients-izsl870d')
for label in ['grok-absent','codex-empty']:
 run=json.loads((base/label/'runner-summary.json').read_text());root=Path(run['temporaryRoot'])/'fixtures';profile=root/'profile';data=profile/'.codex-desktop/desktop';exe=root/'tests';out=base/label/'independent-readback';out.mkdir()
 env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(profile/'.codex'),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_COMPLETION_READBACK_ROOT':str(root),'OCXD024_REARM_GATE_ROOT':str(root)}
 for case,name in [('full-trees','infrastructure::macos_client_originals::completion::tests::inspect_owned_completion_receipt_without_rewriting_it'),('live-state','commands::client_recovery::tests::inspect_owned_client_gate_after_rearm_without_service_actions')]:
  start=time.monotonic();cmd=['/usr/bin/sandbox-exec','-f',str(root.parent/'fixture.sb'),str(exe),'--exact',name,'--ignored','--nocapture','--test-threads=1']
  with (out/(case+'.stdout.log')).open('wb') as stdout,(out/(case+'.stderr.log')).open('wb') as stderr:p=subprocess.run(cmd,cwd=root,env=env,stdout=stdout,stderr=stderr,timeout=30)
  (out/(case+'.json')).write_text(json.dumps({'exitCode':p.returncode,'elapsedSeconds':round(time.monotonic()-start,3),'binarySha256':hashlib.sha256(exe.read_bytes()).hexdigest()},indent=2));print(label,case,p.returncode,flush=True)
  assert p.returncode==0
 c=sqlite3.connect((data/'manager-state/client-session-index.sqlite').as_uri()+'?mode=ro',uri=True);head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(row[0]) for row in c.execute('SELECT document FROM events ORDER BY revision')];c.close()
 (out/'index.json').write_text(json.dumps({'head':head,'events':events},indent=2));assert head['state']=='restored'
 shutil.copytree(data/'manager-state/client-originals'/head['references']['attempt'],out/'registered-originals')
 if (root/'rearm-final-live-status.json').exists():shutil.copy2(root/'rearm-final-live-status.json',out/'live-status.json')
 original=json.loads((out/'registered-originals/original.json').read_text());assert original['grok']['state']=='absent';assert not (profile/'.grok').exists()
 (out/'absence-check.json').write_text(json.dumps({'originalGrokAbsent':True,'currentGrokAbsent':True,'terminalState':'restored','accepted':False},indent=2))
