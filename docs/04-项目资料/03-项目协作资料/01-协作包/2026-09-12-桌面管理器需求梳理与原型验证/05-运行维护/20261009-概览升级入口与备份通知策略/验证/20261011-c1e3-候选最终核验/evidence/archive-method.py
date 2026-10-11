from pathlib import Path
import datetime
import gzip
import hashlib
import json

repo = Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop')
source = Path('/private/tmp/ocx-0110-c1e3-candidate-hqjf10k6')
collab = repo / 'docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略'
record = collab / '验证/20261011-c1e3-候选最终核验'
dest = record / 'evidence'
assert not record.exists(), 'Archive destination must be new'
dest.mkdir(parents=True)
sha = lambda b: hashlib.sha256(b).hexdigest()
selected = [p for p in source.iterdir() if p.is_file()]
selected += [p for p in (source / 'native').iterdir() if p.is_file()]
selected += [p for p in (source / 'ci-candidate').iterdir() if p.is_file()]
selected += [p for p in (source / 'windows').iterdir() if p.is_file() and p.suffix != '.zip']
runner = source / 'windows/runner-evidence'
selected += [runner / n for n in ['environment.json', 'installer-verification.json', 'msi-install.log', 'msi-uninstall.log', 'cargo-tree.txt']]
for kind in ['msi', 'nsis']:
    selected += [p for p in (runner / f'{kind}-startup').iterdir() if p.is_file()]
provenance = []
for p in sorted(selected):
    rel = p.relative_to(source)
    data = p.read_bytes()
    mode = 'byte-identical'
    if p.suffix == '.image':
        assert data.startswith(b'\xff\xd8\xff'), f'Expected JPEG: {rel}'
        outrel = rel.with_suffix('.jpg')
        out = data
        mode = 'extension-only .image to .jpg; no conversion'
    elif p.name.endswith(('.ax.txt', '.log', '.stdout', '.stderr')) or p.name == 'cargo-tree.txt':
        outrel = Path(str(rel) + '.gz')
        out = gzip.compress(data, mtime=0)
        mode = 'lossless gzip; mtime=0'
    else:
        outrel = rel
        out = data
    target = dest / outrel
    target.parent.mkdir(parents=True, exist_ok=True)
    with target.open('xb') as f:
        f.write(out)
    provenance.append({'source': str(rel), 'source_bytes': len(data), 'source_sha256': sha(data), 'archive': str(outrel), 'archive_bytes': len(out), 'archive_sha256': sha(out), 'transformation': mode})
digest_only = []
paths = list((source / 'original').rglob('*'))
paths += list((source / 'windows/installers').rglob('*'))
paths += [source / 'windows/installers.zip', source / 'windows/runner-evidence.zip', runner / 'cargo-metadata.json', runner / 'Cargo.lock']
for p in sorted(paths):
    if p.is_file() and (p.suffix in {'.dmg', '.msi', '.exe', '.zip'} or p.name in {'opencodex-desktop', 'ocxd', 'cargo-metadata.json', 'Cargo.lock', 'installer-verification.json'}):
        b = p.read_bytes()
        digest_only.append({'source': str(p.relative_to(source)), 'bytes': len(b), 'sha256': sha(b), 'retention': 'digest only; not copied'})
payload = {'source_commit': 'c1e3c09d8ed33238543ce5943e439991b8099088', 'source_root': str(source), 'archived_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'selection': 'Top-level receipts/logs/method, native 00-24 including unsuccessful attempts, CI commands, Windows artifact receipts and selected runner diagnostics only. No binary, ZIP, user payload or WebView profile copy.', 'sources': provenance, 'digest_only': digest_only}
(dest / 'source-provenance.json').write_text(json.dumps(payload, ensure_ascii=False, indent=2) + '\n')
(dest / 'archive-method.py').write_bytes(Path(__file__).read_bytes())
files = sorted(p for p in dest.rglob('*') if p.is_file())
(dest / 'SHA256SUMS').write_text(''.join(f'{sha(p.read_bytes())}  {p.relative_to(dest)}\n' for p in files))
for item in provenance:
    sb = (source / item['source']).read_bytes()
    ab = (dest / item['archive']).read_bytes()
    assert len(sb) == item['source_bytes'] and sha(sb) == item['source_sha256']
    assert len(ab) == item['archive_bytes'] and sha(ab) == item['archive_sha256']
    decoded = gzip.decompress(ab) if item['transformation'].startswith('lossless') else ab
    assert decoded == sb
for line in (dest / 'SHA256SUMS').read_text().splitlines():
    digest, name = line.split('  ', 1)
    assert sha((dest / name).read_bytes()) == digest
verification = {'verified_at_utc': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'source_commit': payload['source_commit'], 'archive_files_verified': len(files), 'sources_verified': len(provenance), 'digest_only_files': len(digest_only), 'archive_bytes': sum(p.stat().st_size for p in dest.rglob('*') if p.is_file()), 'roundtrip_byte_match': True, 'all_checksums_match': True, 'release_accepted': False}
(record / 'archive-verification.json').write_text(json.dumps(verification, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(verification, ensure_ascii=False))
