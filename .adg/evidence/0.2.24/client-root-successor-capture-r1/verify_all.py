from pathlib import Path
import hashlib, json, re, subprocess, sys

root = Path(__file__).resolve().parent
manifest = json.loads((root / 'manifest.json').read_text())
expected = set()
for item in manifest['files']:
    path = Path(item['path'])
    assert not path.is_absolute() and '..' not in path.parts
    assert item['path'] not in expected
    expected.add(item['path'])
    raw = (root / path).read_bytes()
    assert len(raw) == item['bytes'] and hashlib.sha256(raw).hexdigest() == item['sha256'], item['path']
actual = {str(path.relative_to(root)) for path in root.rglob('*') if path.is_file()}
assert actual - {'manifest.json', 'verified-evidence.json'} == expected
all_names = set()
for name in ['capture-delivery', 'index-regression', 'root-regression', 'first-originals-regression', 'tree-regression']:
    result = json.loads((root / (name + '.json')).read_text())
    assert result['exitCode'] == 0
    log = (root / (name + '.log')).read_text()
    rows = re.findall(r'^test (\S+) \.\.\. (.*)$', log, re.M)
    names = {name for name, status in rows if not status.startswith('ignored')}
    summaries = re.findall(r'test result: ok\. (\d+) passed; (\d+) failed; (\d+) ignored;', log)
    assert len(names) == int(summaries[-1][0])
    all_names |= names
assert len(all_names) == 112
for name in ['strict-clippy', 'formatting']:
    assert json.loads((root / (name + '.json')).read_text())['exitCode'] == 0
assert json.loads((root / 'successor-initial.json').read_text())['exitCode'] == 101
assert 'Permission denied' in (root / 'successor-initial.log').read_text()
build = json.loads((root / 'source-build.json').read_text())
final = json.loads((root / 'source-final.json').read_text())
assert final['files'] == build['files'] and len(final['files']) == 7
assert final['actualLibraryTestBinarySha256'] == build['actualLibraryTestBinarySha256']
assert final['committedSource'] == manifest['sourceCommit']
raw = json.loads(subprocess.check_output([sys.executable, str(root / 'verify_raw.py')], text=True))
assert raw == json.loads((root / 'verified-raw.json').read_text())
cleanup = json.loads((root / 'cleanup.json').read_text())
assert cleanup['temporaryRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval']
print(json.dumps({'accepted': False, 'manifestFiles': len(expected), 'manifestBytes': sum(v['bytes'] for v in manifest['files']),
                  'uniquePassingFunctions': len(all_names), 'rawExitScenes': len(raw['verified']),
                  'sourceCommit': manifest['sourceCommit'], 'retainedInitialFailure': True,
                  'scope': 'synthetic capture/index facts only; no production next Start or fixed candidate acceptance'}, indent=2))
