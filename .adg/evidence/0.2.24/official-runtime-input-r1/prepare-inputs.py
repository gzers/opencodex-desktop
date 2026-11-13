from pathlib import Path
import json,hashlib,urllib.request,importlib.util,tarfile,base64,time
c=json.loads(Path('/private/tmp/ocxd024official-active.json').read_text());r=Path(c['root']);e=Path(c['evidence']);repo=Path(c['repo']);policy=json.loads((repo/'.github/runtime-platforms-023.json').read_text());opener=urllib.request.build_opener(urllib.request.ProxyHandler({'http':'http://127.0.0.1:3060','https':'http://127.0.0.1:3060'}));results=[]
def get(url,path,expected,limit):
 start=time.monotonic();total=0;h=hashlib.sha256()
 with opener.open(url,timeout=30) as source,path.open('xb') as out:
  while True:
   block=source.read(1024*1024)
   if not block:break
   total+=len(block)
   if total>limit:raise RuntimeError('artifact exceeds limit')
   h.update(block);out.write(block)
 if h.hexdigest()!=expected:raise RuntimeError('artifact SHA256 mismatch')
 result={'url':url,'path':str(path),'bytes':total,'sha256':h.hexdigest(),'elapsedSeconds':time.monotonic()-start,'digestMatched':True};results.append(result);(e/'download-progress.json').write_text(json.dumps(results,indent=2));print(json.dumps(result),flush=True)
try:
 archive=r/'opencodex-2.99.0.tgz';get('https://registry.npmjs.org/@bitkyc08/opencodex/-/opencodex-2.99.0.tgz',archive,'b920ba0214a046e3c6a73447933ea467d52368d1eefb300e274394247414259c',64*1024*1024)
 spec=policy['platforms']['darwin-arm64']['nodeArchive'];node_archive=r/spec['name'];get(spec['url'],node_archive,spec['sha256'],256*1024*1024)
 with tarfile.open(archive,'r:gz') as tar:
  member=tar.getmember('package/package.json')
  if not member.isfile() or member.size>1024*1024:raise RuntimeError('package metadata unsafe')
  package=json.loads(tar.extractfile(member).read())
 if package['name']!='@bitkyc08/opencodex' or package['version']!='2.99.0':raise RuntimeError('official identity mismatch')
 old=policy['packageLock']['packages']['node_modules/@bitkyc08/opencodex']
 if old['dependencies']!=package['dependencies'] or old.get('optionalDependencies',{})!=package.get('optionalDependencies',{}):raise RuntimeError('official dependencies differ; lock must be resolved explicitly, cannot reuse blindly')
 policy['bundleVersion']='0.2.24';policy['runtime']={'version':'2.99.0','archive':archive.name,'sha256':results[0]['sha256'],'distribution':'official-npm','url':results[0]['url']}
 for platform in policy['platforms'].values():platform['archiveName']=platform['archiveName'].replace('0.2.23','0.2.24')
 policy['projectPackage']['version']='0.2.24';policy['projectPackage']['dependencies']['@bitkyc08/opencodex']='file:sources/'+archive.name
 lock=policy['packageLock'];lock['version']='0.2.24';lock['packages']['']['version']='0.2.24';lock['packages']['']['dependencies']['@bitkyc08/opencodex']='file:sources/'+archive.name
 row=lock['packages']['node_modules/@bitkyc08/opencodex'];row['version']='2.99.0';row['resolved']='file:sources/'+archive.name;row['integrity']='sha512-'+base64.b64encode(hashlib.sha512(archive.read_bytes()).digest()).decode();row['bin']=package['bin']
 (r/'runtime-platforms-024.json').write_text(json.dumps(policy,indent=2)+chr(10));(e/'official-package-public.json').write_text(json.dumps({k:package.get(k) for k in ['name','version','dependencies','optionalDependencies','bin','engines']},indent=2));(e/'lock-reuse-proof.json').write_text(json.dumps({'officialArchiveIdentityMatched':True,'dependencyRangesExactlyEqualPriorFrozenLock':True,'transitiveLockEntriesPreserved':True,'projectAndRootOfficialPackageMetadataUpdatedOnly':True,'noLifecycleScriptsOrRuntimeExecuted':True},indent=2))
 module=importlib.util.spec_from_file_location('build_bundle',repo/'.github/scripts/build_desktop_runtime_bundle.py');builder=importlib.util.module_from_spec(module);module.loader.exec_module(builder);builder.extract_node(node_archive,r/'host-node',spec)
 (e/'result.json').write_text(json.dumps({'status':'prepared','downloads':results,'officialVersion':'2.99.0','runtimeSourcesModified':False,'policyCandidate':str(r/'runtime-platforms-024.json'),'buildNode':str(r/'host-node/bin/node'),'npmCli':str(r/'host-node/lib/node_modules/npm/bin/npm-cli.js'),'isolatedRoot':str(r)},indent=2));print('prepared',flush=True)
except Exception as error:
 (e/'failure.json').write_text(json.dumps({'error':str(error),'downloads':results,'workPreserved':True},indent=2));raise
