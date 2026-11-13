from pathlib import Path
import os,json,subprocess,time,hashlib,http.client,base64,hmac,uuid,shutil
b=Path('/private/tmp/ocxd024-versioned-lifecycle-ihwdax9o');old=b/'fixtures';root=b/'foreground-fixture';root.mkdir();out=b/'evidence/foreground-diagnostic';out.mkdir()
profile=root/'profile';h=profile/'.codex-desktop/opencodex/home';client=profile/'.codex';h.mkdir(parents=True,mode=0o700);client.mkdir(mode=0o700)
config={'hostname':'127.0.0.1','port':0,'openaiProviderTierVersion':2,'defaultProvider':'native-fixture','defaultModelAliases':False,'providers':{'native-fixture':{'adapter':'openai-chat','baseUrl':'http://127.0.0.1:1/v1','allowPrivateNetwork':True,'apiKey':'synthetic-unused','models':['native-fixture-model'],'selectedModels':['native-fixture-model'],'liveModels':False,'defaultAliases':False}}}
(h/'config.json').write_text(json.dumps(config));(h/'config.json').chmod(0o600)
(client/'config.toml').write_text('model = "gpt-5.4"\nmodel_provider = "openai"\n')
(client/'opencodex-catalog.json').write_text(json.dumps({'models':[{'slug':'gpt-5.4','display_name':'Native fixture model','description':'synthetic native record','visibility':'list','supported_in_api':True,'priority':0}]}))
coord=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()))/'catalog-write-locks'/hashlib.sha256(str(client.resolve()).encode()).hexdigest()
locks=[Path(str(coord)+s) for s in ['.sqlite','.sqlite-journal','.sqlite-wal','.sqlite-shm']];assert not any(p.exists() for p in locks)
sb=root/'fixture.sb';policy=(b/'native.sb').read_text().replace(str(old),str(root))
oldlocks=json.loads((b/'evidence/native-runner-summary.json').read_text())['locks']
for a,c in zip(oldlocks,locks):policy=policy.replace(a,str(c))
sb.write_text(policy);(out/'fixture.sb').write_text(policy)
report={'accepted':False,'hypothesis':'Capture foreground official start output to distinguish failure of own-H write/OS sandbox from ensure plus detached-start concurrent synchronization. Same unmodified official image and same synthetic client input, no real requests.','normalStop':False,'locks':[str(p) for p in locks]};(out/'hypothesis.json').write_text(json.dumps(report,indent=2))
program=old/'program 中文 space/runtime/versions/2.99.0';node=program/'dependencies/node/bin/node';entry=program/'opencodex/node_modules/@bitkyc08/opencodex/bin/ocx.mjs'
env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','HOME':str(profile),'OPENCODEX_HOME':str(h),'CODEX_HOME':str(client),'TMPDIR':str(root)+'/', 'LANG':'en_US.UTF-8'}
start=time.monotonic()
with (out/'stdout.log').open('wb') as stdout,(out/'stderr.log').open('wb') as stderr:
 p=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sb),str(node),str(entry),'start'],env=env,cwd=root,stdout=stdout,stderr=stderr,start_new_session=True)
 report['launcherPid']=p.pid
 try:
  deadline=time.monotonic()+30;record=None
  while time.monotonic()<deadline and p.poll() is None:
   try:
    record=json.loads((h/'runtime-port.json').read_bytes());c=http.client.HTTPConnection('127.0.0.1',record['port'],timeout=2);c.request('GET','/readyz');r=c.getresponse();v=json.loads(r.read(65536));c.close()
    report['readiness']=v
    if v.get('status') in ['ready','failed']:break
   except (OSError,ValueError,http.client.HTTPException):pass
   time.sleep(.1)
  report['startElapsedMs']=round((time.monotonic()-start)*1000)
  if record:
   report['pid']=record['pid'];report['port']=record['port'];c=http.client.HTTPConnection('127.0.0.1',record['port'],timeout=4)
   for _ in range(2):
    challenge=base64.urlsafe_b64encode(uuid.uuid4().bytes+uuid.uuid4().bytes).rstrip(b'=').decode();c.request('GET','/healthz',headers={'x-opencodex-attestation-challenge':challenge});r=c.getresponse();v=json.loads(r.read(65536));proof=r.getheader('x-opencodex-attestation-proof');expected=hmac.new(record['attestationSecret'].encode(),f"opencodex-local-management-v1\n{challenge}\n{record['pid']}\n{record['port']}".encode(),hashlib.sha256).digest();assert r.status==200 and v['pid']==record['pid'] and v['port']==record['port'] and hmac.compare_digest(base64.urlsafe_b64decode(proof+'='*(-len(proof)%4)),expected)
   token=(h/'admin-api-token').read_text().strip();c.request('POST','/api/stop',body=b'',headers={'Authorization':'Bearer '+token});r=c.getresponse();v=json.loads(r.read(65536));report['stopReply']={'code':r.status,'body':v};c.close()
   report['normalStop']=r.status==200 and v.get('success')==True
   try:report['exitCode']=p.wait(timeout=10)
   except subprocess.TimeoutExpired:report['exitTimeout']=True
 except BaseException as error:report['error']=repr(error)
report['processExited']=p.poll() is not None
(out/'summary.json').write_text(json.dumps(report,ensure_ascii=False,indent=2));shutil.copy2(Path(__file__),out/'runner.py')
print(json.dumps(report,ensure_ascii=False,indent=2),flush=True)
