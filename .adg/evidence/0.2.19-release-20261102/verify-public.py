from pathlib import Path
import json,urllib.request,hashlib,time,sys
root=Path(sys.argv[1]);target=json.loads((root/'verification.json').read_text());assets=target['assets']
def get(url):
 for attempt in range(4):
  try:
   req=urllib.request.Request(url,headers={'User-Agent':'ocx019-anonymous-release-acceptance'})
   with urllib.request.urlopen(req,timeout=40) as response:return response.read(),response.status
  except Exception:
   if attempt==3:raise
   time.sleep(3)
repo='gzers/opencodex-desktop';tag='v0.2.19';base='https://github.com/'+repo+'/releases'
metadata=json.loads(get('https://api.github.com/repos/'+repo+'/releases/latest')[0]);assert metadata['tag_name']==tag and not metadata['draft'] and not metadata['prerelease']
expected=json.loads((root/'assets/latest.json').read_text());manifests=[]
for url in [base+'/download/'+tag+'/latest.json',base+'/latest/download/latest.json']:
 data,status=get(url);assert status==200 and json.loads(data)==expected;manifests.append({'url':url,'status':status,'version':expected['version']})
checks=[]
from urllib.parse import quote
for asset in assets:
 url=base+'/download/'+tag+'/'+quote(asset['name']);data,status=get(url)
 assert status==200 and len(data)==asset['size'] and hashlib.sha256(data).hexdigest()==asset['sha256']
 checks.append({'name':asset['name'],'httpStatus':status,'size':len(data),'sha256':asset['sha256'],'anonymous':True})
report={'result':'pass','latest':tag,'public':True,'nonPrerelease':True,'authorizationHeadersSent':False,'manifests':manifests,'assets':checks}
(root/'anonymous-public.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
