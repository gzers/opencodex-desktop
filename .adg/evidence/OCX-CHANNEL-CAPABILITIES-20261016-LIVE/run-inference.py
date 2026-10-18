import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('live_probe', HERE / 'probe.py')
p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p)
base = {'model': 'gpt-6.1-sol', 'instructions': 'This is an isolated compatibility check. Reply with the requested marker only.',
        'input': [{'role': 'user', 'content': [{'type': 'input_text', 'text': 'Reply exactly: ocx-probe-ok'}]}],
        'stream': True, 'store': False, 'max_output_tokens': 128, 'reasoning': {'effort': 'low'}}
p.call('proxy-standard-inference', 'http://127.0.0.1:10100/v1/responses', {**base, 'model': 'Krill-1/gpt-6.1-sol', 'service_tier': 'default'}, upstream=False)
p.call('proxy-ultrafast-inference', 'http://127.0.0.1:10100/v1/responses', {**base, 'model': 'Krill-1/gpt-6.1-sol', 'service_tier': 'ultrafast'}, upstream=False)
p.call('tutorial-ultrafast-inference', 'https://api.krill-code.net/backend-api/codex/responses', {**base, 'service_tier': 'ultrafast'})
