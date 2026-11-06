"""Read-only configured WebDAV capability/content probe; never prints secrets or remote filenames.
Allowed remote methods: PROPFIND Depth1 and GET only. No upload/LOCK/MKCOL/DELETE.
Credential read from existing application store in memory; TLS verified, redirects denied.
"""
from pathlib import Path
import base64,hashlib,json,ssl,urllib.request,urllib.error,urllib.parse,xml.etree.ElementTree as ET,uuid,datetime
ROOT=Path(__file__).parent
before={};report={'timeUtc':datetime.datetime.now(datetime.timezone.utc).isoformat(),'methodsAllowed':['GET','PROPFIND'],'remoteMutations':False,'requests':[]}
def read(p):
 b=p.read_bytes();before[p]=hashlib.sha256(b).hexdigest();return json.loads(b)
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*args,**kwargs):return None
opener=urllib.request.build_opener(urllib.request.HTTPSHandler(context=ssl.create_default_context()),NoRedirect())
def request(kind,name,method='GET',limit=65536):
 assert method in ('GET','PROPFIND')
 url=directory+urllib.parse.quote(name,safe='-._~')
 headers={'Authorization':auth,'Cache-Control':'no-cache','Pragma':'no-cache','Accept-Encoding':'identity'}
 body=None
 if method=='PROPFIND':
  headers.update({'Depth':'1','Content-Type':'application/xml; charset=utf-8'});body=b'<?xml version="1.0"?><d:propfind xmlns:d="DAV:"><d:prop><d:resourcetype/><d:getcontentlength/></d:prop></d:propfind>'
 row={'kind':kind,'method':method}
 try:
  with opener.open(urllib.request.Request(url,data=body,headers=headers,method=method),timeout=12) as r:
   b=r.read(limit+1);row['status']=r.status;row['bytes']=len(b)
   e=r.headers.get('ETag');row['etagKind']='missing' if not e else 'weak' if e.startswith('W/') else 'strong-shaped' if e.startswith('"') and e.endswith('"') else 'other'
   if len(b)>limit:row['failure']='response-size-limit';return None
   row['bodySha256']=hashlib.sha256(b).hexdigest();return b
 except urllib.error.HTTPError as e:row['status']=e.code;row['failure']='http-error';return None
 except urllib.error.URLError as e:row['failure']='tls-error' if isinstance(e.reason,ssl.SSLError) else 'transport-error';return None
 except Exception as e:row['failure']=type(e).__name__;return None
 finally:report['requests'].append(row);print(json.dumps(row),flush=True)
try:
 anchor=Path('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop');meta=read(anchor/'manager-state/data-root.json');root=Path(meta['active_data_root']);cfg=read(root/'manager-state/sync-endpoints.json');e=cfg['endpoints'][0]
 u=urllib.parse.urlsplit(e['url']);assert u.scheme=='https' and u.hostname and not u.username and not u.password and not u.query and not u.fragment
 remote=e['remote_path'].strip('/');assert remote and all(x not in ('.','..','') for x in remote.split('/'))
 directory=e['url'].strip().rstrip('/')+'/'+urllib.parse.quote(remote,safe='/-._~')+'/'
 ref=e['credential_ref'];account=ref['account_key'];assert account=='ocx.dav.'+ref['ref_id'];assert ref['purpose']=='webdav_credential';assert ref['service_name']=='OpenCodex Desktop'
 safe=''.join(c if c.isascii() and (c.isalnum() or c in '.-') else '_' for c in account)
 cred=read(root/'manager-state/credentials'/(safe+'.json'));assert cred['account']==account and cred['protection']=='plain' and cred['value']
 auth='Basic '+base64.b64encode((e['username'].strip()+':'+cred['value']).encode()).decode();cred=None
 report['endpointIdentity']='configured-active-endpoint-only';listing_sets=[]
 for index in range(2):
  b=request('directory-depth1-'+str(index+1),'','PROPFIND',2*1024*1024)
  if b is None:continue
  try:
   xml=ET.fromstring(b);responses=list(xml.iter('{DAV:}response'));names=set()
   for response in responses:
    href=response.find('{DAV:}href');status=response.find('{DAV:}status')
    if href is None or href.text is None:continue
    path=urllib.parse.unquote(urllib.parse.urlsplit(href.text).path).rstrip('/');name=path.rsplit('/',1)[-1]
    if name.startswith('config-v1-') or name=='config-v1-latest.json':names.add(name)
   listing_sets.append(names);report.setdefault('directoryResults',[]).append({'responseCount':len(responses),'configFileCount':len(names),'currentPointerListed':'config-v1-latest.json' in names})
  except Exception as err:report.setdefault('directoryResults',[]).append({'failure':'xml-'+type(err).__name__})
 report['twoListingsSameObservedConfigNames']=len(listing_sets)==2 and listing_sets[0]==listing_sets[1]
 first=request('latest-pointer-first','config-v1-latest.json','GET',4096)
 if first is not None:
  try:
   pointer=json.loads(first);uuid.UUID(pointer['operationId']);digest=pointer['digest'];assert pointer['format']=='OCXD_CONFIG_POINTER' and pointer['version']==1 and digest.startswith('sha256:') and len(digest)==71 and all(c in '0123456789abcdefABCDEF' for c in digest[7:]);report['pointerStructureValid']=True
   payload=request('current-payload-digest-check','config-v1-'+digest[7:]+'.ocxd','GET',64*1024*1024)
   report['payloadMatchesPointerDigest']=payload is not None and 'sha256:'+hashlib.sha256(payload).hexdigest()==digest
   if payload is not None:report['payloadBytes']=len(payload)
   payload=None
   latest=root/'manager-state/transfer-results/latest-webdav.json'
   if latest.is_file():
    op=read(latest);uuid.UUID(op);rp=latest.parent/(op+'.json');receipt=read(rp) if rp.is_file() else None
    report['currentPointerIsLastLocalOperation']=op==pointer['operationId'];report['lastLocalReceiptError']=receipt.get('errorCode') if receipt else None
   second=request('latest-pointer-final','config-v1-latest.json','GET',4096);report['pointerContentStableDuringProbe']=second is not None and json.loads(second)==pointer
  except Exception as err:report['pointerAnalysisFailure']=type(err).__name__
 report['limits']=['Read-only observation at this time, not original failed request replay','Depth1 returns existing file enumeration only; does not prove fresh-upload visibility latency or complete consistency under concurrent writes','No writes, endpoint edits, service/Codex restart, password print or raw remote body saved','No claim this confirms Windows endpoint state or recovery/migration product implementation']
except Exception as err:report['setupFailure']=type(err).__name__
finally:
 report['localInputBytesUnchanged']=all(p.is_file() and hashlib.sha256(p.read_bytes()).hexdigest()==h for p,h in before.items());report['localFilesRead']=len(before);report['secretsSavedOrPrinted']=False
 (ROOT/'readonly-probe-result.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+chr(10));print('SUMMARY '+json.dumps({k:v for k,v in report.items() if k!='requests'},ensure_ascii=False),flush=True)
