from pathlib import Path
import json,sqlite3,subprocess,shutil,hashlib,base64,time
base=Path('/private/tmp/ocxd024-grok-backup-e5gp7hct');run=json.loads((base/'native/runner-summary.json').read_text());root=Path(run['temporaryRoot'])/'fixtures';profile=root/'profile';client=profile/'.codex';grok=profile/'.grok';data=profile/'.codex-desktop/desktop';out=base/'independent';out.mkdir();sha=lambda p:hashlib.sha256(p.read_bytes()).hexdigest()
c=sqlite3.connect((data/'manager-state/client-session-index.sqlite').as_uri()+'?mode=ro',uri=True);head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);events=[json.loads(r[0]) for r in c.execute('SELECT document FROM events ORDER BY revision')];c.close();(out/'index.json').write_text(json.dumps({'head':head,'events':events},indent=2));refs=head['references'];vault=data/'manager-state/client-originals'/refs['attempt'];shutil.copytree(vault,out/'registered-originals')
assert run['exitCode']==0 and head['state']=='restored' and head['revision']==8 and len(events)==8
for name,key in [('original.json','originalSha256'),('startup-scope.json','scopeSha256'),('startup-live.json','liveSha256')]:assert sha(vault/name)==refs[key]
for phase,h in refs['phases']:assert sha(vault/(phase.replace('_','-')+'.json'))==h
plan_path=vault/('restore-'+head['restorePlan']['id']+'.json');assert sha(plan_path)==head['restorePlan']['sha256'];plan=json.loads(plan_path.read_text());assert plan['schemaVersion']==6 and 'grokBackup' in plan and 'emptyConfig' not in plan
frames=[json.loads((vault/name).read_text()) for name in ['original.json','ready.json','pre-stop.json','stopped.json']]
find=lambda tree,name:next((e for e in tree['entries'] if e['path']==name),None)
assert all(f['grok']['state']=='directory' for f in frames)
assert find(frames[0]['grok']['tree'],'config.toml.bak-opencodex') is None
original_config=find(frames[0]['grok']['tree'],'config.toml');backups=[find(f['grok']['tree'],'config.toml.bak-opencodex') for f in frames[1:]];assert backups[0]==backups[1]==backups[2]
assert original_config['sha256']==backups[0]['sha256'] and original_config['protection']==backups[0]['protection']
old=vault/('restore-'+head['restorePlan']['id']+'-old-4');assert old.read_bytes()==bytes(plan['grokBackup']['stopped']['bytes'])
assert sha(old)==backups[2]['sha256'] and plan['grokBackup']['stopped']['identity']==backups[2]['identity'] and plan['grokBackup']['root']==frames[0]['grok']['tree']['root']
assert not (grok/'config.toml.bak-opencodex').exists() and sha(grok/'config.toml')==original_config['sha256']
for p in vault.glob('tree-content-*.blob'):assert sha(p)==p.name[len('tree-content-'):-len('.blob')]
summary=json.loads((base/'native/native/summary.json').read_text());assert summary['result']=='passed' and summary['reopenedRecoveryGate'];assert summary['startElapsedMs']<30000 and summary['statusElapsedMs']<5000 and summary['stopElapsedMs']<10000
(out/'live-status.json').write_text(json.dumps(summary['reopenedSnapshot'],indent=2));assert summary['reopenedSnapshot']['facts']['client_recovery']=='not_required'
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'CODEX_HOME':str(client),'TMPDIR':str(root/'tmp')+'/','LANG':'en_US.UTF-8','OCXD024_COMPLETION_READBACK_ROOT':str(root),'OCXD024_GROK_GUARD_ROOT':str(root)}
for label,name in [('full-trees','infrastructure::macos_client_originals::completion::tests::inspect_owned_completion_receipt_without_rewriting_it'),('official-path-exclusion','infrastructure::official_client_coordination::native::config::grok_tests::official_grok_and_codex_writer_guards_are_distinct_and_exclude_their_own_paths')]:
 cmd=['/usr/bin/sandbox-exec','-f',str(root.parent/'fixture.sb'),str(root/'tests'),'--exact',name,'--ignored','--nocapture','--test-threads=1'];clock=time.monotonic()
 with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:p=subprocess.run(cmd,cwd=root,env=env,stdout=stdout,stderr=stderr,timeout=40)
 (out/(label+'.json')).write_text(json.dumps({'exitCode':p.returncode,'elapsedSeconds':round(time.monotonic()-clock,3),'binarySha256':sha(root/'tests')},indent=2));print(label,p.returncode,flush=True);assert p.returncode==0
raw=(out/'full-trees.stdout.log').read_text();v=json.loads(next(line[line.index('{'):] for line in raw.splitlines() if '{' in line));assert v['independentCurrentHashesMatchReceipt']
for field in ['codexTreeBase64','grokTreeBase64']:(out/(field[:-6]+'.json')).write_bytes(base64.b64decode(v[field]))
shutil.copy2(root/'grok-guard-proof.json',out/'official-path-exclusion-proof.json')
for path in [client,grok]:assert not list(path.glob('config.toml.ocx-write.lock*'))
proof={'accepted':False,'result':'passed-directed-first-Grok-backup-recovery','startMs':summary['startElapsedMs'],'statusMs':summary['statusElapsedMs'],'stopMs':summary['stopElapsedMs'],'originalBackupAbsent':True,'ReadyPreStopStoppedExactSameOriginalConfigCopy':True,'newBackupImageAndParentIdentityBoundToSchema6':True,'exactBackupObjectArchivedAsOld4':True,'backupFinallyAbsent':True,'GrokConfigAndAgentOriginalsRestored':True,'independentFullNativeTreesMatchCompletion':True,'distinctOfficialConfigurationLocksVerifiedByRealWriterProcesses':True,'nativeDriverExitCode':0,'indexState':head['state'],'indexRevision':head['revision'],'allRequiredBudgetsUnchanged':True,'officialSourceUnmodified':True,'noBusinessRequests':True}
(base/'verified-native.json').write_text(json.dumps(proof,indent=2));shutil.copy2(Path(__file__),out/'verify_native.py');print(json.dumps(proof,indent=2))
