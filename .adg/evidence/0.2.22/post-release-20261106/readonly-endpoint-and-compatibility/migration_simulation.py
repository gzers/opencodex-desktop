"""Design simulation only. Loopback mock WebDAV, no daily config or external host.
Candidate: immutable per-operation commits with parent revisions; PROPFIND discovery.
No ETag/conditional header is used. This is not the current product implementation.
"""
from pathlib import Path
import concurrent.futures,hashlib,http.client,json,socket,threading,unittest,uuid,xml.etree.ElementTree as ET
from http.server import BaseHTTPRequestHandler,ThreadingHTTPServer
from urllib.parse import urlsplit
ROOT=Path(__file__).parent
sha=lambda b:hashlib.sha256(b).hexdigest()
encode=lambda v:json.dumps(v,sort_keys=True,separators=(',',':')).encode()
class Storage:
 def __init__(self):self.lock=threading.RLock();self.files={};self.hidden=set();self.fail={};self.lose=set();self.mode='missing';self.serial=0;self.log=[];self.list_supported=True
state=Storage()
class Handler(BaseHTTPRequestHandler):
 protocol_version='HTTP/1.1'
 def log_message(self,*args):pass
 def handle_request(self):
  method=self.command;path=urlsplit(self.path).path;n=int(self.headers.get('Content-Length','0'));body=self.rfile.read(n) if n else b''
  with state.lock:
   state.serial+=1;seq=state.serial;status=200;output=b''
   assert 'If-Match' not in self.headers and 'If-None-Match' not in self.headers,'ETag conditional request forbidden in simulation'
   if path in state.fail:status=state.fail[path]
   elif method=='PUT':state.files[path]=body;status=201
   elif method=='GET':
    if path not in state.files:status=404
    else:output=state.files[path]
   elif method=='PROPFIND':
    if not state.list_supported:status=405
    else:
     status=207;ns='{DAV:}';tree=ET.Element(ns+'multistatus')
     for name in sorted(state.files):
      if name.startswith(path) and name not in state.hidden:ET.SubElement(ET.SubElement(tree,ns+'response'),ns+'href').text=name
     output=ET.tostring(tree)
   else:status=405
   state.log.append({'n':seq,'method':method,'path':path,'status':status,'bytes':len(body),'digest':sha(body) if body else None})
   lose=method=='PUT' and path in state.lose
   if lose:state.lose.remove(path)
   mode=state.mode
  if lose:self.close_connection=True;self.connection.shutdown(socket.SHUT_RDWR);self.connection.close();return
  self.send_response(status)
  if mode=='weak':self.send_header('ETag','W/"unreliable"')
  elif mode=='changing':self.send_header('ETag','"'+str(seq)+'"')
  self.send_header('Cache-Control','no-store');self.send_header('Content-Length',str(len(output)));self.end_headers();self.wfile.write(output)
 do_GET=handle_request;do_PUT=handle_request;do_PROPFIND=handle_request
