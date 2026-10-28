#!/usr/bin/env python3
"""One explicitly selected, output-bounded real request through an existing local proxy.
Uses no key file or inherited secret. Synthetic payload only; no runtime config mutation.
"""
import argparse,json,time,urllib.request,urllib.error,hashlib
from datetime import datetime,timezone
from pathlib import Path
p=argparse.ArgumentParser(description=__doc__)
p.add_argument('--base-url',default='http://127.0.0.1:10100/v1')
p.add_argument('--model',required=True);p.add_argument('--scenario',choices=['basic','search'],default='basic');p.add_argument('--output',required=True,type=Path)
a=p.parse_args()
if a.base_url!='http://127.0.0.1:10100/v1':raise SystemExit('This probe only targets the observed existing local proxy.')
opener=urllib.request.build_opener(urllib.request.ProxyHandler({}))
payload={'model':a.model,'input':[{'role':'user','content':'Synthetic compatibility probe. Reply exactly PROBE_OK. No tools, no external actions.'}],'reasoning':{'effort':'low'},'max_output_tokens':512,'stream':False,'store':False}
if a.scenario=='search':
 payload['input']=[{'role':'user','content':'Synthetic search compatibility probe. Use web search once to open the public IANA example.com page. Return only its page heading and its public URL. Do not access any other user data.'}]
 payload['tools']=[{'type':'web_search'}];payload['tool_choice']='required'
r={'atUtc':datetime.now(timezone.utc).isoformat(),'route':a.base_url+'/responses','requestedModel':a.model,'scenario':a.scenario,'syntheticInputOnly':True,'credentialsRead':False,'dailyConfigModified':False,'maxOutputTokens':512,'scriptSha256':hashlib.sha256(Path(__file__).read_bytes()).hexdigest(),'actualClientRequests':0,'checks':[]}
t=time.monotonic()
try:
 with opener.open('http://127.0.0.1:10100/healthz',timeout=8) as h:
  health=json.load(h);r['proxyIdentity']={k:health.get(k) for k in ['service','version','pid']}
 if r['proxyIdentity']['service']!='opencodex':raise RuntimeError('Existing listener is not the observed OpenCodex service.')
 req=urllib.request.Request(r['route'],data=json.dumps(payload).encode(),headers={'Content-Type':'application/json'},method='POST');r['actualClientRequests']=1
 with opener.open(req,timeout=55) as response:
  raw=response.read(2*1024*1024+1);r['httpStatus']=response.status
  if len(raw)>2*1024*1024:raise RuntimeError('response exceeds probe bound')
  value=json.loads(raw)
 text=''.join(c.get('text','') for item in value.get('output',[]) if isinstance(item,dict) for c in item.get('content',[]) if isinstance(c,dict) and c.get('type')=='output_text').strip()
 kinds=[x.get('type') for x in value.get('output',[]) if isinstance(x,dict)]
 r.update({'responseStatus':value.get('status'),'servedModel':value.get('model'),'syntheticOutput':text[:1200],'usage':value.get('usage'),'outputKinds':kinds,'serviceTier':value.get('service_tier')})
 ok=value.get('status')=='completed' and (text=='PROBE_OK' if a.scenario=='basic' else 'web_search_call' in kinds)
 r['status']='passed' if ok else 'inconclusive'
 r['checks'].append({'id':'real_'+a.scenario+'_response','status':r['status'],'scope':'requested route and synthetic task only; catalog presence is not proof of service availability'})
except urllib.error.HTTPError as error:r.update({'status':'failed','httpStatus':error.code,'error':error.read(1400).decode(errors='replace')})
except Exception as error:r.update({'status':'inconclusive','error':type(error).__name__+': '+str(error)[:400]})
r['elapsedSeconds']=round(time.monotonic()-t,3)
a.output.parent.mkdir(parents=True,exist_ok=True);a.output.write_text(json.dumps(r,ensure_ascii=False,indent=2)+'\n')
print(json.dumps(r,ensure_ascii=False,indent=2))
raise SystemExit(0 if r['status']=='passed' else 1)
