from pathlib import Path
import urllib.request,urllib.error,subprocess,os,uuid,concurrent.futures,hashlib,json
root=Path('/private/tmp/ocxd023-r27-native-preflight');size=8072;expected='b6f0dd38f197e419e7282a705f70f36a62cb980364a38286745f5e62149aed91'
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs):return None
proxy=urllib.request.ProxyHandler({'https':'http://127.0.0.1:3060','http':'http://127.0.0.1:3060'})
try:
 token=subprocess.check_output(['gh','auth','token'],stderr=subprocess.DEVNULL).decode().strip()
 request=urllib.request.Request('https://api.github.com/repos/gzers/opencodex-desktop/actions/artifacts/13630874650/zip?download_nonce='+uuid.uuid4().hex,headers={'Authorization':'Bearer '+token,'Accept':'application/vnd.github+json','Cache-Control':'no-cache'})
 try:urllib.request.build_opener(proxy,NoRedirect).open(request,timeout=30);raise RuntimeError('Expected original artifact redirect')
 except urllib.error.HTTPError as error:
  if error.code!=302:raise RuntimeError('Artifact admission HTTP '+str(error.code)) from None
  location=error.headers['Location']
 del token,request
 def part(index):
  start=index*4*1024*1024;end=min(size-1,start+4*1024*1024-1)
  request=urllib.request.Request(location,headers={'Range':f'bytes={start}-{end}','Accept-Encoding':'identity','Cache-Control':'no-cache'})
  with urllib.request.build_opener(proxy).open(request,timeout=120) as response:
   if response.status!=206 or response.headers.get('Content-Range')!=f'bytes {start}-{end}/{size}':raise RuntimeError('Range identity mismatch')
   value=response.read(end-start+2)
  if len(value)!=end-start+1:raise RuntimeError('Incomplete exact range')
  path=root/('part-'+str(index)+'.bin');path.write_bytes(value);return {'part':index,'bytes':len(value),'sha256':hashlib.sha256(value).hexdigest()}
 count=(size+4*1024*1024-1)//(4*1024*1024)
 with concurrent.futures.ThreadPoolExecutor(max_workers=4) as executor:parts=list(executor.map(part,range(count)))
 target=root/'artifact-ranged.zip'
 with target.open('xb') as output:
  for index in range(count):output.write((root/('part-'+str(index)+'.bin')).read_bytes())
 actual=hashlib.sha256(target.read_bytes()).hexdigest()
 if target.stat().st_size!=size or actual!=expected:raise RuntimeError('Original ZIP size or GitHub SHA256 mismatch')
 (root/'download-verification.json').write_text(json.dumps({'artifactId':13630874650,'bytes':size,'sha256':actual,'parts':parts,'verified':True},indent=2))
 print('Verified complete original artifact ZIP: bytes='+str(size)+' sha256='+actual)
except Exception as error:
 print('Bounded artifact range download did not complete: '+type(error).__name__)
 raise SystemExit(1)
