import importlib.util
from pathlib import Path

HERE = Path(__file__).resolve().parent
spec = importlib.util.spec_from_file_location('live_probe', HERE / 'probe.py')
p = importlib.util.module_from_spec(spec)
spec.loader.exec_module(p)
history = [{'role': 'user', 'content': [{'type': 'input_text', 'text': 'Synthetic compatibility check. Remember the marker ocx-probe-ok and the chosen color cobalt. Next action: verify the marker. No files or external tools are involved.'}]},
           {'role': 'assistant', 'content': [{'type': 'output_text', 'text': 'Recorded the synthetic marker and decision.'}]}]
body = {'model': 'gpt-6.1-sol', 'input': history}
p.call('configured-direct-standalone-compact', 'https://api.krill-code.net/codex/v1/responses/compact', body)
obs, compacted = p.call('tutorial-direct-standalone-compact', 'https://api.krill-code.net/backend-api/codex/responses/compact', body, retain_output=True)
p.call('proxy-standalone-compact', 'http://127.0.0.1:10100/v1/responses/compact', {**body, 'model': 'Krill-1/gpt-6.1-sol'}, upstream=False)
ctx = {'context': {'session_id': p.state['session']}}
p.call('tutorial-history-list', 'https://api.krill-code.net/backend-api/codex/alpha/history/v2/list_windows', ctx)
p.call('proxy-history-list', 'http://127.0.0.1:10100/v1/alpha/history/v2/list_windows', ctx, upstream=False)
if obs.get('http_status') == 200 and isinstance(compacted, list) and compacted:
    followup = {'model': 'gpt-6.1-sol', 'input': compacted + [{'role': 'user', 'content': [{'type': 'input_text', 'text': 'Reply only with the marker previously recorded.'}]}],
                'instructions': 'Isolated synthetic state replay check. Reply concisely.', 'stream': True, 'store': False,
                'max_output_tokens': 128, 'reasoning': {'effort': 'low'}}
    p.call('tutorial-native-compaction-replay', 'https://api.krill-code.net/backend-api/codex/responses', followup)