class Fault(Exception):pass
class Client:
 def __init__(self,namespace):self.prefix='/'+namespace+'/'
 def request(self,method,name,body=None):
  conn=http.client.HTTPConnection('127.0.0.1',port,timeout=4)
  try:
   conn.request(method,self.prefix+name,body=body,headers={'Depth':'1'} if method=='PROPFIND' else {})
   response=conn.getresponse();return response.status,response.read()
  finally:conn.close()
 def read(self,name,missing=False):
  status,b=self.request('GET',name)
  if status==404 and missing:return None
  if status!=200:raise Fault('read-status-'+str(status))
  return b
 def put_verified(self,name,b):
  old=self.read(name,True)
  if old is not None:
   if old!=b:raise Fault('immutable-operation-content-changed')
   return
  status,_=self.request('PUT',name,b)
  if status not in (200,201,204):raise Fault('put-status-'+str(status))
  if self.read(name)!=b:raise Fault('write-mismatch')
 def inventory(self):
  status,b=self.request('PROPFIND','')
  if status!=207:raise Fault('discovery-unavailable-'+str(status))
  try:names=[x.text for x in ET.fromstring(b).iter('{DAV:}href')]
  except Exception as e:raise Fault('invalid-directory') from e
  commits={}
  for path in names:
   if not path.startswith(self.prefix+'commit-') or not path.endswith('.json'):continue
   raw=self.read(path[len(self.prefix):])
   try:c=json.loads(raw);uuid.UUID(c['id']);assert c['format']=='SIM_COMMIT_V1';assert path==self.prefix+'commit-'+c['id']+'.json';assert c['parents']==sorted(set(c['parents']));assert isinstance(c['digest'],str) and len(c['digest'])==64
   except Exception as e:raise Fault('invalid-commit') from e
   b=self.read('data-'+c['id']+'.bin')
   if sha(b)!=c['digest']:raise Fault('payload-mismatch')
   commits[c['id']]=c
  for c in commits.values():
   if any(p not in commits for p in c['parents']):raise Fault('missing-parent')
  # Cycles must never choose a silent winner.
  def visit(key,chain):
   if key in chain:raise Fault('commit-cycle')
   for parent in commits[key]['parents']:visit(parent,chain|{key})
  for key in commits:visit(key,set())
  parents={p for c in commits.values() for p in c['parents']};heads=set(commits)-parents
  return commits,heads
 def prepare(self,b,parents=None):
  if parents is None:
   _,heads=self.inventory()
   if len(heads)>1:raise Fault('explicit-conflict-choice-required')
   parents=sorted(heads)
  return {'id':str(uuid.uuid4()),'format':'SIM_COMMIT_V1','parents':sorted(parents),'digest':sha(b),'payload':b}
 def send(self,plan):
  c={k:v for k,v in plan.items() if k!='payload'};self.put_verified('data-'+c['id']+'.bin',plan['payload']);self.put_verified('commit-'+c['id']+'.json',encode(c));return self.reconcile(plan)
 def reconcile(self,plan):
  c={k:v for k,v in plan.items() if k!='payload'};raw=self.read('commit-'+c['id']+'.json')
  if raw!=encode(c) or sha(self.read('data-'+c['id']+'.bin'))!=c['digest']:raise Fault('original-commit-mismatch')
  _,heads=self.inventory()
  return {'originalStored':True,'view':'conflict' if len(heads)>1 else 'current' if c['id'] in heads else 'superseded','heads':sorted(heads)}
 def download(self):
  commits,heads=self.inventory()
  if not heads:return None
  if len(heads)!=1:raise Fault('conflict')
  head=next(iter(heads));payload=self.read('data-'+head+'.bin');_,after=self.inventory()
  if after!=heads:raise Fault('changed-during-download')
  return payload
