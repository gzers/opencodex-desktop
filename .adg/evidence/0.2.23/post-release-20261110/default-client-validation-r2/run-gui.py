from pathlib import Path
import json,subprocess,time,os,signal,hashlib
c=json.loads(Path('/private/tmp/ocxd24v-active.json').read_text());r=Path(c['root']);e=Path(c['evidence']);label=__import__('sys').argv[1];launch=json.loads((e/'gui-outer-sandbox-launch.json').read_text());portfile=r/'u/Library/Application Support/Codex/DevToolsActivePort';portfile.unlink(missing_ok=True);p=subprocess.Popen(launch['argv'],env=launch['env'],cwd=r,stdout=open(e/(label+'.stdout.log'),'w'),stderr=open(e/(label+'.stderr.log'),'w'),start_new_session=True);c['guiPid']=p.pid;Path('/private/tmp/ocxd24v-active.json').write_text(json.dumps(c));(e/(label+'-process.json')).write_text(json.dumps({'pid':p.pid,'pgid':p.pid},indent=2))
for _ in range(60):
 if p.poll()!=None:break
 if portfile.exists():break
 time.sleep(.2)
assert p.poll() is None
q=subprocess.run(['/Users/ezio/.local/bin/node',str(r/'observe-v2.cjs'),str(r),str(e),label],cwd=r,capture_output=True,text=True,timeout=50);(e/(label+'-observation.stdout.log')).write_text(q.stdout);(e/(label+'-observation.stderr.log')).write_text(q.stderr);print(q.stdout,q.stderr);print('observer exit',q.returncode)
try:p.wait(timeout=10)
except subprocess.TimeoutExpired:os.killpg(p.pid,signal.SIGTERM);p.wait(timeout=5)
(e/(label+'-execution.json')).write_text(json.dumps({'appPid':p.pid,'observerExit':q.returncode,'appExit':p.returncode,'catalogSha256After':hashlib.sha256((r/'h/catalog.json').read_bytes()).hexdigest()},indent=2));assert q.returncode==0
