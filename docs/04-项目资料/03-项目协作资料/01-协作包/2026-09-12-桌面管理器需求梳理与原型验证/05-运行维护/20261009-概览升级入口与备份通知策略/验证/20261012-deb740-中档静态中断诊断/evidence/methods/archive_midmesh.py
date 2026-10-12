"""Immutable bounded evidence archive; raw bytes retained, attribution filtered."""
import argparse
import datetime as dt
import gzip
import hashlib
import json
from pathlib import Path
import subprocess
import sys
import tempfile

p=argparse.ArgumentParser()
p.add_argument('--source',type=Path,default=Path(__file__).resolve().parent)
p.add_argument('--destination',type=Path,required=True)
a=p.parse_args(); source=a.source.resolve(); destination=a.destination.resolve()
assert not destination.exists(), 'Immutable evidence directory already exists'
evidence=destination/'evidence'; evidence.mkdir(parents=True)
def sha(b): return hashlib.sha256(b).hexdigest()
def write(path,value): path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+chr(10))
copied=[]; filtered=[]
def copy(origin,relative):
    b=origin.read_bytes(); compressed=relative.suffix in ('.json','.jsonl','.txt') and len(b)>50000
    target=Path(str(relative)+'.gz') if compressed else relative
    out=evidence/target; out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(gzip.compress(b,mtime=0) if compressed else b)
    assert (gzip.decompress(out.read_bytes()) if compressed else out.read_bytes())==b
    copied.append(dict(source_absolute=str(origin),source_size=len(b),source_sha256=sha(b),
        archived_relative=str(target),archived_size=out.stat().st_size,archived_sha256=sha(out.read_bytes()),
        compression='gzip' if compressed else 'none',roundtrip_verified=True))
phases=('diagnostic-unlocked-mid-mesh','diagnostic-mid-mesh-active','diagnostic-mid-mesh-unlocked-final','diagnostic-mid-mesh-finder-reopen')
for phase in phases:
    base=source/'candidate0110'/phase
    for origin in sorted(base.iterdir()):
        assert origin.is_file()
        relative=origin.relative_to(source)
        if origin.name=='attribution.json':
            b=origin.read_bytes(); rows=json.loads(b); admitted=[x for x in rows if x['admitted']]
            assert len(admitted)==8 and all(x['resource_coalition']['bundle ID']=='com.gzers.opencodex.desktop.perf0110oct12' for x in admitted)
            value=dict(kind='admitted_only_attribution',original_relative=str(relative),original_size=len(b),
                original_sha256=sha(b),original_entry_count=len(rows),admitted_entry_count=len(admitted),
                rejected_entry_count=len(rows)-len(admitted),filter='Keep only exact admitted test resource-coalition records; unrelated rejected app metadata excluded',entries=admitted)
            out=evidence/relative.with_name('attribution-admitted.json'); out.parent.mkdir(parents=True,exist_ok=True); write(out,value)
            filtered.append({k:v for k,v in value.items() if k!='entries'})
        else: copy(origin,relative)
for name in ('measure.py','run_scene.py','archive_midmesh.py','recompute_midmesh.py'):
    copy(source/name,Path('methods')/name)
monitor_root=Path('/private/tmp/ocx-0110-deb740-foreground')
copy(monitor_root/'foreground-monitor-routes.swift',Path('methods/foreground-monitor-routes.swift'))
binary=monitor_root/'foreground-monitor-routes'
assert sha(binary.read_bytes())=='e6789bc23754d19da7cffcf9c605652dfc335c221b81ae6fde58ddaef9e42a05'
write(evidence/'source-map.json',dict(kind='bounded_archive_provenance',external_source_root=str(source),
    lossless_sources=copied,filtered_derivatives=filtered,
    binary_digest_only=dict(path=str(binary),size=binary.stat().st_size,sha256=sha(binary.read_bytes())),
    excluded=['Application bundles and binaries','User payload, test-data, backup and WebView profile','Unrelated rejected attribution app metadata','Unrelated working-tree guard'],
    notes=['All source files retained as raw bytes or reversible gzip; invalid-envelope and preparation records not repaired',
        'settings-end.json is valid JSON and preserved without mutation',
        'Context appearance high is stale; actual mid settings validated separately',
        'Attribution derivative is explicitly filtered, not a lossless source copy']))
with tempfile.TemporaryDirectory(prefix='ocx-midmesh-offline-') as scratch:
    first=Path(scratch)/'external.json'; second=Path(scratch)/'archived.json'
    outputs=[]
    for input_root,target in ((source,first),(evidence,second)):
        r=subprocess.run([sys.executable,str(evidence/'methods/recompute_midmesh.py'),'--root',str(input_root),'--output',str(target)],capture_output=True,text=True,timeout=20)
        assert r.returncode==0,(r.stdout,r.stderr)
        outputs.append(dict(returncode=r.returncode,stdout=r.stdout,stderr=r.stderr))
    original=json.loads(first.read_bytes()); archived=json.loads(second.read_bytes())
    original.pop('inputs_sha256'); archived.pop('inputs_sha256')
    assert original==archived, 'Independent source/archive diagnostic metrics differ'
    (evidence/'diagnostic-recompute.json').write_bytes(second.read_bytes())
items=[(f.relative_to(evidence),sha(f.read_bytes()),f.stat().st_size) for f in sorted(evidence.rglob('*')) if f.is_file()]
manifest=chr(10).join(f'{digest}  {relative}' for relative,digest,size in items)+chr(10)
(evidence/'SHA256SUMS').write_text(manifest)
for relative,digest,size in items:
    b=(evidence/relative).read_bytes(); assert sha(b)==digest and len(b)==size
verification=dict(captured_at=dt.datetime.now(dt.timezone.utc).isoformat(),result='passed',
    manifest_item_count=len(items),lossless_source_count=len(copied),filtered_derivative_count=len(filtered),
    gzip_roundtrip_count=sum(x['compression']=='gzip' for x in copied),evidence_bytes=sum(x[2] for x in items),
    sha256sums_sha256=sha((evidence/'SHA256SUMS').read_bytes()),
    offline_recomputation=dict(source_and_archived_metrics_equal=True,runs=outputs),
    gate='archive_integrity_only; interrupted diagnostic is not acceptance')
write(destination/'archive-verification.json',verification)
print(json.dumps({k:v for k,v in verification.items() if k!='offline_recomputation'},indent=2))
