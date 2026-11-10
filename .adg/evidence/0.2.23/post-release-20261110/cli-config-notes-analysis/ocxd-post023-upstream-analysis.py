import urllib.request,json,pathlib,hashlib,concurrent.futures,urllib.error
out=pathlib.Path('/private/tmp/ocxd-post023-upstream-analysis-20261110');out.mkdir(exist_ok=True)
opener=urllib.request.build_opener(urllib.request.ProxyHandler({'http':'http://127.0.0.1:3060','https':'http://127.0.0.1:3060'}))
def get(version):
 url='https://registry.npmjs.org/@bitkyc08%2fopencodex/'+version
 with opener.open(urllib.request.Request(url,headers={'User-Agent':'ocxd-readonly-analysis'}),timeout=35) as r:data=r.read()
 d=json.loads(data);(out/(version+'.metadata.json')).write_bytes(data)
 print(json.dumps({'requested':version,'version':d.get('version'),'repository':d.get('repository'),'bin':d.get('bin'),'dist':d.get('dist'),'gitHead':d.get('gitHead')},ensure_ascii=False))
 return d
with concurrent.futures.ThreadPoolExecutor(max_workers=3) as pool: list(pool.map(get,['2.97.0','2.99.0','latest']))
