import json
import pathlib
import subprocess
import sys
import time

root = pathlib.Path(__file__).resolve().parent
folder, label = sys.argv[1:3]
directory = root / folder / label
directory.mkdir(parents=True)
arguments = ['git', '-c', 'http.proxy=http://127.0.0.1:3060', '-c', 'http.version=HTTP/1.1',
             '-c', 'http.sslBackend=openssl', '-c', 'http.lowSpeedLimit=1', '-c', 'http.lowSpeedTime=16',
             *sys.argv[3:]]
start = time.monotonic()
with (directory / 'stdout.log').open('xb') as out, (directory / 'stderr.log').open('xb') as err:
    completed = subprocess.run(arguments, stdout=out, stderr=err)
result = {'arguments': arguments, 'raw': completed.returncode, 'elapsedMs': round((time.monotonic() - start) * 1000)}
(directory / 'result.json').write_text(json.dumps(result, indent=2) + '\n', encoding='utf-8')
for name in ('stdout.log', 'stderr.log'):
    print((directory / name).read_text(encoding='utf-8', errors='replace'), end='')
sys.exit(completed.returncode)
