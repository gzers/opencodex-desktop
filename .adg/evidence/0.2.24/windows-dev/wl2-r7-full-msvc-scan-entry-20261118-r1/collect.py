from pathlib import Path
import hashlib
import json
import subprocess

root = Path(__file__).resolve().parent
code = Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
read = lambda p: json.loads(p.read_text(encoding='utf-8-sig'))
fixed = read(root / 'fixed-after/inputs.json')
results = read(root / 'fixed-after/result.json')
first = read(root / 'result.json')
phases = read(root / 'phases/result.json')
product = read(root / 'product-scanner/result.json')
sha = fixed['code']
assert sha == subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=code).decode().strip()
assert not subprocess.check_output(['git', 'status', '--porcelain'], cwd=code)
paths = subprocess.check_output(['git', 'diff-tree', '--no-commit-id', '--name-only', '-r', sha], cwd=code).decode().splitlines()
assert set(paths) == {x['path'] for x in fixed['sourceFiles'] if x['path'].endswith('.rs')}
for row in fixed['sourceFiles']:
    archive = (code / row['path']).read_bytes()
    assert hashlib.sha256(archive).hexdigest() == row['sha256']
    git = subprocess.check_output(['git', 'show', f'{sha}:{row["path"]}'], cwd=code)
    assert archive.replace(b'\r\n', b'\n') == git
assert first['raw'] == 101 and not first['fixtureReport']['passed']
assert results['raw'] == phases['raw'] == product['raw'] == 0
assert results['fixtureReport']['completeInventoryAndManifestReadback']
assert results['fixtureReport']['elapsedUs'] <= 10_000_000
previous = read(root.parent / 'wl2-r7-bound-preparation-20261118-r1/status.json')['matrix']
assert len(previous) == 179
with (root / 'status.json').open('x', encoding='utf-8') as out:
    matrix = [
        dict(item='完整 MSVC 制品首次全树与 manifest 原10秒入口', status='raw101, 10079.394 ms, runtime_image_unconfirmed, 失败阶段无记录'),
        dict(item='原完整 MSVC 制品独立 reference/native 1/4 worker 计量', status='raw0, 12213条目, 623970751字节, 全部 rows/digest一致, 无总 deadline'),
        dict(item='缓冲区改动完整 MSVC 构建和 Windows 扫描回归', status='raw0, 16 passed, 3 explicit ignored'),
        dict(item='改动后同固定副本原10秒入口全树与 manifest', status='raw0, 2634.174 ms，自有固定副本'),
        dict(item='r7只读回执重新核对', status='10个文件与既有原证据匹配，现行r2计划仍 executionReady=false'),
        dict(item='本轮清理', status='无新增测试根，生成schema归档，代码树干净，旧拒删保留'),
    ]
    json.dump(dict(matrix=previous+matrix, previousEntries=179, source=sha, sourceFilesVerified=fixed['sourceFiles'], changedPaths=paths, productionAccepted=False, originalR2Touched=False, publicStartOpen=False), out, indent=2, ensure_ascii=False)
