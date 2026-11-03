from pathlib import Path
import json,subprocess,hashlib,base64,urllib.parse,plistlib
repo='gzers/opencodex-desktop';tag='v0.2.20';source='34a3553f9f014c0d553e16e8c509669a3e64fcd9'
code=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes')
root=Path('/tmp/ocx020-update-scale-final');assets=root/'assets';assets.mkdir(exist_ok=True)
def gh(*args):return subprocess.check_output(['gh',*args])
release_id=json.loads((root/'new-private-release.json').read_text())['id']
release=json.loads(gh('api',f'repos/{repo}/releases/{release_id}'));assert release['draft'] and release['prerelease'] and release['id']!=432929365
ref=json.loads(gh('api',f'repos/{repo}/git/ref/tags/{tag}'));assert ref['object']['sha']==source
expected={'latest.json','OpenCodeX.Desktop_0.2.20_aarch64.dmg','OpenCodeX.Desktop_0.2.20_x64-setup.exe','OpenCodeX.Desktop_0.2.20_x64-setup.exe.sig','OpenCodeX.Desktop_0.2.20_x64_en-US.msi','OpenCodeX.Desktop_0.2.20_x64_en-US.msi.sig','OpenCodeX.Desktop_aarch64.app.tar.gz','OpenCodeX.Desktop_aarch64.app.tar.gz.sig'}
assert len(release['assets'])==8 and {a['name'] for a in release['assets']}==expected
results=[]
for asset in release['assets']:
 name=asset['name'];path=assets/name
 if not path.exists():
  with path.open('wb') as stream:subprocess.run(['gh','api',f"repos/{repo}/releases/assets/{asset['id']}",'--header','Accept: application/octet-stream','--allow-escape-sequences'],stdout=stream,check=True)
 value=path.read_bytes();digest=hashlib.sha256(value).hexdigest()
 assert len(value)==asset['size'] and asset['digest']=='sha256:'+digest,(name,digest,asset['digest'])
 results.append({'name':name,'id':asset['id'],'bytes':len(value),'sha256':digest})
manifest=json.loads((assets/'latest.json').read_text());assert manifest['version']=='0.2.20'
required={'darwin-aarch64','darwin-aarch64-app','windows-x86_64','windows-x86_64-msi','windows-x86_64-nsis'};assert set(manifest['platforms'])==required
config=json.loads(subprocess.check_output(['git','show',source+':apps/desktop/tauri/tauri.conf.json'],cwd=code));assert config['version']=='0.2.20'
key=root/'updater.pub';key.write_bytes(base64.b64decode(config['plugins']['updater']['pubkey'],validate=True));signed=[]
for platform,entry in manifest['platforms'].items():
 prefix='https://github.com/'+repo+'/releases/download/'+tag+'/'
 url=urllib.parse.unquote(entry['url']);assert url.startswith(prefix);name=url[len(prefix):];assert name in expected
 assert entry['signature']==(assets/(name+'.sig')).read_text().strip(),platform
 if name in signed:continue
 signature=root/(name+'.minisig');signature.write_bytes(base64.b64decode(entry['signature'],validate=True))
 verified=subprocess.run(['minisign','-V','-m',str(assets/name),'-p',str(key),'-x',str(signature)],capture_output=True,text=True);assert verified.returncode==0,verified.stderr
 (root/(name+'.verification.log')).write_text(verified.stdout+verified.stderr);signed.append(name)
assert len(signed)==3
unpack=root/'unpacked';unpack.mkdir(exist_ok=True)
app=unpack/'OpenCodeX Desktop.app'
if not app.exists():subprocess.run(['tar','-xzf',str(assets/'OpenCodeX.Desktop_aarch64.app.tar.gz'),'-C',str(unpack)],check=True)
subprocess.run(['codesign','--verify','--deep','--strict',str(app)],check=True)
details=subprocess.run(['codesign','-dvv',str(app)],capture_output=True,text=True);(root/'macos-codesign.log').write_text(details.stdout+details.stderr);assert details.returncode==0
plist=plistlib.loads((app/'Contents/Info.plist').read_bytes());assert plist['CFBundleShortVersionString']=='0.2.20' and plist['CFBundleIdentifier']=='com.gzers.opencodex.desktop'
binary=app/'Contents/MacOS/opencodex-desktop';data=binary.read_bytes();assert source.encode() in data
for name in ['LICENSE','THIRD-PARTY-NOTICES.txt']:assert (app/'Contents/Resources/resources'/name).stat().st_size>0
dmg=subprocess.run(['hdiutil','verify',str(assets/'OpenCodeX.Desktop_0.2.20_aarch64.dmg')],capture_output=True,text=True);(root/'dmg-verification.log').write_text(dmg.stdout+dmg.stderr);assert dmg.returncode==0
result={'result':'pass','sourceCommit':source,'releaseId':release['id'],'draft':release['draft'],'prerelease':release['prerelease'],'assets':sorted(results,key=lambda a:a['name']),'signedArtifacts':signed,'platforms':sorted(required),'macBundle':{'version':plist['CFBundleShortVersionString'],'id':plist['CFBundleIdentifier'],'deepStrictSignature':'pass','embeddedSourceCommit':source,'executableSha256':hashlib.sha256(data).hexdigest(),'licenseAndNotices':'present'},'scope':'actual uploaded8assets/3updater signatures+sidecars/manifest/Macbundle seal/commit/resources/DMG; installation and actual OTA separate'}
(root/'full-release-before-public.json').write_text(json.dumps(release,indent=2)+'\n');(root/'asset-verification.json').write_text(json.dumps(result,indent=2)+'\n');print(json.dumps(result,indent=2))
