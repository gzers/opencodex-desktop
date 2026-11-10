from pathlib import Path
import tempfile,json,hashlib,os
here=Path(__file__).parent.resolve();roots=here/'roots.json';prior=json.loads(roots.read_text()) if roots.exists() else [json.loads((here/'fixture.json').read_text())['root']];r=Path(tempfile.mkdtemp(prefix='ocxd299-five-artifacts-preexisting-')).resolve()
for d in ['user','tmp','out','runtime','codex','claude','grok']:(r/d).mkdir()
policy='(version 1)\n(allow default)\n(deny network*)\n(deny file-write*)\n(allow file-write* (subpath '+json.dumps(str(r))+'))\n'
for p in ['/Users/ezio/.codex','/Users/ezio/.opencodex','/Users/ezio/.codex-desktop','/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/opencodex-home','/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/manager-state']:policy+='(deny file-read* (subpath '+json.dumps(p)+'))\n'
paths=[];digest=hashlib.sha256(str(r/'codex').encode()).hexdigest();shared=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()))
for domain in ['catalog-write-locks','native-write-locks']:
 for suffix in ['','-journal','-wal','-shm']:
  p=shared/domain/(digest+'.sqlite'+suffix);assert not p.exists();paths.append(str(p));policy+='(allow file-write* (literal '+json.dumps(str(p))+'))\n'
 # SQLite adoption creates and atomically publishes one new db for this exact private home.
 pattern='^'+str(shared/domain)+'/[.]'+digest+'[.]sqlite[.]adopt-[0-9]+-[a-f0-9-]+[.]tmp(-journal|-wal|-shm)?$';policy+='(allow file-write* (regex #"'+pattern.replace('\\','\\\\')+'"))\n'
home=str(r/'codex');state=str(r/'codex/state_5.sqlite');history=hashlib.sha256((str(len(home))+':'+home+str(len(state))+':'+state).encode()).hexdigest()
for suffix in ['','-journal','-wal','-shm']:
 p=shared/'history-write-locks'/(history+'.sqlite'+suffix);assert not p.exists();paths.append(str(p));policy+='(allow file-write* (literal '+json.dumps(str(p))+'))\n'
(r/'sandbox.sb').write_text(policy);(r/'fixture-lock-paths.json').write_text(json.dumps(paths));prior.append(str(r));roots.write_text(json.dumps(prior,indent=2));(here/'fixture-attempt4.json').write_bytes((here/'fixture.json').read_bytes());(here/'fixture.json').write_text(json.dumps({'root':str(r),'exactNewLockPaths':paths},indent=2));(here/'sandbox-preexisting.sb').write_text(policy);print(r)
