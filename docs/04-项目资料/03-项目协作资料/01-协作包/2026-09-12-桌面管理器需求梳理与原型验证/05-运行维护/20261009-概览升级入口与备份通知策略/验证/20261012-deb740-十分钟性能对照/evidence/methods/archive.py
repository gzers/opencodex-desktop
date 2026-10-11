"""Archive selected test evidence, filter unrelated attribution, verify roundtrip."""
import argparse
import datetime
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
a=p.parse_args();source=a.source.resolve();destination=a.destination.resolve()
assert not destination.exists(), 'Refuse to overwrite an existing immutable evidence directory'
evidence=destination/'evidence';evidence.mkdir(parents=True)
def sha(data): return hashlib.sha256(data).hexdigest()
def write_json(path,value): path.write_text(json.dumps(value,ensure_ascii=False,indent=2)+chr(10))
sources=[];filtered=[]
def copy(relative,target=None):
    origin=source/relative;raw=origin.read_bytes()
    target=Path(target or relative)
    if target.suffix=='.png':
        assert raw[:3]==bytes([255,216,255]), 'Expected JPEG bytes in PNG-named source'
        target=target.with_suffix('.jpg')
    compressed=target.suffix in ('.txt','.jsonl') and len(raw)>50000
    if compressed: target=Path(str(target)+'.gz')
    out=evidence/target;out.parent.mkdir(parents=True,exist_ok=True)
    out.write_bytes(gzip.compress(raw,mtime=0) if compressed else raw)
    restored=gzip.decompress(out.read_bytes()) if compressed else out.read_bytes()
    assert restored==raw
    sources.append(dict(source_relative=str(relative),source_size=len(raw),source_sha256=sha(raw),archived_relative=str(target),archived_size=out.stat().st_size,archived_sha256=sha(out.read_bytes()),compression='gzip' if compressed else 'none',roundtrip_verified=True,extension_renamed=Path(relative).suffix=='.png'))
for prefix in ('baseline019/idle-high','candidate0110/idle-high','candidate0110/diagnostic-settings'):
    for origin in sorted((source/prefix).iterdir()):
        assert origin.is_file(), 'Only explicit phase files are admitted'
        relative=origin.relative_to(source)
        if origin.name=='attribution.json':
            raw=origin.read_bytes();rows=json.loads(raw)
            admitted=[x for x in rows if x['admitted']]
            bundle=json.loads((source/'candidates.json').read_bytes())[Path(prefix).parts[0]]['bundle']
            assert len(admitted)==8 and all(x['resource_coalition']['bundle ID']==bundle for x in admitted)
            derived=dict(kind='admitted_only_attribution',original_relative=str(relative),original_size=len(raw),original_sha256=sha(raw),original_entry_count=len(rows),admitted_entry_count=len(admitted),rejected_entry_count=len(rows)-len(admitted),filter='Only admitted exact test application resource-coalition records retained; unrelated app records intentionally excluded.',entries=admitted)
            target=evidence/relative.with_name('attribution-admitted.json');target.parent.mkdir(parents=True,exist_ok=True);write_json(target,derived)
            filtered.append({k:v for k,v in derived.items() if k!='entries'})
        else:
            copy(relative)
for relative in ('candidates.json','comparison.json','final-process-snapshot.json','process-snapshot-writeback.json','baseline019/adhoc-sign.json','candidate0110/adhoc-sign.json'):
    copy(relative)
for relative in ('measure.py','recompute.py','archive.py'):
    copy(relative,Path('methods')/relative)
write_json(evidence/'source-map.json',dict(kind='bounded_archive_provenance',external_source_root=str(source),lossless_sources=sources,filtered_derivatives=filtered,excluded=['Application binaries and bundles','test-data payloads, backups and WebView profiles','Unrelated rejected WebKit app metadata','docs-before.json unrelated working-tree path guard'],notes=['PNG-named source screenshots contain JPEG bytes; only extension changed.','Original attribution remains external; admitted-only derivatives are explicitly not lossless copies.','Logs are byte snapshots; the application may continue appending externally.']))
with tempfile.TemporaryDirectory(prefix='ocx-0110-offline-') as scratch:
    output=Path(scratch)/'comparison.json'
    result=subprocess.run([sys.executable,str(evidence/'methods/recompute.py'),'--root',str(evidence),'--output',str(output)],capture_output=True,text=True,timeout=20)
    assert result.returncode==0,(result.stdout,result.stderr)
    assert output.read_bytes()==(evidence/'comparison.json').read_bytes(), 'Archived-input recomputation drift'
    offline=dict(returncode=result.returncode,stdout=result.stdout,stderr=result.stderr,comparison_byte_equal=True)
items=[]
for path in sorted(evidence.rglob('*')):
    if path.is_file(): items.append((path.relative_to(evidence),sha(path.read_bytes()),path.stat().st_size))
manifest=chr(10).join(f'{digest}  {relative}' for relative,digest,size in items)+chr(10)
(evidence/'SHA256SUMS').write_text(manifest)
for relative,digest,size in items:
    raw=(evidence/relative).read_bytes();assert sha(raw)==digest and len(raw)==size
verification=dict(captured_at=datetime.datetime.now(datetime.timezone.utc).isoformat(),result='passed',manifest_item_count=len(items),lossless_source_count=len(sources),filtered_derivative_count=len(filtered),gzip_roundtrip_count=sum(x['compression']=='gzip' for x in sources),jpeg_extension_rename_count=sum(x['extension_renamed'] for x in sources),evidence_bytes=sum(x[2] for x in items),sha256sums_sha256=sha((evidence/'SHA256SUMS').read_bytes()),offline_recomputation=offline,gate='archive_integrity_only; performance acceptance not passed')
write_json(destination/'archive-verification.json',verification)
print(json.dumps({k:v for k,v in verification.items() if k!='offline_recomputation'},indent=2))
