import urllib.request,urllib.error,json,pathlib,concurrent.futures,time,hashlib
out=pathlib.Path('/private/tmp/ocxd-post023-upstream-analysis-20261110')
class NoRedirect(urllib.request.HTTPRedirectHandler):
 def redirect_request(self,*a,**k):return None
urls={'npm':'https://registry.npmjs.org/@bitkyc08%2fopencodex/2.99.0','release':'https://api.github.com/repos/lidge-jun/opencodex/releases/tags/v2.99.0'}
def probe(item):
 mode,name,url=item
 op=urllib.request.build_opener(urllib.request.ProxyHandler({} if mode=='direct' else {'http':'http://127.0.0.1:3060','https':'http://127.0.0.1:3060'}),NoRedirect())
 start=time.monotonic();receipt={'mode':mode,'target':name,'url':url}
 try:
  with op.open(urllib.request.Request(url,headers={'User-Agent':'OpenCodeX-Desktop','Accept':'application/json'}),timeout=12) as r:
   b=r.read(262145);status=r.status
   receipt.update(status=status,bytes=len(b),sha256=hashlib.sha256(b).hexdigest(),withinLimit=len(b)<=262144)
   (out/(mode+'-'+name+'.response.json')).write_bytes(b)
   v=json.loads(b)
   if name=='npm':receipt['sourceIdentityPass']=v['name']=='@bitkyc08/opencodex' and v['version']=='2.99.0' and v['repository']['url']=='git+https://github.com/lidge-jun/opencodex.git'
   else:receipt.update(sourceIdentityPass=v['tag_name']=='v2.99.0' and v['draft']==False and v['html_url']=='https://github.com/lidge-jun/opencodex/releases/tag/v2.99.0',bodyPresent=bool(v.get('body')),prerelease=v.get('prerelease'))
 except urllib.error.HTTPError as e:receipt.update(status=e.code,error='HTTPError',rateLimitRemaining=e.headers.get('X-RateLimit-Remaining'),redirect=bool(e.headers.get('Location')))
 except Exception as e:receipt.update(error=type(e).__name__,detail=str(e)[:180])
 receipt['seconds']=round(time.monotonic()-start,3);return receipt
with concurrent.futures.ThreadPoolExecutor(max_workers=4) as p:rows=list(p.map(probe,[(m,n,u) for m in ['direct','task-proxy'] for n,u in urls.items()]))
(out/'release-notes-probes.json').write_text(json.dumps(rows,indent=2)+'\n');print(json.dumps(rows,indent=2))