class Cases(unittest.TestCase):
 def setUp(self):self.c=Client(self._testMethodName)
 def test_naive_read_compare_write_loses_an_acknowledged_upload(self):
  a,b=self.c,self.c
  self.assertIsNone(a.read('latest.json',True));self.assertIsNone(b.read('latest.json',True))
  a.request('PUT','latest.json',b'A');self.assertEqual(a.read('latest.json'),b'A') # A sees success
  b.request('PUT','latest.json',b'B');self.assertEqual(b.read('latest.json'),b'B')
  self.assertNotIn(b'A',[state.files[a.prefix+'latest.json']])
 def test_first_upload_and_sequential_download_no_etag(self):
  self.assertIsNone(self.c.download());p=self.c.prepare(b'first');self.assertEqual(self.c.send(p)['view'],'current');self.assertEqual(self.c.download(),b'first');q=self.c.prepare(b'next');self.c.send(q);self.assertEqual(self.c.download(),b'next');self.assertEqual(self.c.reconcile(p)['view'],'superseded')
 def test_etag_missing_weak_or_changing_does_not_gate_content(self):
  for mode in ['missing','weak','changing']:
   state.mode=mode;c=Client(self._testMethodName+mode);p=c.prepare(mode.encode());self.assertTrue(c.send(p)['originalStored']);self.assertEqual(c.download(),mode.encode())
  state.mode='missing'
 def test_concurrent_first_upload_keeps_both_and_requires_choice(self):
  a=self.c.prepare(b'A');b=self.c.prepare(b'B');self.c.send(a);self.assertEqual(self.c.send(b)['view'],'conflict');self.assertEqual(self.c.reconcile(a)['view'],'conflict');self.assertRaises(Fault,self.c.download)
  commits,heads=self.c.inventory();self.assertEqual(len(commits),2);self.assertEqual(len(heads),2)
  selected=self.c.prepare(b'A',parents=heads);self.c.send(selected);self.assertEqual(self.c.download(),b'A');self.assertEqual(len(self.c.inventory()[0]),3)
 def test_concurrent_updates_and_stress_preserve_all_operation_files(self):
  first=self.c.prepare(b'base');self.c.send(first);plans=[self.c.prepare(('writer'+str(i)).encode(),[first['id']]) for i in range(20)]
  with concurrent.futures.ThreadPoolExecutor(max_workers=4) as ex:list(ex.map(self.c.send,plans))
  commits,heads=self.c.inventory();self.assertEqual(len(commits),21);self.assertEqual(len(heads),20)
  for p in plans:self.assertEqual(self.c.read('data-'+p['id']+'.bin'),p['payload'])
 def test_lost_commit_reply_reconciles_same_files_without_reupload(self):
  p=self.c.prepare(b'original');state.lose.add(self.c.prefix+'commit-'+p['id']+'.json')
  with self.assertRaises((http.client.RemoteDisconnected,ConnectionError,OSError)):self.c.send(p)
  before=len([r for r in state.log if r['method']=='PUT' and r['path'].startswith(self.c.prefix)])
  self.assertTrue(self.c.reconcile(p)['originalStored']);self.assertEqual(before,len([r for r in state.log if r['method']=='PUT' and r['path'].startswith(self.c.prefix)]))
 def test_orphan_payload_is_not_an_effective_configuration(self):
  p=self.c.prepare(b'orphan');self.c.put_verified('data-'+p['id']+'.bin',p['payload']);self.assertIsNone(self.c.download());self.assertRaises(Fault,self.c.reconcile,p)
 def test_invalid_or_unreadable_data_never_means_empty(self):
  p=self.c.prepare(b'valid');self.c.send(p);name=self.c.prefix+'data-'+p['id']+'.bin';state.files[name]=b'corrupt';self.assertRaises(Fault,self.c.download);state.files[name]=p['payload']
  for code in [401,403,500]:
   state.fail[name]=code;self.assertRaises(Fault,self.c.download)
  state.fail.pop(name);self.assertEqual(self.c.download(),b'valid')
 def test_delayed_listing_detects_later_conflict_without_erasing_commit(self):
  a=self.c.prepare(b'A');b=self.c.prepare(b'B');self.c.send(a);self.c.send(b);name=self.c.prefix+'commit-'+b['id']+'.json';state.hidden.add(name)
  self.assertEqual(self.c.download(),b'A') # Observation cannot prove there is no invisible commit.
  state.hidden.remove(name);self.assertRaises(Fault,self.c.download);self.assertEqual(len(self.c.inventory()[0]),2)
 def test_unsupported_discovery_and_missing_parent_fail_closed(self):
  state.list_supported=False
  try:self.assertRaises(Fault,self.c.prepare,b'anything')
  finally:state.list_supported=True
  p=self.c.prepare(b'dangling',[str(uuid.uuid4())]);self.assertRaises(Fault,self.c.send,p)
 def test_same_id_with_changed_payload_rejected_and_original_kept(self):
  p=self.c.prepare(b'old');self.c.send(p);bad=dict(p,payload=b'new',digest=sha(b'new'));self.assertRaises(Fault,self.c.send,bad);self.assertEqual(self.c.download(),b'old')
 def test_local_edits_do_not_block_original_readonly_confirmation(self):
  p=self.c.prepare(b'original');self.c.send(p);local=b'new-local-draft';before=len(state.log);r=self.c.reconcile(p);self.assertTrue(r['originalStored']);self.assertEqual(local,b'new-local-draft');self.assertFalse(any(x['method']=='PUT' for x in state.log[before:]));self.assertNotEqual(sha(local),p['digest'])
 def test_legacy_pointer_does_not_merge_into_candidate_log_implicitly(self):
  self.c.request('PUT','latest.json',b'legacy-generation');self.assertIsNone(self.c.download());p=self.c.prepare(b'candidate');self.c.send(p);self.c.request('PUT','latest.json',b'legacy-overwrite');self.assertEqual(self.c.download(),b'candidate')
class MockServer(ThreadingHTTPServer):
 request_queue_size=128

