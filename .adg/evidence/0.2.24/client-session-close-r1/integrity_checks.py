from pathlib import Path
import json,subprocess,hashlib,sqlite3,os,stat
base=Path('/private/tmp/ocxd024-session-close-v9lwsp8w/evidence');records=[json.loads(l) for l in (base/'read-only-build.jsonl').read_text().splitlines() if l.startswith('{')];exe=Path(next(r['executable'] for r in records if r.get('reason')=='compiler-artifact' and r.get('target',{}).get('name')=='opencodex_desktop_lib' and r.get('executable')));r=json.loads((base/'native/runner-summary.json').read_text());root=Path(r['temporaryRoot'])/'fixtures';data=root/'profile/.codex-desktop/desktop';idx=data/'manager-state/client-session-index.sqlite';c=sqlite3.connect(idx.as_uri()+'?mode=ro&immutable=1',uri=True);head=json.loads(c.execute('SELECT document FROM current_session').fetchone()[0]);c.close();vault=data/'manager-state/client-originals'/head['references']['attempt'];out=base/'integrity';out.mkdir();summary={'readOnlyBinarySha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'actualProductAlgorithmUnchangedFromNativeRound':True,'checks':[]}
index_before=idx.read_bytes()
def observe(label,expect):
 with (out/(label+'.stdout.log')).open('wb') as stdout,(out/(label+'.stderr.log')).open('wb') as stderr:p=subprocess.run([str(exe),'--exact','infrastructure::macos_client_originals::session_index::tests::inspect_owned_closed_index_without_starting_or_repairing','--ignored','--nocapture'],env=dict(os.environ,OCXD024_CLOSED_INDEX_ROOT=str(root),OCXD024_CLOSED_INDEX_EXPECT=expect,TMPDIR='/private/tmp/'),stdout=stdout,stderr=stderr)
 assert p.returncode==0,label
 lines=[l for l in (out/(label+'.stdout.log')).read_text().splitlines() if l.startswith('{')];assert len(lines)==1;value=json.loads(lines[0]);assert value['observed']==expect and idx.read_bytes()==index_before
 summary['checks'].append({'case':label,'expected':expect,'result':value,'indexBytesUnchanged':True})
observe('valid-before','restored')
for label,path,kind in [('receipt-missing',vault/'completion-readback.json','missing'),('receipt-corrupt',vault/'completion-readback.json','corrupt'),('outcome-corrupt',vault/('restore-'+head['restorePlan']['id']+'-outcome.json'),'corrupt'),('scope-corrupt',vault/'startup-scope.json','corrupt'),('live-corrupt',vault/'startup-live.json','corrupt')]:
 raw=path.read_bytes();mode=stat.S_IMODE(path.stat().st_mode);saved=path.with_name(path.name+'.integrity-test-saved')
 if kind=='missing':path.rename(saved)
 else:path.chmod(0o600);path.write_bytes(b'{}');path.chmod(mode)
 try:observe(label,'refused')
 finally:
  if kind=='missing':saved.rename(path)
  else:path.chmod(0o600);path.write_bytes(raw);path.chmod(mode)
 assert path.read_bytes()==raw and stat.S_IMODE(path.stat().st_mode)==mode
observe('valid-after-owned-fixture-restore','restored')
config=root/'profile/.codex/config.toml';raw=config.read_bytes();config.write_bytes(b"model = 'user-edit-after-completed-session'\n")
try:observe('legitimate-user-config-edit-after-closure','restored')
finally:config.write_bytes(raw)
assert idx.read_bytes()==index_before
(out/'summary.json').write_text(json.dumps(summary,indent=2));print('read-only integrity checks passed',len(summary['checks']))
