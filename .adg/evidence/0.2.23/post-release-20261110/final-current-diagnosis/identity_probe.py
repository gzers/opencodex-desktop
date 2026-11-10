from pathlib import Path
import json,urllib.request,hmac,hashlib,secrets,base64,subprocess,datetime
from zoneinfo import ZoneInfo
home=Path('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/opencodex-home');record=home/'runtime-port.json';before=record.read_bytes();d=json.loads(before)
assert d['hostname']=='127.0.0.1' and isinstance(d['pid'],int) and 1<=d['port']<=65535
challenge=base64.urlsafe_b64encode(secrets.token_bytes(32)).decode().rstrip('=')
op=urllib.request.build_opener(urllib.request.ProxyHandler({}))
with op.open(urllib.request.Request('http://127.0.0.1:'+str(d['port'])+'/healthz',headers={'x-opencodex-attestation-challenge':challenge}),timeout=5) as response:
 raw=response.read(32768);health=json.loads(raw);proof=response.headers.get('x-opencodex-attestation-proof');code=response.status
expected=base64.urlsafe_b64encode(hmac.new(d['attestationSecret'].encode(),('opencodex-local-management-v1\n'+challenge+'\n'+str(d['pid'])+'\n'+str(d['port'])).encode(),hashlib.sha256).digest()).decode().rstrip('=')
assert hmac.compare_digest(expected,proof or '') and health['pid']==d['pid'] and health['port']==d['port'] and health['service']=='opencodex'
assert record.read_bytes()==before
identityFile=home/'quota-pool-identity.json';identity=json.loads(identityFile.read_bytes());assert health['instanceId']==identity['homeId']
comm=subprocess.check_output(['ps','-p',str(d['pid']),'-o','comm='],text=True).strip()
app=Path('/Applications/OpenCodeX Desktop.app/Contents/Info.plist');import plistlib
report={'recordedAt':datetime.datetime.now(ZoneInfo('Asia/Shanghai')).isoformat(),'method':'exact local GET /healthz HMAC challenge; no admin token/model/sync requests','httpStatus':code,'response':{k:health.get(k) for k in ['status','service','version','pid','port','restartCapability','providerReloadCapability']},'hmacProofVerified':True,'recordUnchangedDuringProbe':True,'homeInstanceMatchesOnDiskIdentity':True,'home':str(home),'configPath':str(home/'config.json'),'processImage':comm,'desktopDiskVersion':plistlib.loads(app.read_bytes()).get('CFBundleShortVersionString'),'desktopProcess':59923,'limits':'Service identity and HOME verified by HMAC+homeInstance ID, read-only records and process image. No original historical failing window attribution.'}
Path(__file__).with_name('identity-result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n');print(json.dumps(report,ensure_ascii=False,indent=2))
