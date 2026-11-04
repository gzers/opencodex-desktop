import json, subprocess
from pathlib import Path
base=Path(__file__).resolve().parent
root=base/'candidate'; home=root/'sandbox/opencodex-home'
node='C:/Users/15119/.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node.exe'
def call(name):
    result=subprocess.run([node,str(base.parent/'cdp-eval.mjs'),'9432',str(base/'restart-call.js'),str(root/(name+'.json'))],capture_output=True,text=True,encoding='utf-8',check=True)
    print(result.stdout)
record=home/'runtime-port.json'; original=record.read_bytes()
try:
    record.write_text('{bad',encoding='utf-8');call('corrupt-record-final')
    data=json.loads(original);data['port']=1;record.write_text(json.dumps(data),encoding='utf-8');call('stale-listener-final')
finally:record.write_bytes(original)
token=home/'admin-api-token'; held=token.with_suffix('.held')
try:
    token.rename(held);call('missing-token-final')
finally:held.rename(token)
pending=root/'sandbox/manager-state/provider-client-renames.json'
assert not pending.exists()
pending.write_bytes((root/'provider-client-renames.fixture-blocked.json').read_bytes())
config=home/'config.json';held=config.with_suffix('.held')
try:
    config.rename(held);call('missing-saved-config-final')
finally:
    held.rename(config);pending.unlink()
