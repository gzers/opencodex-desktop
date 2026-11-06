from pathlib import Path
import base64,concurrent.futures,datetime,hashlib,json,ssl,time,urllib.request,urllib.error,urllib.parse,uuid,xml.etree.ElementTree as ET
ROOT=Path(__file__).parent;run=str(uuid.uuid4());child='ocxd-validation-'+run;trace=[];checks=[];inputs={};created=False;cleanup={'attempted':False};owned=set()
sha=lambda b:'sha256:'+hashlib.sha256(b).hexdigest()
enc=lambda v:json.dumps(v,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode()
class Fault(Exception):pass
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*a,**k):return None
opener=urllib.request.build_opener(urllib.request.HTTPSHandler(context=ssl.create_default_context()),NoRedirect())
def local(p):
 b=p.read_bytes();inputs[p]=hashlib.sha256(b).hexdigest();return json.loads(b)
def request(method,name='',body=None,parent=False):
 assert method in ['GET','PROPFIND','MKCOL','PUT','DELETE'];assert not parent or method in ['GET','PROPFIND']
 base=directory if parent else testdir;url=base+urllib.parse.quote(name,safe='-._~');headers={'Authorization':auth,'Cache-Control':'no-cache','Accept-Encoding':'identity'}
 if method=='PROPFIND':headers.update({'Depth':'1','Content-Type':'application/xml'});body=b'<d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/></d:prop></d:propfind>'
 start=time.monotonic();row={'method':method,'target':'production-directory-read' if parent else name or 'owned-test-collection','requestBytes':len(body or b'')}
 try:
  with opener.open(urllib.request.Request(url,headers=headers,data=body,method=method),timeout=15) as r:
   b=r.read(2*1024*1024+1);assert len(b)<=2*1024*1024;row.update({'status':r.status,'elapsedMs':round((time.monotonic()-start)*1000,2),'responseBytes':len(b)});return r.status,b
 except urllib.error.HTTPError as e:row['status']=e.code;return e.code,b''
 except Exception as e:row['failure']=type(e).__name__;raise Fault(type(e).__name__) from None
 finally:trace.append(row)
def listing(parent=False):
 status,b=request('PROPFIND',parent=parent)
 if status!=207:raise Fault('list-'+str(status))
 names=set()
 for el in ET.fromstring(b).iter('{DAV:}href'):
  path=urllib.parse.unquote(urllib.parse.urlsplit(el.text or '').path).rstrip('/');names.add(path.rsplit('/',1)[-1])
 return names
def get(n):
 status,b=request('GET',n)
 if status!=200:raise Fault('get-'+str(status))
 return b
def put(n,b):
 assert n.startswith(('commit-','data-','generation-')) and '/' not in n
 owned.add(n);status,_=request('PUT',n,b)
 if status not in [200,201,204]:raise Fault('put-'+str(status))
 if get(n)!=b:raise Fault('put-content-mismatch')
def generation():
 ids=sorted(n for n in listing() if n.startswith('generation-'))
 if len(ids)!=1:raise Fault('generation-unknown-or-conflict')
 return json.loads(get(ids[0]))['id']
def view():
 gen=generation();names=listing();commits={}
 for n in names:
  if not n.startswith('commit-'):continue
  c=json.loads(get(n));assert c['generation']==gen and n=='commit-'+c['id']+'.json';assert sha(get('data-'+c['id']+'.bin'))==c['digest'];commits[c['id']]=c
 for c in commits.values():
  if any(x not in commits for x in c['parents']):raise Fault('missing-parent')
 parents={x for c in commits.values() for x in c['parents']};return commits,set(commits)-parents
def plan(body,parents=None):
 commits,heads=view()
 return {'id':str(uuid.uuid4()),'generation':generation(),'parents':sorted(heads if parents is None else parents),'digest':sha(body),'body':body}
def send(p):
 if generation()!=p['generation']:raise Fault('old-generation-plan')
 put('data-'+p['id']+'.bin',p['body']);put('commit-'+p['id']+'.json',enc({k:v for k,v in p.items() if k!='body'}));return p
