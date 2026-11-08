from pathlib import Path
import json,http.client,secrets,base64,hmac,hashlib,time,subprocess
root=Path('/private/var/folders/fq/l0j4lwrs5ld6gpfx2pppv2640000gn/T/ocxd023-real-port-lifecycle-WYga1K');h=root/'profile/.codex-desktop/opencodex/home';r=json.loads((h/'runtime-port.json').read_text());c=http.client.HTTPConnection('127.0.0.1',r['port'],timeout=10)
challenge=base64.urlsafe_b64encode(secrets.token_bytes(32)).decode().rstrip('=');c.request('GET','/healthz',headers={'x-opencodex-attestation-challenge':challenge});response=c.getresponse();body=json.loads(response.read());expected=base64.urlsafe_b64encode(hmac.new(r['attestationSecret'].encode(),('opencodex-local-management-v1\n'+challenge+'\n'+str(r['pid'])+'\n'+str(r['port'])).encode(),hashlib.sha256).digest()).decode().rstrip('=');assert hmac.compare_digest(response.getheader('x-opencodex-attestation-proof'),expected);assert body['pid']==r['pid']
c.request('POST','/api/stop',body=b'',headers={'Authorization':'Bearer '+(h/'admin-api-token').read_text().strip()});response=c.getresponse();result=json.loads(response.read());c.close();assert result['success'] and result['sharedTeardown']=='performed'
for _ in range(50):
 if not(h/'runtime-port.json').exists():break
 time.sleep(.1)
processes=[]
for line in subprocess.check_output(['ps','-axo','pid=,ppid=,comm=']).decode().splitlines():
 if str(root) in line:processes.append(line.strip())
assert not processes,processes
assert not(h/'runtime-port.json').exists() and not(h/'ocx.pid').exists()
out=Path('/private/tmp/ocxd023-native-start-failure-r1');(out/'cleanup-final.json').write_text(json.dumps({'officialStopSuccess':True,'sharedTeardown':'performed','recordAbsent':True,'pidRecordAbsent':True,'ownedProcessImagesRemaining':processes,'failure':'launchd has job loaded but isolated HOME plist absent; ownership refused without touching daily service'},indent=2))
print('owned service restored and stopped; native process images and records absent')
# The source bundle is shared and retained; only this exact failed fixture is removed.
assert root.name=='ocxd023-real-port-lifecycle-WYga1K' and root.parent.name=='T'
import shutil
shutil.rmtree(root)
print('exact task fixture removed; failure evidence retained')