class Migration:
 """Candidate only: keep v1 files unchanged; v2 immutable files share folder but distinct prefix.
 There is no way to remotely prevent an unchanged legacy client writing v1 with same credentials.
 Detect legacy drift and require explicit import/conflict resolution. No automatic cutover.
 """
 def __init__(self,c):self.c=c
 def v1(self):
  raw=self.c.read('config-v1-latest.json',True)
  if raw is None:return None,None
  p=json.loads(raw);assert p['format']=='OCXD_CONFIG_POINTER';uuid.UUID(p['operationId']);b=self.c.read('config-v1-'+p['digest']+'.ocxd')
  if sha(b)!=p['digest']:raise Fault('legacy-payload-corrupt')
  return p,b
 def old_upload(self,b):
  p={'format':'OCXD_CONFIG_POINTER','version':1,'operationId':str(uuid.uuid4()),'digest':sha(b)};self.c.put_verified('config-v1-'+p['digest']+'.ocxd',b);self.c.request('PUT','config-v1-latest.json',encode(p));return p
 def readv2(self):
  code,body=self.c.request('PROPFIND','')
  if code!=207:raise Fault('listing-unavailable')
  names=[x.text for x in ET.fromstring(body).iter('{DAV:}href')];commits={}
  for name in names:
   if not name.startswith(self.c.prefix+'config-v2-commit-'):continue
   c=json.loads(self.c.read(name[len(self.c.prefix):]));uuid.UUID(c['id']);assert name==self.c.prefix+'config-v2-commit-'+c['id']+'.json'
   payload=self.c.read('config-v2-data-'+c['id']+'.bin')
   if sha(payload)!=c['digest']:raise Fault('candidate-corrupt')
   commits[c['id']]=c
  if any(parent not in commits for c in commits.values() for parent in c['parents']):raise Fault('missing-parent')
  heads=set(commits)-{p for c in commits.values() for p in c['parents']}
  baselines={encode(commits[h]['legacyBaseline']) for h in heads};current,_=self.v1()
  drift=any(b!=encode(current) for b in baselines)
  return commits,heads,drift
 def bootstrap(self):
  commits,heads,_=self.readv2()
  if commits:raise Fault('already-migrated-review-existing')
  legacy,payload=self.v1()
  if legacy is None:raise Fault('legacy-empty-explicit-new-domain-required')
  return self.write(payload,[],legacy,'explicit-bootstrap')
 def write(self,payload,parents,legacy,reason):
  c={'id':str(uuid.uuid4()),'parents':sorted(parents),'digest':sha(payload),'legacyBaseline':legacy,'reason':reason}
  self.c.put_verified('config-v2-data-'+c['id']+'.bin',payload);self.c.put_verified('config-v2-commit-'+c['id']+'.json',encode(c));return c
 def ordinary(self,b):
  commits,heads,drift=self.readv2()
  if drift:raise Fault('legacy-client-drift-needs-review')
  if len(heads)!=1:raise Fault('candidate-conflict')
  return self.write(b,heads,commits[next(iter(heads))]['legacyBaseline'],'normal-v2')
 def import_legacy_branch(self):
  legacy,payload=self.v1();assert legacy is not None
  return self.write(payload,[],legacy,'explicit-legacy-import-conflict')
 def resolve(self,b):
  commits,heads,_=self.readv2();legacy,_=self.v1();return self.write(b,heads,legacy,'explicit-user-choice')
