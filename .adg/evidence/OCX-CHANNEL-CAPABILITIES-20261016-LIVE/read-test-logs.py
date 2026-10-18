import json
from pathlib import Path
import urllib.request
import urllib.error
from urllib.parse import urlencode

HERE = Path(__file__).resolve().parent
observed = json.loads((HERE / 'observations.json').read_text())
params = urlencode({'conversationId': observed['session'], 'limit': 20})
opener = urllib.request.build_opener(urllib.request.ProxyHandler({}))
result = {'kind': 'bounded-local-read', 'method': 'GET', 'path': '/api/logs', 'scope': 'Current synthetic session only'}
try:
    try:
        response = opener.open('http://127.0.0.1:10100/api/logs?' + params, timeout=8)
    except urllib.error.HTTPError as err:
        response = err
    with response:
        result['status'] = response.status
        if response.status == 200:
            payload = json.loads(response.read(2 * 1024 * 1024))
            rows = payload.get('logs', payload.get('entries', []))
            result['available_top_level_fields'] = list(payload)
            keep = ['requestId', 'conversationId', 'provider', 'model', 'requestedModel', 'status', 'requestedServiceTier',
                    'callerServiceTier', 'configuredServiceTier', 'responseServiceTier', 'tierOutcome', 'protocolMode', 'sendCount']
            result['logs'] = [{k: row[k] for k in keep if k in row} for row in rows if isinstance(row, dict)]
            result['total'] = payload.get('total')
        else:
            result['meaning'] = 'Management read not available without additional authentication; no bypass attempted.'
except Exception as err:
    result['transport_error'] = type(err).__name__
(HERE / 'test-session-logs.json').write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps(result, ensure_ascii=False, indent=2))
