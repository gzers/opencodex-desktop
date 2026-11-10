from pathlib import Path
import json,urllib.request,base64,hashlib,tarfile,io,os
r=Path(__file__).parent
meta=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main/.adg/evidence/0.2.23/post-release-20261110/cli-config-notes-analysis/2.99.0.metadata.json')
d=json.loads(meta.read_text());op=urllib.request.build_opener(urllib.request.ProxyHandler({'https':'http://127.0.0.1:3060'}))
with op.open(d['dist']['tarball'],timeout=40) as response:b=response.read(32*1024*1024)
assert 'sha512-'+base64.b64encode(hashlib.sha512(b).digest()).decode()==d['dist']['integrity']
with tarfile.open(fileobj=io.BytesIO(b)) as t:
 for m in t:
  rel=Path(m.name.removeprefix('package/'))
  assert not rel.is_absolute() and '..' not in rel.parts
  if not m.isfile() or not (str(rel).startswith('src/') or str(rel)=='package.json'):continue
  assert m.size<12*1024*1024
  p=r/'official299'/rel;p.parent.mkdir(parents=True,exist_ok=True);p.write_bytes(t.extractfile(m).read())
(r/'official299/node_modules').symlink_to('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules',target_is_directory=True)
for sub in ['fixture-user/.codex','fixture-user/.opencodex','fixture-user/.codex-desktop/opencodex/home','tmp','cache']: (r/sub).mkdir(parents=True,exist_ok=True)
(r/'sandbox.sb').write_text('(version 1)\n(allow default)\n(deny network*)\n(deny file-write*)\n(allow file-write* (subpath "'+str(r)+'"))\n'+''.join('(deny file-read* (subpath "'+p+'"))\n' for p in ['/Users/ezio/.codex','/Users/ezio/.opencodex','/Users/ezio/.codex-desktop','/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/opencodex-home','/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/manager-state']))
print('official299 source extracted, package integrity verified; read-only external dependencies; sandbox denies network, daily data and writes outside fixture')
