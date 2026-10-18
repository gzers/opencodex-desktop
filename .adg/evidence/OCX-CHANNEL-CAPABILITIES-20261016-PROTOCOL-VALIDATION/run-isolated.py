import json
from pathlib import Path
import shutil
import subprocess
import tempfile

HERE = Path(__file__).resolve().parent
PACKAGE = Path('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/runtime/opencodex/node_modules/@bitkyc08/opencodex')
CODEX = Path('/Applications/ChatGPT.app/Contents/Resources/codex-cli/CodexCLI.app/Contents/MacOS/codex')
BUN = PACKAGE / 'node_modules/bun/bin/bun.exe'

with tempfile.TemporaryDirectory(prefix='ocx-channel-validation-') as temp:
    sandbox = Path(temp)
    dirs = {name: sandbox / name for name in ['home', 'codex', 'opencodex', 'tmp', 'cache', 'config', 'data']}
    for path in dirs.values():
        path.mkdir()
    env = {'HOME': str(dirs['home']), 'CODEX_HOME': str(dirs['codex']), 'OPENCODEX_HOME': str(dirs['opencodex']),
           'TMPDIR': str(dirs['tmp']), 'XDG_CACHE_HOME': str(dirs['cache']), 'XDG_CONFIG_HOME': str(dirs['config']),
           'XDG_DATA_HOME': str(dirs['data']), 'PATH': '/usr/bin:/bin', 'LANG': 'en_US.UTF-8', 'LC_ALL': 'en_US.UTF-8',
           'OCX_PROBE_PACKAGE': str(PACKAGE), 'OCX_PROBE_RESULT': str(sandbox / 'runtime-results.json')}
    version = subprocess.run([str(CODEX), '--version'], cwd=sandbox, env=env, capture_output=True, text=True, timeout=15)
    cases = {
        'baseline': [],
        'async-enabled': ['-c', 'features.async_compaction=true'],
        'history-notes-enabled': ['-c', 'features.context_management=true', '-c', 'features.token_budget.enabled=true', '-c', 'features.token_budget.use_history_notes_extension=true'],
        'all-enabled-parse-only': ['-c', 'features.async_compaction=true', '-c', 'features.context_management=true', '-c', 'features.token_budget.enabled=true'],
        'remote-v2-parse-only': ['-c', 'model_providers.fixture.name="Fixture"', '-c', 'model_providers.fixture.base_url="http://127.0.0.1:9/v1"', '-c', 'model_providers.fixture.capabilities.remote_compaction="v2"'],
    }
    observations = []
    for name, flags in cases.items():
        result = subprocess.run([str(CODEX), 'features', 'list', *flags], cwd=sandbox, env=env, capture_output=True, text=True, timeout=20)
        selected = [line for line in result.stdout.splitlines() if any(term in line for term in ['async_compaction', 'context_management', 'token_budget', 'fast_mode'])]
        observations.append({'name': name, 'exit_code': result.returncode, 'selected_features': selected, 'stderr': result.stderr[:5000],
                             'meaning': 'Parsing and feature listing only; unknown keys may be ignored.'})
    (HERE / 'client-observations.json').write_text(json.dumps({'version': version.stdout.strip(), 'version_exit_code': version.returncode,
        'cases': observations, 'scope': 'No agent turn or provider request'}, ensure_ascii=False, indent=2) + '\n')
    probe = subprocess.run([str(BUN), str(HERE / 'runtime-probe.mjs')], cwd=sandbox, env=env, capture_output=True, text=True, timeout=60)
    if Path(env['OCX_PROBE_RESULT']).exists():
        shutil.copyfile(env['OCX_PROBE_RESULT'], HERE / 'runtime-results.json')
    (HERE / 'probe-process.json').write_text(json.dumps({'exit_code': probe.returncode, 'stdout': probe.stdout[:12000], 'stderr': probe.stderr[:12000],
        'environment': 'Temporary HOME, CODEX_HOME, OPENCODEX_HOME and XDG paths; no inherited credentials or proxy variables.',
        'temporary_files_removed_on_exit': True}, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps({'client_version': version.stdout.strip(), 'client_cases': [{k: row[k] for k in ['name', 'exit_code', 'selected_features']} for row in observations],
        'probe_exit_code': probe.returncode, 'probe_stdout': probe.stdout[:6000], 'probe_stderr': probe.stderr[:6000]}, ensure_ascii=False, indent=2))
    if probe.returncode:
        raise SystemExit(probe.returncode)
