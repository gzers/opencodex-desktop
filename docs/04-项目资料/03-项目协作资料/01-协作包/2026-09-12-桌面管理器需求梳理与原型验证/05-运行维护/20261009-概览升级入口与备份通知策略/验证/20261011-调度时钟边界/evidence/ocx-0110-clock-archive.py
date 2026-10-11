from pathlib import Path
from datetime import datetime, timezone
import gzip
import hashlib
import json
import re
import subprocess

repo = Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop')
source = Path('/private/tmp/ocx-0110-clock-audit')
code = Path('/Users/ezio/.codex/worktrees/ocx-0110-main')
collab = Path('docs/04-项目资料/03-项目协作资料/01-协作包/2026-09-12-桌面管理器需求梳理与原型验证/05-运行维护/20261009-概览升级入口与备份通知策略')
record = collab / '验证/20261011-调度时钟边界'
dest = repo / record / 'evidence'
sha = lambda data: hashlib.sha256(data).hexdigest()
identity = json.loads((source / 'source-identity.json').read_text())
candidate = json.loads((source / 'candidate-verification.json').read_text())
assert identity['source_sha'] == 'deb740c09857b5f0bef7c9b4090fe5e8587bb4b3'
assert candidate['source_sha'] == identity['source_sha']
assert subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=code).decode().strip() == identity['source_sha']
assert subprocess.check_output(['git', 'diff', '--name-only'], cwd=code) == b''
assert subprocess.check_output(['git', 'diff', '--cached', '--name-only'], cwd=repo) == b''
before = json.loads((source / 'docs-dirty-before.json').read_text())
for name, old in before.items():
    p = repo / name
    now = {'exists': p.exists(), 'sha256': sha(p.read_bytes()) if p.exists() else None}
    assert now == old, ('unrelated path changed', name)
assert not (repo / record).exists()
dest.mkdir(parents=True)

selected = ['source-identity.json', 'candidate-verification.json', 'macos-build.json', 'macos-build.raw.log', 'ci-list.json', 'ci-list.stderr.log', 'ci-status-latest.json']
for stem in ['ui-red', 'rust-red', 'ui-green', 'rust-green-command', 'ui-full', 'rust-full', 'rust-fmt', 'rust-clippy', 'ui-build', 'code-diff-check', 'code-commit', 'code-push']:
    selected += [stem + '.json', stem + '.log']
selected += ['macos-file.log', 'macos-dmg-verify.log', 'macos-signature-verify.log', 'macos-signature-display.log', 'macos-signing-identities.log']
sources = []
for name in selected:
    p = source / name
    data = p.read_bytes()
    if p.suffix == '.json':
        json.loads(data)
    archived_name = name + '.gz' if p.suffix == '.log' else name
    archived = gzip.compress(data, mtime=0) if p.suffix == '.log' else data
    (dest / archived_name).write_bytes(archived)
    sources.append({'source': name, 'source_bytes': len(data), 'source_sha256': sha(data), 'archive': archived_name, 'archive_bytes': len(archived), 'archive_sha256': sha(archived), 'transformation': 'lossless gzip; mtime=0' if p.suffix == '.log' else 'byte-identical'})
for helper in [Path('/private/tmp/ocx-0110-clock-collect.py'), Path(__file__)]:
    data = helper.read_bytes()
    (dest / helper.name).write_bytes(data)
    sources.append({'source': str(helper), 'source_bytes': len(data), 'source_sha256': sha(data), 'archive': helper.name, 'archive_bytes': len(data), 'archive_sha256': sha(data), 'transformation': 'byte-identical'})

rust = (source / 'rust-full.log').read_text()
suites = re.findall(r'test result: ok[.] ([0-9]+) passed; ([0-9]+) failed; ([0-9]+) ignored;', rust)
totals = {'suites': len(suites), 'passed': sum(int(x[0]) for x in suites), 'failed': sum(int(x[1]) for x in suites), 'ignored': sum(int(x[2]) for x in suites)}
assert totals == {'suites': 34, 'passed': 866, 'failed': 0, 'ignored': 6}, totals
assert '95 passed (95)' in (source / 'ui-full.log').read_text()
assert '474 passed (474)' in (source / 'ui-full.log').read_text()
results = {'source_sha': identity['source_sha'], 'ui_full': {'files': 95, 'passed': 474, 'failed': 0}, 'rust_full': totals, 'ui_targeted': {'files': 4, 'passed': 24}, 'rust_commands_targeted': {'passed': 4, 'filtered_out': 699}, 'ui_initial_red': {'passed': 7, 'failed': 2}, 'rust_initial_red': {'passed': 0, 'failed': 3}, 'not_native_or_performance_acceptance': True, 'early_manifests_have_no_start_time_or_elapsed_measurement': True}
(dest / 'test-results.json').write_text(json.dumps(results, ensure_ascii=False, indent=2) + chr(10))
digest_only = []
for item in candidate['assets']:
    p = source / item['path']
    assert p.stat().st_size == item['bytes'] and sha(p.read_bytes()) == item['sha256']
    digest_only.append({**item, 'retention': 'digest only; not copied to docs'})
provenance = {'source_sha': identity['source_sha'], 'source_root': str(source), 'archived_at_utc': datetime.now(timezone.utc).isoformat(), 'selection': 'Named red/green test outputs, command manifests, source identity, code commit/push, macOS build/verification and CI snapshots. Raw logs are lossless gzip. No application binary, source patch, user payload, cache, profile or unrelated document dirty snapshot copied.', 'sources': sources, 'digest_only': digest_only, 'unrelated_tracked_paths_verified': len(before), 'code_before_snapshot_not_a_clean_baseline': True}
(dest / 'source-provenance.json').write_text(json.dumps(provenance, ensure_ascii=False, indent=2) + chr(10))
files = sorted(p for p in dest.iterdir() if p.is_file())
(dest / 'SHA256SUMS').write_text(''.join(f'{sha(p.read_bytes())}  {p.name}' + chr(10) for p in files))
for item in sources:
    original_path = Path(item['source'])
    if not original_path.is_absolute(): original_path = source / original_path
    raw = original_path.read_bytes()
    packed = (dest / item['archive']).read_bytes()
    assert sha(raw) == item['source_sha256']
    assert sha(packed) == item['archive_sha256']
    assert (gzip.decompress(packed) if item['transformation'].startswith('lossless') else packed) == raw
for line in (dest / 'SHA256SUMS').read_text().splitlines():
    digest, name = line.split('  ', 1)
    assert sha((dest / name).read_bytes()) == digest
verification = {'verified_at_utc': datetime.now(timezone.utc).isoformat(), 'source_sha': identity['source_sha'], 'archive_files_verified': len(files), 'sources_verified': len(sources), 'digest_only_files': len(digest_only), 'roundtrip_byte_match': True, 'all_checksums_match': True, 'unrelated_tracked_paths_unchanged': len(before), 'release_accepted': False}
(repo / record / 'archive-verification.json').write_text(json.dumps(verification, ensure_ascii=False, indent=2) + chr(10))
print(json.dumps(verification, ensure_ascii=False))
