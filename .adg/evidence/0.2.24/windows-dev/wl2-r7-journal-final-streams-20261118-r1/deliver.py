from pathlib import Path
import json
import os
import subprocess
import sys


ROOT = Path(__file__).resolve().parent
NAME, CHECKOUT, BRANCH, ATTEMPT = sys.argv[1:]
REF = f'refs/heads/{BRANCH}'
OUT = ROOT / f'delivery-{NAME}' / ATTEMPT
OUT.mkdir(parents=True)
GIT = ['git', '-c', 'http.proxy=http://127.0.0.1:3060', '-c', 'http.version=HTTP/1.1',
       '-c', 'http.lowSpeedLimit=1', '-c', 'http.lowSpeedTime=10']
ENV = dict(os.environ, GIT_TERMINAL_PROMPT='0')
head = subprocess.check_output(['git', 'rev-parse', 'HEAD'], cwd=CHECKOUT, text=True).strip()


def run(name, args):
    with (OUT / f'{name}.stdout.log').open('xb') as stdout, (OUT / f'{name}.stderr.log').open('xb') as stderr:
        try:
            return subprocess.run(GIT + args, cwd=CHECKOUT, env=ENV, stdout=stdout, stderr=stderr, timeout=45).returncode
        except subprocess.TimeoutExpired:
            return 'timeout'


push = run('push', ['push', 'origin', f'{head}:{REF}'])
readback = run('readback', ['ls-remote', '--exit-code', 'origin', REF]) if push == 0 else None
expected = f'{head}\t{REF}'
verified = readback == 0 and (OUT / 'readback.stdout.log').read_text(encoding='utf-8').strip() == expected
result = dict(commit=head, ref=REF, pushRaw=push, readbackRaw=readback, exactRemoteMatch=verified)
payload = json.dumps(result, indent=2) + '\n'
(OUT / 'result.json').write_text(payload, encoding='utf-8', newline='\n')
print(payload, end='')
sys.exit(0 if verified else 1)
