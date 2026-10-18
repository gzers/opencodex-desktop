import json
from pathlib import Path
import time
import urllib.request
import urllib.error
import uuid

HERE = Path(__file__).resolve().parent
CONFIG = Path('/Users/ezio/Library/Application Support/com.gzers.opencodex.desktop/opencodex-home/config.json')
PLAN = json.loads((HERE / 'plan.json').read_text())
state = json.loads((HERE / 'observations.json').read_text()) if (HERE / 'observations.json').exists() else {'kind': 'provider-live-observations', 'provider': PLAN['provider'], 'session': str(uuid.uuid4()), 'calls': [], 'limitations': []}
daily = json.loads(CONFIG.read_text())
provider = daily['providers'][PLAN['provider']]
credential = provider.get('apiKey')
if not isinstance(credential, str) or not credential or credential.startswith(('$', 'env:')):
    raise SystemExit('Configured credential is not directly usable; no key printed.')

class NoRedirect(urllib.request.HTTPRedirectHandler):
    def redirect_request(self, req, fp, code, msg, headers, newurl):
        return None

direct = urllib.request.build_opener(NoRedirect)
local = urllib.request.build_opener(urllib.request.ProxyHandler({}), NoRedirect)

def summaries(payload):
    if not isinstance(payload, dict):
        return {'json_type': type(payload).__name__}
    result = {}
    for field in ['id', 'model', 'status', 'service_tier', 'object']:
        value = payload.get(field)
        if isinstance(value, (str, int, bool)):
            result[field] = value[:180] if isinstance(value, str) else value
    if isinstance(payload.get('usage'), dict):
        result['usage'] = {k: v for k, v in payload['usage'].items() if k in ['input_tokens', 'output_tokens', 'total_tokens'] and isinstance(v, (int, float))}
    output = payload.get('output')
    if isinstance(output, list):
        result['output_types'] = [x.get('type') for x in output if isinstance(x, dict)]
        result['compaction_items'] = sum(isinstance(x, dict) and x.get('type') in ['compaction', 'compaction_summary', 'context_compaction'] for x in output)
        encrypted = [x.get('encrypted_content') for x in output if isinstance(x, dict) and isinstance(x.get('encrypted_content'), str)]
        result['state_formats'] = ['ocx1' if x.startswith('ocx1:') else 'opaque' for x in encrypted]
        texts = []
        for item in output:
            if isinstance(item, dict):
                for part in item.get('content', []) if isinstance(item.get('content'), list) else []:
                    if isinstance(part, dict) and isinstance(part.get('text'), str):
                        texts.append(part['text'])
        joined = ''.join(texts)
        result['expected_marker_present'] = 'ocx-probe-ok' in joined
        result['text_characters'] = len(joined)
    err = payload.get('error')
    if err:
        if isinstance(err, dict):
            result['error'] = {k: str(err[k])[:600].replace(credential, '[REDACTED]') for k in ['type', 'code', 'message'] if k in err}
        elif isinstance(err, str):
            result['error'] = err[:600].replace(credential, '[REDACTED]')
    models = payload.get('models', payload.get('data'))
    if isinstance(models, list):
        result['model_count'] = len(models)
        wanted = []
        for entry in models:
            if isinstance(entry, dict) and entry.get('id', entry.get('slug')) in ['gpt-6.1-sol', 'Krill-1/gpt-6.1-sol']:
                wanted.append({k: entry[k] for k in ['id', 'slug', 'supports_experimental_context', 'service_tiers', 'additional_speed_tiers', 'context_window'] if k in entry})
        result['selected_model_metadata'] = wanted
    return result

def call(label, url, body=None, upstream=True, retain_output=False):
    if len(state['calls']) >= PLAN['max_requests']:
        raise RuntimeError('Request cap reached')
    allowed = url.startswith('https://api.krill-code.net/') if upstream else url.startswith('http://127.0.0.1:10100/')
    if not allowed:
        raise RuntimeError('Destination outside plan')
    headers = {'Content-Type': 'application/json', 'Accept': 'application/json, text/event-stream',
               'User-Agent': 'ocx-desktop-compatibility-probe/20261016', 'session-id': state['session'], 'session_id': state['session']}
    if upstream:
        headers['Authorization'] = 'Bearer ' + credential
    req = urllib.request.Request(url, data=None if body is None else json.dumps(body).encode(), headers=headers, method='GET' if body is None else 'POST')
    obs = {'label': label, 'url': url, 'method': req.method}
    start = time.monotonic()
    captured_output = None
    try:
        try:
            response = (direct if upstream else local).open(req, timeout=40)
        except urllib.error.HTTPError as err:
            response = err
        with response:
            obs['http_status'] = response.status
            obs['content_type'] = response.headers.get('content-type', '')
            raw = response.read(4 * 1024 * 1024 + 1)
            obs['response_bytes'] = len(raw)
            if len(raw) > 4 * 1024 * 1024:
                obs['truncated'] = True
            else:
                decoded = raw.decode('utf-8', errors='replace')
                try:
                    payload = json.loads(decoded)
                    obs['response'] = summaries(payload)
                    if retain_output and isinstance(payload, dict):
                        captured_output = payload.get('output')
                except json.JSONDecodeError:
                    events = []
                    terminal = None
                    for line in decoded.splitlines():
                        if not line.startswith('data:'):
                            continue
                        try:
                            event = json.loads(line[5:].strip())
                        except json.JSONDecodeError:
                            continue
                        if isinstance(event, dict):
                            events.append(event.get('type'))
                            if event.get('type') in ['response.completed', 'response.failed', 'response.incomplete']:
                                terminal = event.get('response')
                            if event.get('type') == 'error':
                                obs['stream_error'] = summaries({'error': event.get('error', event)})
                    obs['event_types'] = sorted({x for x in events if isinstance(x, str)})
                    if terminal:
                        obs['response'] = summaries(terminal)
                        if retain_output and isinstance(terminal, dict):
                            captured_output = terminal.get('output')
                    elif not events:
                        obs['non_json_response'] = True
    except Exception as err:
        obs['transport_error'] = type(err).__name__
        obs['transport_detail'] = str(err)[:400].replace(credential, '[REDACTED]')
    obs['elapsed_seconds'] = round(time.monotonic() - start, 3)
    state['calls'].append(obs)
    (HERE / 'observations.json').write_text(json.dumps(state, ensure_ascii=False, indent=2) + '\n')
    print(json.dumps(obs, ensure_ascii=False), flush=True)
    return (obs, captured_output) if retain_output else obs

if __name__ == '__main__':
    call('proxy-model-directory', 'http://127.0.0.1:10100/v1/models', upstream=False)
    call('configured-krill-model-directory', 'https://api.krill-code.net/codex/v1/models')
    call('tutorial-krill-model-directory', 'https://api.krill-code.net/backend-api/codex/models')
