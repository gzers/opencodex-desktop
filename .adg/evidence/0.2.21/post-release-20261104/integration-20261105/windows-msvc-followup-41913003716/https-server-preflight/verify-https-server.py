from pathlib import Path
import json,hashlib,shutil,ssl,urllib.request,urllib.error,threading,importlib.util
from datetime import datetime,timezone
w=Path.cwd();b=w/'.toolchain/post021-msvc-followup';tls=b/'tls';tls.mkdir(exist_ok=True);(tls/'empty').mkdir(exist_ok=True)
for name in ['ca.pem','server.pem','server.key']:shutil.copyfile(w/'.toolchain/post021-windows-final/tls'/name,tls/name)
module_path=w/'post021-windows-materials/test/smoke/post021-windows-final/cloud-readonly-server.py';spec=importlib.util.spec_from_file_location('fixture_server',module_path);module=importlib.util.module_from_spec(spec);spec.loader.exec_module(module)
run=b/('https-preflight-'+datetime.now(timezone.utc).strftime('%Y%m%dT%H%M%S%fZ'));run.mkdir();stage=run/'stage.txt';stage.write_text('initial',encoding='utf-8');root=b/'cloud-fixtures-reviewed'
snapshot=lambda:{str(p.relative_to(root)):hashlib.sha256(p.read_bytes()).hexdigest()for p in root.rglob('*')if p.is_file()}
before=snapshot();server=module.FixtureServer(root,stage,run/'requests.jsonl',str(tls/'server.pem'),str(tls/'server.key'),port=0);thread=threading.Thread(target=server.serve_forever,daemon=True);thread.start();ctx=ssl.create_default_context(cafile=str(tls/'ca.pem'));results=[]
def request(path,method='GET',auth=True,expected=200):
 headers={'Authorization':'Basic c3ludGhldGljLXBvc3QwMjE6c3ludGhldGljLXBvc3QwMjEtcGFzc3dvcmQ='}if auth else{}
 req=urllib.request.Request('https://127.0.0.1:'+str(server.server_port)+path,headers=headers,method=method,data=b'forbidden-synthetic-write'if method=='PUT'else None)
 try:
  with urllib.request.urlopen(req,context=ctx,timeout=5)as response:status=response.status;body=response.read();etag=response.headers.get('ETag')
 except urllib.error.HTTPError as error:status=error.code;body=error.read();etag=error.headers.get('ETag')
 assert status==expected,(path,method,status,expected);results.append({'path':path,'method':method,'status':status,'etag':etag,'bytes':len(body)});return body,etag
try:
 request('/native021-only/config-v1-latest.json',auth=False,expected=401)
 pointers=[]
 for name in ['initial','later']:
  stage.write_text(name,encoding='utf-8');body,etag=request('/native021-only/config-v1-latest.json');pointer=json.loads(body);pointers.append(pointer)
  assert etag=='"'+hashlib.sha256(body).hexdigest()+'"'
  filename='config-v1-'+pointer['digest'][7:]+'.ocxd';payload,payload_etag=request('/native021-only/'+filename);assert 'sha256:'+hashlib.sha256(payload).hexdigest()==pointer['digest'];assert payload_etag=='"'+pointer['digest'][7:]+'"'
 assert pointers[0]!=pointers[1]
 request('/native021-only/config-v1-latest.json',method='PUT',expected=405)
 request('/native021-only/config-v1-latest.json',method='DELETE',expected=405)
 request('/native021-only/../fixture-manifest.json',expected=404)
 stage.write_text('unrecognized',encoding='utf-8');request('/native021-only/config-v1-latest.json',expected=503)
 stage.write_text('initial',encoding='utf-8')
 assert before==snapshot(),'server mutated fixture payloads'
 log=(run/'requests.jsonl').read_text(encoding='utf-8');assert 'Authorization'not in log and 'password'not in log
 report={'result':'passed-server-preflight-only','nativeAcceptance':False,'at':datetime.now(timezone.utc).isoformat(),'requests':results,'payloadBytesUnchanged':True,'rawCredentialsLogged':False,'serverThreadStopped':None}
finally:
 server.shutdown();server.server_close();thread.join(timeout=5)
report['serverThreadStopped']=not thread.is_alive();assert report['serverThreadStopped'];(run/'report.json').write_text(json.dumps(report,indent=2)+chr(10),encoding='utf-8',newline=chr(10));(b/'https-preflight-location.txt').write_text(str(run),encoding='utf-8');print(str(run));print(json.dumps({'requests':len(results),'result':report['result'],'stopped':report['serverThreadStopped']}))
