import sys,subprocess,json,time,hashlib,base64,importlib.util,tarfile,plistlib
from pathlib import Path
repo='gzers/opencodex-desktop'; tag='v0.2.19'
code=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes')
out=Path(sys.argv[1]);out.mkdir(parents=True,exist_ok=True)
def gh(*args):
 for attempt in range(3):
  p=subprocess.run(['gh',*args],capture_output=True,text=True)
  if p.returncode==0:return p.stdout
  if attempt==2:raise RuntimeError(p.stderr)
  time.sleep(2)
spec=importlib.util.spec_from_file_location('publish_release',code/'.github/scripts/publish_release.py');mod=importlib.util.module_from_spec(spec);spec.loader.exec_module(mod);mod.gh=gh
release=mod.get_release(repo,tag)
(out/'release-metadata.json').write_text(json.dumps(release,indent=2)+'\n')
assets=release['assets'];assert len(assets)==8,len(assets)
assetdir=out/'assets';assetdir.mkdir(exist_ok=True)
inventory=[]
for asset in assets:
 p=assetdir/asset['name']
 if not p.exists():gh('release','download',tag,'--repo',repo,'--pattern',asset['name'],'--output',str(p))
 data=p.read_bytes();digest=hashlib.sha256(data).hexdigest()
 assert asset['size']==len(data)
 assert asset['digest']=='sha256:'+digest,(p,asset['digest'],digest)
 inventory.append({'name':p.name,'size':len(data),'sha256':digest})
manifest=json.loads((assetdir/'latest.json').read_text());mod.validate_manifest(manifest,release,tag,repo)
config=json.loads(base64.b64decode(json.loads(gh('api',f'repos/{repo}/contents/apps/desktop/tauri/tauri.conf.json?ref={tag}'))['content']))
original=json.loads(subprocess.check_output(['git','show','v0.2.18:apps/desktop/tauri/tauri.conf.json'],cwd=code))
assert config['plugins']['updater']['pubkey']==original['plugins']['updater']['pubkey']
key=out/'updater.pub';key.write_bytes(base64.b64decode(config['plugins']['updater']['pubkey'],validate=True))
signed=[];seen=set()
for platform,entry in manifest['platforms'].items():
 name=entry['url'].rsplit('/',1)[-1]
 from urllib.parse import unquote
 name=unquote(name)
 if name in seen:continue
 seen.add(name);p=assetdir/name;sig=out/(name+'.manifest.minisig');sig.write_bytes(base64.b64decode(entry['signature'],validate=True))
 result=subprocess.run(['minisign','-V','-m',str(p),'-p',str(key),'-x',str(sig)],capture_output=True,text=True)
 (out/(platform+'-minisign.log')).write_text(result.stdout+result.stderr);assert result.returncode==0
 signed.append({'platform':platform,'asset':name,'signature':'verified with unchanged 0.2.18 key'})
assert len(signed)==3
appdir=out/'macos-app';appdir.mkdir(exist_ok=True)
with tarfile.open(assetdir/'OpenCodeX.Desktop_aarch64.app.tar.gz') as tf:
 for m in tf.getmembers():
  assert not Path(m.name).is_absolute() and '..' not in Path(m.name).parts
 tf.extractall(appdir,filter='data')
apps=list(appdir.glob('*.app'));assert len(apps)==1
app=apps[0];info=plistlib.loads((app/'Contents/Info.plist').read_bytes())
assert info['CFBundleIdentifier']=='com.gzers.opencodex.desktop'
assert info['CFBundleShortVersionString']=='0.2.19'
for name in ['LICENSE','THIRD-PARTY-NOTICES.txt']:
 p=app/'Contents/Resources/resources'/name;assert p.is_file() and p.stat().st_size>0
for args,name in [(['codesign','--verify','--deep','--strict',str(app)],'macos-signature-verify.log'),(['codesign','-dvv',str(app)],'macos-signature-details.log')]:
 p=subprocess.run(args,capture_output=True,text=True);(out/name).write_text(p.stdout+p.stderr);assert p.returncode==0
exe=app/'Contents/MacOS'/info['CFBundleExecutable']
source=subprocess.check_output(['git','rev-parse',tag+'^{commit}'],cwd=code,text=True).strip()
assert source.encode() in exe.read_bytes(),'Package must contain tagged build commit'
report={'result':'pass','tag':tag,'source':source,'releaseId':release['id'],'draft':release['draft'],'assets':inventory,'updaterSignatures':signed,'macosSealedResources':True,'macosVersion':info['CFBundleShortVersionString'],'macosExecutableSha256':hashlib.sha256(exe.read_bytes()).hexdigest(),'dailyInstallTouched':False,'notarizationClaimed':False}
(out/'verification.json').write_text(json.dumps(report,indent=2)+'\n');print(json.dumps(report,indent=2))
