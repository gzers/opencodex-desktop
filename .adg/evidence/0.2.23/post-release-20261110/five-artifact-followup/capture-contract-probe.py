from pathlib import Path
import json,hashlib,base64,os
from importlib.util import spec_from_file_location,module_from_spec
HERE=Path(__file__).parent
# Independent bounded collector: only explicit synthetic fixture objects, no discovered user files.
def take(root,paths):
 records=[]
 for rel in paths:
  if rel not in {'codex/config.toml','codex/opencodex-catalog.json','codex/models_cache.json','codex/opencodex-catalog-audit.jsonl','grok/config.toml.bak-opencodex'}:raise ValueError('unregistered-synthetic-target')
  path=root/rel
  try:
   before=path.lstat()
   if not path.is_file() or path.is_symlink() or before.st_nlink!=1:raise ValueError('unsafe-target')
   if before.st_size>2*1024*1024:raise ValueError('capture-too-large')
   fd=os.open(path,os.O_RDONLY|os.O_NOFOLLOW)
   try:
    opened=os.fstat(fd);data=os.read(fd,before.st_size+1);after=os.fstat(fd);last=path.lstat();key=lambda s:(s.st_dev,s.st_ino,s.st_size,s.st_mtime_ns,s.st_ctime_ns,s.st_nlink)
    if not key(before)==key(opened)==key(after)==key(last) or len(data)!=before.st_size:raise ValueError('capture-changed')
   finally:os.close(fd)
   records.append({'path':rel,'state':'file','size':len(data),'sha256':hashlib.sha256(data).hexdigest(),'bytesBase64':base64.b64encode(data).decode()})
  except FileNotFoundError:records.append({'path':rel,'state':'absent'})
  except (OSError,ValueError) as err:records.append({'path':rel,'state':'unknown','errorType':type(err).__name__,'errno':getattr(err,'errno',None)})
 return records
root=Path(json.loads((HERE/'fixture.json').read_text())['root'])/'capture-contract';(root/'codex').mkdir(parents=True);(root/'grok').mkdir();checks=[]
(root/'codex/config.toml').write_bytes(b'model="changed-synthetic-value"\r\n');(root/'codex/opencodex-catalog.json').write_bytes(b'{invalid-but-captured-after');(root/'codex/models_cache.json').write_bytes(b'locked-synthetic');(root/'codex/models_cache.json').chmod(0)
(root/'grok/config.toml.bak-opencodex').symlink_to(root/'codex/config.toml')
rows=take(root,['codex/config.toml','codex/opencodex-catalog.json','codex/models_cache.json','codex/opencodex-catalog-audit.jsonl','grok/config.toml.bak-opencodex']);values={x['path']:x for x in rows};assert base64.b64decode(values['codex/config.toml']['bytesBase64'])==b'model="changed-synthetic-value"\r\n';checks.append('changed contents saved verbatim without equality gate')
assert base64.b64decode(values['codex/opencodex-catalog.json']['bytesBase64'])==b'{invalid-but-captured-after';checks.append('invalid JSON raw afterimage captured; parse failure cannot discard it')
assert values['codex/models_cache.json']['state']=='unknown';checks.append('unreadable does not mean absent and does not abort remaining collection')
assert values['codex/opencodex-catalog-audit.jsonl']['state']=='absent';checks.append('actual missing recorded distinctly')
assert values['grok/config.toml.bak-opencodex']['state']=='unknown';checks.append('symlink not followed or serialized')
for x in rows:
 if x['state']=='file':assert hashlib.sha256(base64.b64decode(x['bytesBase64'])).hexdigest()==x['sha256']
checks.append('all serialized raw bytes match recorded size/digest')
try:take(root,['codex/auth.json'])
except ValueError:pass
else:raise AssertionError('arbitrary secret target allowed')
checks.append('unregistered credential or discovered target rejected')
(root/'codex/models_cache.json').chmod(0o600)
(HERE/'capture-contract-result.json').write_text(json.dumps({'result':'pass','checks':checks,'count':len(checks),'scope':'explicit synthetic allowlist; POSIX collector proof, not PowerShell/Windows production capture','states':[{k:v for k,v in x.items() if k!='bytesBase64'} for x in rows]},ensure_ascii=False,indent=2));print(json.dumps({'result':'pass','checks':len(checks)}))
