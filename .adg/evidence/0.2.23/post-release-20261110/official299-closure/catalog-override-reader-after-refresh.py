from pathlib import Path
import copy,json,subprocess,time,selectors,os,hashlib
r=Path('/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocxd-official299-gates-1g7e493p').resolve();e=Path(__file__).parent.resolve();a=e/'catalog-override-reader-after-official-refresh';a.mkdir(exist_ok=False)
binary='/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex'
ch=r/'user/codex';ch.mkdir(exist_ok=True);official=r/'out/official-catalog.json';source=official.read_bytes();base=json.loads(source)
candidate=r/'out/desktop-catalog.json';catalog=json.loads(candidate.read_text());rows=catalog['models'];names=['Krill-Renamed','中文渠道','Team One','团队/海外']
(ch/'config.toml').write_text('model_catalog_json = '+json.dumps(str(official))+'\n')
env={'PATH':'/usr/bin:/bin','HOME':str(r/'user'),'TMPDIR':str(r/'tmp'),'CODEX_HOME':str(ch),'OPENCODEX_HOME':str(r/'user/opencodex'),'LANG':'en_US.UTF-8','RUST_LOG':'error'}
err=open(r/'out/reader-stderr.log','w');p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(r/'probe.sb'),binary,'app-server','--stdio','-c','model_catalog_json='+json.dumps(str(candidate))],cwd=r,env=env,stdin=subprocess.PIPE,stdout=subprocess.PIPE,stderr=err,bufsize=0);responses=[];sel=selectors.DefaultSelector();sel.register(p.stdout,selectors.EVENT_READ);pending=b''
def send(x):p.stdin.write((json.dumps(x)+'\n').encode());p.stdin.flush()
def receive(wanted,seconds=12):
 global pending
 until=time.monotonic()+seconds
 while time.monotonic()<until:
  if b'\n' in pending:
   line,pending=pending.split(b'\n',1)
   if not line:continue
   try:x=json.loads(line)
   except ValueError:continue
   responses.append(x)
   if x.get('id')==wanted:return x
   continue
  ready=sel.select(min(0.25,max(0,until-time.monotonic())))
  if ready:
   data=os.read(p.stdout.fileno(),65536)
   if not data:raise RuntimeError('reader exited before response '+str(wanted))
   pending+=data
 raise TimeoutError('reader response timeout '+str(wanted))
result={'scope':'Real official-generated rows + Desktop-only display_name projection + explicit client override consumed by real Codex model/list; no session/turn/model inference','officialCatalogUnchanged':False,'accepted':False}
try:
 send({'id':1,'method':'initialize','params':{'clientInfo':{'name':'ocxd299-closure','version':'1'},'capabilities':{'experimentalApi':True}}});init=receive(1);assert 'result' in init,init
 send({'method':'initialized','params':{}});send({'id':2,'method':'model/list','params':{'includeHidden':True}});reply=receive(2);assert 'result' in reply,reply
 data=reply['result']['data'];observed={x.get('model'):x.get('displayName') for x in data}
 for row in rows:assert observed.get(row['slug'])==row['display_name'],(row['slug'],observed.get(row['slug']))
 assert official.read_bytes()==source
 result.update(accepted=True,officialCatalogUnchanged=True,codexVersion='0.167.0-alpha.3.2',modelCount=len(data),onDiskPointerStillOfficial=json.loads((ch/'config.toml').read_text().split('=',1)[1].strip())==str(official),cases=[{'route':x['slug'],'label':observed[x['slug']]} for x in rows],nonDisplayFieldsUnchanged=True)
except Exception as exc:result['error']=str(exc);raise
finally:
 p.stdin.close()
 try:p.wait(timeout=3)
 except subprocess.TimeoutExpired:p.terminate();p.wait(timeout=3)
 err.close();sel.close();result['readerExitCode']=p.returncode;result['readerExited']=p.poll() is not None
 (a/'result.json').write_text(json.dumps(result,ensure_ascii=False,indent=2));(a/'responses.json').write_text(json.dumps(responses,ensure_ascii=False,indent=2));(a/'stderr.log').write_bytes((r/'out/reader-stderr.log').read_bytes());print(json.dumps(result,ensure_ascii=False))
