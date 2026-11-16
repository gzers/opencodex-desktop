from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
sha=lambda b:hashlib.sha256(b).hexdigest()
rows=[]
for name in ['library02','library04']:
    checked=json.loads((ROOT/f'native-traces-{name}.json').read_text(encoding='utf-8'))
    for observation in checked['observations']:
        for i,archive in enumerate(observation['actualOriginalArchives']):
            path=Path(archive['directory'])/'snapshot.json'
            if not path.exists():continue
            raw=path.read_bytes();metadata=json.loads(raw)
            source=metadata['source']
            # serde_json::Value recursively orders object keys; these frames
            # contain only strings/bools/integers/arrays, no floating ambiguity.
            digest=sha(json.dumps(source,ensure_ascii=False,sort_keys=True,separators=(',',':')).encode())
            assert digest==metadata['context']['originalFrames'][i]
            rows.append(dict(run=name,fault=observation['observation']['fault'],path=str(path),metadataSha256=sha(raw),
                sourceFrameSha256=digest,boundOriginalFrame=metadata['context']['originalFrames'][i],sourceOriginalNodes=len(source['nodes'])))
assert rows
with (ROOT/'archive-source-frame-digest-check.json').open('x',encoding='utf-8') as out:
    json.dump(dict(frames=rows,canonicalValueHashEqual=True,offlineDataOnly=True,nativeAuthority=False),out,indent=2)
print(json.dumps(dict(originalFrames=len(rows),allExactCanonicalFrameDigests=True)))
