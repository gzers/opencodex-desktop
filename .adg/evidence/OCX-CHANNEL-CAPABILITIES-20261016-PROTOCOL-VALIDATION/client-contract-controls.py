import json
from pathlib import Path
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
CODEX = '/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex'
cases = [
    ('remote-local-valid', 'model_providers.fixture.capabilities.remote_compaction="unsupported"', True),
    ('remote-protocol-invalid-control', 'model_providers.fixture.capabilities.remote_compaction="fixture-invalid"', False),
    ('async-type-invalid-control', 'features.async_compaction="fixture-invalid"', False),
]
observed = []
with tempfile.TemporaryDirectory(prefix='ocx-channel-validation-') as temp:
    root = Path(temp)
    for name in ['home', 'codex', 'tmp']:
        (root / name).mkdir()
    env = {'HOME': str(root / 'home'), 'CODEX_HOME': str(root / 'codex'), 'TMPDIR': str(root / 'tmp'),
           'PATH': '/usr/bin:/bin', 'LANG': 'en_US.UTF-8'}
    for name, assignment, accepted in cases:
        result = subprocess.run([CODEX, 'features', 'list', '-c', 'model_providers.fixture.name="Fixture"',
                                 '-c', 'model_providers.fixture.base_url="http://127.0.0.1:9/v1"', '-c', assignment], cwd=root, env=env,
                                capture_output=True, text=True, timeout=20)
        passed = (result.returncode == 0) == accepted
        observed.append({'name': name, 'exit_code': result.returncode, 'expected_accepted': accepted,
                         'result': 'passed' if passed else 'failed', 'stderr': result.stderr[:5000]})
output = {'kind': 'isolated-client-config-contract-controls', 'cases': observed,
          'passed': sum(x['result'] == 'passed' for x in observed),
          'failed': sum(x['result'] == 'failed' for x in observed),
          'scope': 'Config parsing only; no model turn, login or provider request.'}
(HERE / 'client-contract-results.json').write_text(json.dumps(output, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(output, ensure_ascii=False, indent=2))
raise SystemExit(0 if output['failed'] == 0 else 1)
