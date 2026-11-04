"""Read-only current-provider quota investigation. Never writes credentials/config."""
import concurrent.futures
import datetime
import hashlib
import json
import math
from pathlib import Path
import subprocess
import urllib.parse

ROOT = Path.home() / 'Library/Application Support/com.gzers.opencodex.desktop'
OUT = Path(__file__).parent
CONFIG = ROOT / 'opencodex-home/config.json'
LAYOUT = ROOT / 'manager-state/model-channel-layout.json'
NUMBERS = {'total_available', 'total_granted', 'total_used', 'remain_quota',
           'used_quota', 'hard_limit_usd', 'soft_limit_usd', 'system_hard_limit_usd',
           'total_usage', 'balance', 'credit', 'quota', 'quota_per_unit',
           'usd_exchange_rate', 'unlimited_quota', 'has_payment_method',
           'display_in_currency', 'quota_display_type', 'currency', 'unit'}
MARK = '\nOCX_META:'

def digest(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()

def evidence_only(node):
    """Keep response schema and flags, never account quota/usage amounts."""
    public = {'quota_per_unit','usd_exchange_rate','display_in_currency',
              'quota_display_type','currency','unit'}
    if isinstance(node, list):
        return [evidence_only(value) for value in node]
    if not isinstance(node, dict):
        return node
    result = {}
    for key, value in node.items():
        if key == 'fields':
            result[key] = {name: (item if name.rsplit('.',1)[-1] in public or isinstance(item,bool)
                                 else {'type':'number','sign':'negative' if item<0 else 'zero' if item==0 else 'positive'})
                           for name,item in value.items()}
        else:
            result[key] = evidence_only(value)
    return result

def query(origin, path, key=None):
    # Explicit finite GET allowlist. Auth travels on stdin, never argv or disk.
    assert path in {'/api/usage/token/', '/api/usage/token',
                    '/dashboard/billing/subscription', '/dashboard/billing/usage',
                    '/v1/dashboard/billing/subscription',
                    '/v1/dashboard/billing/usage', '/api/status'}
    url = origin + path
    args = ['curl', '--silent', '--show-error', '--connect-timeout', '5',
            '--max-time', '12', '--max-filesize', '524288', '--max-redirs', '0',
            '--proto', '=https', '--request', 'GET', '--url', url,
            '--write-out', MARK + '%{http_code} %{content_type}', '--config', '-']
    config = 'header = "Accept: application/json"\n'
    if key:
        config += 'header = ' + json.dumps('Authorization: Bearer ' + key) + '\n'
    result = subprocess.run(args, input=config.encode(), capture_output=True, timeout=16)
    body, _, meta = result.stdout.rpartition(MARK.encode())
    parts = meta.decode(errors='replace').split(' ', 1)
    code = int(parts[0]) if parts and parts[0].isdigit() else 0
    info = {'path': path, 'http': code, 'curlExit': result.returncode}
    if result.returncode:
        info['state'] = {6:'dns_error',7:'connect_error',28:'timeout',
                         35:'tls_error',60:'tls_trust_error',63:'body_too_large'}.get(result.returncode, 'transport_error')
        return info, None
    try:
        value = json.loads(body)
    except (ValueError, UnicodeDecodeError):
        info['state'] = 'html_or_non_json'
        return info, None
    if not isinstance(value, dict):
        info['state'] = 'unexpected_json'
        return info, None
    info['state'] = 'json' if 200 <= code < 300 else ('auth_rejected' if code in (401,403) else 'http_error')
    if value.get('success') is False:
        info['state'] = 'api_rejected'
    # Persist whitelisted quota scalars only; no raw body, user/name or secret.
    fields = {}
    def collect(node, prefix='', depth=0):
        if depth > 3 or not isinstance(node, dict):
            return
        for name, item in node.items():
            if name in NUMBERS and (isinstance(item, (bool,int,float)) or name in {'currency','unit','quota_display_type'} and isinstance(item,str) and len(item)<20):
                if not isinstance(item,float) or math.isfinite(item):
                    fields[prefix+name] = item
            if name in {'data','user_quota','subscription','result'}:
                collect(item, prefix+name+'.', depth+1)
    collect(value)
    info['fields'] = fields
    info['object'] = value.get('object') if value.get('object') in {'token_usage','credit_summary','subscription','list'} else None
    return info, value

def inspect(item):
    pid, provider, label = item
    row = {'providerId': pid, 'label': label, 'enabled': not provider.get('disabled', False), 'attempts': []}
    key = provider.get('apiKey')
    base = urllib.parse.urlsplit(provider.get('baseUrl',''))
    if provider.get('authMode') in {'forward','oauth','local','none'} or not key:
        row['result'] = 'non_api_key_source'
        return row
    if not isinstance(key, str) or key.startswith('$'):
        row['result'] = 'external_credential_not_resolved'
        return row
    if base.scheme != 'https' or not base.hostname or base.username or base.password:
        row['result'] = 'unsupported_origin'
        return row
    origin = urllib.parse.urlunsplit((base.scheme,base.netloc,'','',''))
    row['origin'] = origin
    standard, _ = query(origin, '/api/usage/token/', key)
    row['attempts'].append(standard)
    values = standard.get('fields',{})
    if standard['state']=='json' and any(k.endswith('total_available') or k.endswith('remain_quota') for k in values):
        row['result'] = 'token_quota_available'
        row['unit'] = 'provider_quota_units'
        status, _ = query(origin, '/api/status')
        row['displayMetadata'] = status
        return row
    sub, _ = query(origin, '/v1/dashboard/billing/subscription', key)
    row['attempts'].append(sub)
    if sub['state']=='json' and 'hard_limit_usd' in sub.get('fields',{}):
        usage, _ = query(origin, '/v1/dashboard/billing/usage', key)
        row['attempts'].append(usage)
        row['result'] = 'legacy_billing_available' if usage['state']=='json' and 'total_usage' in usage.get('fields',{}) else 'legacy_limit_only'
        row['unit'] = 'USD_limit_usage_unit_requires_validation'
    else:
        row['result'] = 'standard_query_unavailable'
    return row

def main():
    OUT.mkdir(parents=True, exist_ok=True)
    before = {p.name: digest(p) for p in [CONFIG,LAYOUT]}
    config = json.loads(CONFIG.read_text())
    labels = {c['connection']['providerId']:c.get('label') for c in json.loads(LAYOUT.read_text()).get('channels',[])}
    items = [(pid,p,labels.get(pid) or pid) for pid,p in config.get('providers',{}).items()]
    result = {'time': datetime.datetime.now(datetime.timezone.utc).isoformat(),
              'scope': 'All configured providers, active API Key and default/current baseUrl only; no alternate-node/key sweep',
              'network': 'GET only; same configured HTTPS origin; redirects disabled; no model requests; no login/admin credentials',
              'before': before, 'providers': []}
    with concurrent.futures.ThreadPoolExecutor(max_workers=4) as pool:
        for row in pool.map(inspect, items):
            result['providers'].append(row)
            print(row['providerId'], row['result'], [(a['path'],a['http'],a['state']) for a in row['attempts']], flush=True)
    result['after'] = {p.name: digest(p) for p in [CONFIG,LAYOUT]}
    result['dailyConfigUnchanged'] = result['before'] == result['after']
    (OUT/'quota-probe-result.json').write_text(json.dumps(evidence_only(result),ensure_ascii=False,indent=2)+'\n')
    assert result['dailyConfigUnchanged'], 'External config changed during observation; no writes were performed by this probe'
    print('providers=',len(items),'dailyConfigUnchanged=',result['dailyConfigUnchanged'])

if __name__ == '__main__':
    main()