try:
 anchor=Path('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop');root=Path(local(anchor/'manager-state/data-root.json')['active_data_root']);ep=local(root/'manager-state/sync-endpoints.json')['endpoints'][0];ref=ep['credential_ref'];account=ref['account_key'];assert account=='ocx.dav.'+ref['ref_id'];cred=local(root/'manager-state/credentials'/(account+'.json'));assert cred['account']==account and cred['protection']=='plain'
 u=urllib.parse.urlsplit(ep['url']);assert u.scheme=='https' and not u.username and not u.query and not u.fragment;remote=ep['remote_path'].strip('/');assert all(x not in ('','..','.') for x in remote.split('/'));directory=ep['url'].rstrip('/')+'/'+urllib.parse.quote(remote,safe='/-._~')+'/';testdir=directory+child+'/'
 auth='Basic '+base64.b64encode((ep['username'].strip()+':'+cred['value']).encode()).decode();cred=None
 production_before=listing(True);pointer_status,pointer_before=request('GET','config-v1-latest.json',parent=True);assert child not in production_before
 status,_=request('MKCOL');assert status==201;created=True
 gen=str(uuid.uuid4());other_gen=str(uuid.uuid4())
 with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:list(ex.map(lambda g:put('generation-'+g+'.json',enc({'id':g})),[gen,other_gen]))
 try:generation();raise AssertionError('ambiguous generation accepted')
 except Fault as e:assert str(e)=='generation-unknown-or-conflict'
 checks.append('simultaneous directory generation initialization fails closed without content upload')
 status,_=request('DELETE','generation-'+other_gen+'.json');assert status in [200,204];owned.remove('generation-'+other_gen+'.json')
 assert generation()==gen;checks.append('unique isolated directory and generation init')
 a=plan(b'synthetic first A');b=plan(b'synthetic first B')
 with concurrent.futures.ThreadPoolExecutor(max_workers=2) as ex:list(ex.map(send,[a,b]))
 commits,heads=view();assert heads=={a['id'],b['id']};checks.append('two simultaneous first uploads retain both branches')
 r=send(plan(a['body'],heads));assert view()[1]=={r['id']};checks.append('explicit conflict choice references both parents and keeps original files')
 plans=[plan(('synthetic writer '+str(i)).encode(),[r['id']]) for i in range(4)]
 with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:list(ex.map(send,plans))
 commits,heads=view();assert heads=={p['id'] for p in plans};checks.append('four simultaneous updates retain all heads')
 assert all(get('data-'+p['id']+'.bin')==p['body'] for p in plans);checks.append('direct content verification independent of ETag')
 before_puts=sum(x['method']=='PUT' for x in trace);one=plans[0];assert json.loads(get('commit-'+one['id']+'.json'))['digest']==sha(get('data-'+one['id']+'.bin'));assert sum(x['method']=='PUT' for x in trace)==before_puts;checks.append('same operation read-only recovery does not reupload')
 local_new=b'synthetic local unsaved changes';assert get('data-'+one['id']+'.bin')==one['body'] and sha(local_new)!=one['digest'];checks.append('original confirmation independent of newer local content')
 names=listing();assert all('commit-'+p['id']+'.json' in names for p in [a,b,r,*plans]);checks.append('new file immediate listing complete for observed commits')
 frozen=plan(b'synthetic frozen old generation');old_gen=gen
 # Clear only owned test files, then initialize another generation; no production file touched.
 for n in list(owned):
  status,_=request('DELETE',n);assert status in [200,204,404];owned.remove(n)
 gen=str(uuid.uuid4());put('generation-'+gen+'.json',enc({'id':gen}));assert generation()!=old_gen
 try:send(frozen);raise AssertionError('old generation accepted')
 except Fault as e:assert str(e)=='old-generation-plan'
 checks.append('clear and reinitialize rejects old frozen upload before writes')
 orphan=plan(b'synthetic orphan');put('data-'+orphan['id']+'.bin',orphan['body']);assert not view()[0];checks.append('orphan payload is not active version')
 result={'status':'passed','checks':checks}
except Exception as e:result={'status':'failed','errorClass':type(e).__name__,'message':str(e) if isinstance(e,(Fault,AssertionError)) else 'redacted','checks':checks}
finally:
 if created:
  cleanup['attempted']=True;fail=[]
  for n in list(owned):
   try:
    status,_=request('DELETE',n)
    if status not in [200,204,404]:fail.append(n)
   except Exception:fail.append(n)
  try:
   status,_=request('DELETE');cleanup['collectionDeleteStatus']=status
  except Exception:cleanup['collectionDeleteStatus']='request-failed'
  try:
   after=listing(True);cleanup['testCollectionAbsent']=child not in after;cleanup['productionNamesUnchanged']=after==production_before
   end_status,end_body=request('GET','config-v1-latest.json',parent=True);cleanup['productionLatestBytesUnchanged']=end_status==pointer_status and end_body==pointer_before
  except Exception:cleanup['postCheckFailure']=True
  cleanup['individualDeleteFailures']=fail
 cleanup['localInputBytesUnchanged']=all(hashlib.sha256(p.read_bytes()).hexdigest()==h for p,h in inputs.items())
 report={'runId':run,'testDirectoryBasename':child,'timeUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'candidateProtocolOnly':True,'result':result,'cleanup':cleanup,'noETagConditions':True,'noSecretsOrRawPayloadSaved':True,'limits':['Synthetic protocol model uses real server transport; not application new implementation','Observed visibility and concurrent preservation are finite tests, not proof under all network/cache schedules','Directory generation bootstrap races must remain explicit conflict/fail-closed until a concrete initialization protocol is implemented','No production sync/receipt/config or daily service modifications'],'requests':trace};(ROOT/'live-webdav-result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+chr(10));print(json.dumps({k:v for k,v in report.items() if k!='requests'},ensure_ascii=False,indent=2))
assert result['status']=='passed' and cleanup.get('testCollectionAbsent') and cleanup.get('productionNamesUnchanged') and cleanup.get('productionLatestBytesUnchanged') and not cleanup.get('individualDeleteFailures')
