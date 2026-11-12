from pathlib import Path
import subprocess,json,hashlib,sqlite3,shutil,time,os
base=Path('/private/tmp/ocxd024-ready-recovery-kybrs_7j');native=json.loads((base/'native/runner-summary.json').read_text());work=Path(native['temporaryRoot']);root=work/'fixtures';profile=root/'profile';data=profile/'.codex-desktop/desktop';client=profile/'.codex';out=base/'current-recovery';out.mkdir()
code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes');binary=code/'apps/desktop/tauri/target/debug/deps/opencodex_desktop_lib-114f658f3f2e2b97';exe=root/'tests-after-status';shutil.copy2(binary,exe)
sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
assert sha(binary)==sha(exe)
index=data/'manager-state/client-session-index.sqlite'
def read_index():
 conn=sqlite3.connect(index.as_uri()+'?mode=ro',uri=True);head=json.loads(conn.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(v[0]) for v in conn.execute('SELECT document FROM events ORDER BY revision')];conn.close();return {'head':head,'events':events}
before=read_index();head=before['head'];assert head['state']=='ready' and head['revision']==4 and len(head['references']['phases'])==1
vault=data/'manager-state/client-originals'/head['references']['attempt'];assert vault.is_dir()
assert not (vault/'pre-stop.json').exists() and not (vault/'stopped.json').exists()
preserved=base/'before-recovery';preserved.mkdir();(preserved/'index.json').write_text(json.dumps(before,indent=2));shutil.copytree(vault,preserved/'registered-originals')
refs={p.name:sha(p) for p in vault.iterdir() if p.is_file()}
paths=[client/n for n in ['config.toml','opencodex-catalog.json','models_cache.json','opencodex-catalog-audit.jsonl']]+[profile/'.grok'/n for n in ['config.toml','config.toml.bak-opencodex','agents/AGENTS.md']]
source={'baseCommit':subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip(),'binarySha256':sha(exe),'files':[{'path':n,'sha256':sha(code/n)} for n in subprocess.check_output(['git','diff','--name-only'],cwd=code,text=True).splitlines()]};(out/'source.json').write_text(json.dumps(source,indent=2))
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_ALIAS_GRAPH':'1','OCXD024_MISSING_PLAN_ROOT':str(root),'OCXD024_COMPLETION_READBACK_ROOT':str(root),'OCXD024_REARM_GATE_ROOT':str(root)}
(out/'hypothesis.json').write_text(json.dumps({'accepted':False,'hypothesis':'Retain exact failed Stop/Ready refs1. Current explicit plan binds Original+Ready+current observation; no PreStop/Stopped. Separate Continue restores with fresh native absence and writer leases, no new service lifecycle. Status reconciliation of persisted Ready and actual stopped must be Required, not Active, without mutating history. Each mutation keeps10s budget.','originalIndexSha256':sha(index)},indent=2))
for label,name in [('ready-pending-status','commands::client_recovery::tests::inspect_owned_ready_pending_gate_without_service_actions'),('prepare-continue','commands::client_recovery::tests::prepare_owned_stopped_missing_plan_then_continue'),('full-readback','infrastructure::macos_client_originals::completion::tests::inspect_owned_completion_receipt_without_rewriting_it'),('final-status','commands::client_recovery::tests::inspect_owned_client_gate_after_rearm_without_service_actions')]:
 cmd=['/usr/bin/sandbox-exec','-f',str(work/'fixture.sb'),str(exe),'--exact',name,'--ignored','--nocapture','--test-threads=1'];start=time.monotonic()
 with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:p=subprocess.run(cmd,env=env,cwd=root,stdout=stdout,stderr=stderr,timeout=90)
 report={'accepted':False,'exitCode':p.returncode,'elapsedSeconds':round(time.monotonic()-start,3),'binarySha256':sha(exe),'command':cmd}
 (out/(label+'.json')).write_text(json.dumps(report,indent=2));print(label,p.returncode,flush=True)
 if p.returncode:break
for p in root.glob('missing-plan-*.json'):shutil.copy2(p,out/p.name)
if (root/'ready-pending-live-status.json').exists():shutil.copy2(root/'ready-pending-live-status.json',out/'ready-pending-live-status.json')
if (root/'rearm-final-live-status.json').exists():shutil.copy2(root/'rearm-final-live-status.json',out/'final-live-status.json')
after=read_index();(out/'index-final.json').write_text(json.dumps(after,indent=2));shutil.copytree(vault,out/'registered-originals')
assert all(sha(vault/n)==h for n,h in refs.items());assert not (vault/'pre-stop.json').exists() and not (vault/'stopped.json').exists()
(out/'summary.json').write_text(json.dumps({'accepted':False,'originalHistoryUnchanged':True,'noHistoricalPreStopOrStopped':True,'head':after['head'],'binarySha256':sha(exe),'lastExitCode':p.returncode if isinstance(p,subprocess.CompletedProcess) else report['exitCode']},indent=2))
shutil.copy2(Path(__file__),out/'recover_current.py')
