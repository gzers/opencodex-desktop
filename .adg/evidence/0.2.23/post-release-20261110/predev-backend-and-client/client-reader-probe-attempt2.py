from pathlib import Path
import json,subprocess,selectors,time,os
h=Path(__file__).parent.resolve();r=Path(json.loads((h/'client-fixture.json').read_text())['root']);ch=r/'codex';binary='/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex';checks=[]
env={'PATH':'/usr/bin:/bin','HOME':str(r/'user'),'TMPDIR':str(r/'tmp'),'CODEX_HOME':str(ch),'CODEX_SQLITE_HOME':str(ch),'OPENCODEX_HOME':str(r/'runtime'),'LANG':'C.UTF-8','RUST_LOG':'error'}
def run(label,args):
 stderr=open(r/'out'/(label+'.stderr.log'),'w');p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(r/'sandbox.sb'),binary]+args+['app-server','--stdio'],cwd=r,env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=stderr,bufsize=0);sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);pending=b'';responses=[]
 def send(x):p.stdin.write((json.dumps(x)+'\n').encode());p.stdin.flush()
 def receive(wanted):
  nonlocal pending
  until=time.monotonic()+10
  while time.monotonic()<until:
   if b'\n' in pending:
    line,pending=pending.split(b'\n',1)
    if not line:continue
    try:v=json.loads(line)
    except ValueError:continue
    responses.append(v)
    if v.get('id')==wanted:return v
   elif sel.select(0.2):
    b=os.read(p.stdout.fileno(),65536)
    if not b:raise RuntimeError('reader-exited')
    pending+=b
  raise TimeoutError('response-timeout')
 result={'label':label,'args':args,'modelListOnly':True}
 try:
  send({'id':1,'method':'initialize','params':{'clientInfo':{'name':'ocxd024-predev','version':'1'},'capabilities':{'experimentalApi':True}}});a=receive(1);assert 'result' in a,a;send({'method':'initialized','params':{}});send({'id':2,'method':'model/list','params':{'includeHidden':True}});reply=receive(2);assert 'result' in reply,reply;result['models']=[{'model':x['model'],'displayName':x['displayName']} for x in reply['result']['data']];result['state']='read';return result
 except Exception as e:result['state']='refused';result['error']=str(e);return result
 finally:
  p.stdin.close()
  try:p.wait(timeout=3)
  except subprocess.TimeoutExpired:p.terminate();p.wait(timeout=3)
  sel.close();stderr.close();result['exit']=p.returncode;(h/(label+'.json')).write_text(json.dumps({'result':result,'responses':responses},ensure_ascii=False,indent=2));(h/(label+'.stderr.log')).write_bytes((r/'out'/(label+'.stderr.log')).read_bytes())
def label(v):return next(x['displayName'] for x in v['models'] if x['model']=='fixture/gpt-6.1-sol')
# No CLI override: persisted user pointer is read by the real bundled client parser.
(ch/'config.toml').write_text('model_catalog_json = '+json.dumps(str(r/'display.json'))+'\n');a=run('client-persisted-pointer-r2',[]);assert label(a)=='团队/海外/gpt-6.1-sol';checks.append('persisted user config without CLI override reads exact full display label')
# Save/rename equivalent at the display file; client relaunch is needed and is actually done.
projected=json.loads((r/'display.json').read_text());projected['models'][0]['display_name']='新渠道 名/gpt-6.1-sol';(r/'display.json').write_text(json.dumps(projected,ensure_ascii=False));b=run('client-restart-renamed-r2',[]);assert label(b)=='新渠道 名/gpt-6.1-sol';checks.append('new actual client process reads renamed persisted label')
# Separate current documented profile file and explicit selection coexist with official root pointer.
(ch/'config.toml').write_text('model_catalog_json = '+json.dumps(str(r/'official.json'))+'\n');(ch/'ocxd-display.config.toml').write_text('model_catalog_json = '+json.dumps(str(r/'display.json'))+'\n');c=run('client-explicit-profile-r2',['--profile','ocxd-display']);assert c['state']=='refused';checks.append('app-server rejects --profile on this actual binary; cannot use profile flag as GUI shortcut');d=run('client-unselected-profile-r2',[]);assert label(d)=='fixture/gpt-6.1-sol';checks.append('unselected separate profile does not replace default catalog pointer')
# Restore original root bytes/pointer and restart, then verify actual original visible labels.
original='model_catalog_json = '+json.dumps(str(r/'official.json'))+'\n';(ch/'config.toml').write_text(original);f=run('client-pointer-rollback-r2',[]);assert label(f)=='fixture/gpt-6.1-sol';assert (ch/'config.toml').read_text()==original;checks.append('root pointer rollback and fresh reader restore original route/label')
(h/'client-reader-summary.json').write_text(json.dumps({'result':'pass','checks':checks,'count':len(checks),'scope':'Actual bundled Codex parser/model-list via fresh app-server processes; no GUI render or inference','manualGuiVerified':False,'originalSourceCatalogUnmodified':json.loads((r/'official.json').read_text())['models'][0]['display_name']=='fixture/gpt-6.1-sol'},ensure_ascii=False,indent=2));print('client checks',len(checks))