class MigrationCases(unittest.TestCase):
 def setUp(self):self.c=Client(self._testMethodName);self.m=Migration(self.c)
 def test_bootstrap_reads_legacy_preserves_v1_and_does_not_write_legacy_pointer(self):
  original=self.m.old_upload(b'legacy');before={k:v for k,v in state.files.items() if k.startswith(self.c.prefix)};n=len(state.log);self.m.bootstrap();self.assertEqual(self.m.v1()[0],original)
  self.assertFalse(any(r['method']=='PUT' and '/config-v1-' in r['path'] for r in state.log[n:]));self.assertTrue(all(state.files[k]==v for k,v in before.items()))
 def test_legacy_client_continues_to_read_legacy_and_cannot_silently_consume_new_protocol(self):
  self.m.old_upload(b'legacy');self.m.bootstrap();self.m.ordinary(b'new');self.assertEqual(self.m.v1()[1],b'legacy')
 def test_old_client_write_after_cutover_is_detected_and_new_sync_blocks(self):
  self.m.old_upload(b'legacy');self.m.bootstrap();new=self.m.ordinary(b'new');self.m.old_upload(b'old-client-write');self.assertTrue(self.m.readv2()[2]);self.assertRaisesRegex(Fault,'legacy-client-drift',self.m.ordinary,b'silent-override');self.assertEqual(self.c.read('config-v2-data-'+new['id']+'.bin'),b'new')
 def test_explicit_old_change_import_creates_conflict_then_choice_preserves_both(self):
  self.m.old_upload(b'legacy');self.m.bootstrap();new=self.m.ordinary(b'new');self.m.old_upload(b'old-advance');branch=self.m.import_legacy_branch();self.assertEqual(len(self.m.readv2()[1]),2);self.assertRaises(Fault,self.m.ordinary,b'no');chosen=self.m.resolve(b'old-advance');self.assertFalse(self.m.readv2()[2]);self.assertEqual(len(self.m.readv2()[1]),1);self.assertEqual(self.c.read('config-v2-data-'+new['id']+'.bin'),b'new');self.assertEqual(self.c.read('config-v2-data-'+branch['id']+'.bin'),b'old-advance');self.assertEqual(len(chosen['parents']),2)
 def test_two_devices_bootstrapping_same_legacy_do_not_overwrite_each_other(self):
  old=self.m.old_upload(b'legacy');a=self.m.write(b'legacy',[],old,'bootstrap-A');b=self.m.write(b'legacy',[],old,'bootstrap-B');commits,heads,drift=self.m.readv2();self.assertEqual(heads,{a['id'],b['id']});self.assertFalse(drift);self.assertRaises(Fault,self.m.ordinary,b'needs-choice')
 def test_old_write_between_candidate_check_and_commit_is_detected_after_commit(self):
  old=self.m.old_upload(b'legacy');self.m.bootstrap();_,heads,drift=self.m.readv2();self.assertFalse(drift);self.m.old_upload(b'concurrent-old');new=self.m.write(b'new',heads,old,'race');self.assertTrue(self.m.readv2()[2]);self.assertEqual(self.c.read('config-v2-data-'+new['id']+'.bin'),b'new');self.assertEqual(self.m.v1()[1],b'concurrent-old')
 def test_unreadable_or_corrupt_legacy_is_not_treated_as_empty(self):
  self.m.old_upload(b'legacy');self.m.bootstrap();name=self.c.prefix+'config-v1-latest.json';state.fail[name]=403
  try:self.assertRaises(Fault,self.m.ordinary,b'no')
  finally:state.fail.pop(name)
  state.files[name]=b'invalid-json';self.assertRaises((ValueError,Fault),self.m.readv2)
 def test_candidate_reconciliation_reads_existing_operation_without_republishing(self):
  old=self.m.old_upload(b'legacy');self.m.bootstrap();new=self.m.ordinary(b'new');before=len(state.log);commits,heads,drift=self.m.readv2();self.assertIn(new['id'],heads);self.assertFalse(drift);self.assertFalse(any(r['method']=='PUT' for r in state.log[before:]));self.assertIn(new['id'],commits)
server=MockServer(('127.0.0.1',0),Handler);server.daemon_threads=False;port=server.server_address[1];thread=threading.Thread(target=server.serve_forever,name='post022-migration-mock');thread.start();result=None
try:
 with (ROOT/'migration-tests.txt').open('w') as out:result=unittest.TextTestRunner(stream=out,verbosity=2).run(unittest.defaultTestLoader.loadTestsFromTestCase(MigrationCases))
finally:
 server.shutdown();server.server_close();thread.join(5);probe=socket.socket();probe.settimeout(1);closed=probe.connect_ex(('127.0.0.1',port))!=0;probe.close()
 summary={'tests':result.testsRun if result else 0,'errors':len(result.errors) if result else None,'failures':len(result.failures) if result else None,'threadStopped':not thread.is_alive(),'portClosed':closed,'port':port,'candidateOnly':True,'noETagConditions':True,'requests':len(state.log),'limits':['V2 protocol not implemented or adopted','Old clients cannot readV2 and cannot be remotely blocked with same credentials; upgrade-all or explicit separate generation policy required','V1 latest drift is detectable on next new-client read; not an atomic old-client freeze','Only modeled coexistence, no remote write/migration occurred']}
 (ROOT/'migration-result.json').write_text(json.dumps(summary,indent=2)+chr(10));(ROOT/'migration-request-trace.json').write_text(json.dumps(state.log,indent=2)+chr(10));state.files.clear()
print((ROOT/'migration-tests.txt').read_text());print(json.dumps(summary,indent=2));assert closed and not thread.is_alive();raise SystemExit(0 if result and result.wasSuccessful() else 1)
