from pathlib import Path
import json,hashlib,hmac,base64,secrets,time,urllib.request,shutil,os
r=Path(__file__).parent;home=Path('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/opencodex-home');record=home/'runtime-port.json';recordBytes=record.read_bytes();d=json.loads(recordBytes)
nonce=base64.urlsafe_b64encode(secrets.token_bytes(32)).decode().rstrip('=');expires=int(time.time()*1000)+8000;path='/api/config'
payload='opencodex-local-management-read-v1\n'+nonce+'\nGET\n'+path+'\n'+str(d['pid'])+'\n'+str(d['port'])+'\n'+str(expires)
cap=base64.urlsafe_b64encode(hmac.new(d['attestationSecret'].encode(),payload.encode(),hashlib.sha256).digest()).decode().rstrip('=')
headers={'x-opencodex-local-expected-pid':str(d['pid']),'x-opencodex-local-nonce':nonce,'x-opencodex-local-expires-at':str(expires),'x-opencodex-local-capability':cap}
op=urllib.request.build_opener(urllib.request.ProxyHandler({}));configBytes=(home/'config.json').read_bytes()
with op.open(urllib.request.Request('http://127.0.0.1:'+str(d['port'])+path,headers=headers),timeout=5) as response:
 b=response.read(2*1024*1024);status=response.status;proof=response.headers.get('x-opencodex-attestation-proof')
expected=base64.urlsafe_b64encode(hmac.new(d['attestationSecret'].encode(),('opencodex-local-management-v1\n'+nonce+'\n'+str(d['pid'])+'\n'+str(d['port'])).encode(),hashlib.sha256).digest()).decode().rstrip('=')
assert hmac.compare_digest(expected,proof or '') and record.read_bytes()==recordBytes and (home/'config.json').read_bytes()==configBytes
v=json.loads(b);assert isinstance(v.get('providers'),dict)
fixture=r/'private-fixture';fixture.mkdir(mode=0o700)
for sub in ['user/home','user/.codex','tmp','cache']:(fixture/sub).mkdir(parents=True,exist_ok=True)
for name,data in [('download.json',Path('/Users/ezio/Downloads/config.json').read_bytes()),('user/home/config.json',configBytes),('live-public.json',b)]:
 p=fixture/name;p.write_bytes(data);os.chmod(p,0o600)
s=Path('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/@bitkyc08/opencodex');shutil.copytree(s/'src',fixture/'package/src');shutil.copyfile(s/'package.json',fixture/'package/package.json');(fixture/'package/node_modules').symlink_to(s.parent.parent,target_is_directory=True)
(fixture/'sandbox.sb').write_text('(version 1)\n(allow default)\n(deny network*)\n(deny file-write*)\n(allow file-write* (subpath "'+str(fixture)+'"))\n'+''.join('(deny file-read* (subpath "'+p+'"))\n' for p in ['/Users/ezio/.codex','/Users/ezio/.opencodex','/Users/ezio/.codex-desktop','/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/opencodex-home','/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/manager-state','/Users/ezio/Downloads']))
report={'scope':'Actual authenticated one-use local GET /api/config; no reusable admin token; raw public body remains only private temporary input','pid':d['pid'],'port':d['port'],'httpStatus':status,'responseHmacVerified':True,'recordUnchanged':True,'configBytesUnchangedDuringRead':True,'livePublicProviderCount':len(v['providers']),'currentDiskSha256':hashlib.sha256(configBytes).hexdigest(),'rawSecretsPrintedOrArchived':False}
(r/'live-read-result.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
