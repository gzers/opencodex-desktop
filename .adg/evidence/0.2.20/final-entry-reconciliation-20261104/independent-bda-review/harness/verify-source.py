from pathlib import Path
import hashlib, json, subprocess, urllib.request, datetime

OUT = Path('/tmp/ocx020-final-review-agent')
REPO = Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes')
SHA = 'bda7c92da4e1f2e02e6bc2c641e0747748b98dd5'
PROTO = Path('/tmp/ocx020-compact-prototype-complete')
def digest(data): return hashlib.sha256(data).hexdigest()
def git(*args): return subprocess.check_output(['git', '--no-optional-locks', *args], cwd=REPO)
files = git('ls-tree', '-r', '--name-only', SHA, '--', 'apps/desktop/ui/src', 'apps/desktop/ui/qa').decode().splitlines()
rows = []
for file in files:
    frozen = git('show', SHA + ':' + file)
    current = (REPO / file).read_bytes()
    rows.append({'path': file, 'sha256': digest(current), 'frozenSha256': digest(frozen), 'equal': frozen == current})
assert all(x['equal'] for x in rows), [x for x in rows if not x['equal']]
samples = ['src/styles/tokens.css', 'src/styles/base.css', 'src/App.vue', 'src/main.ts',
 'src/features/models/AgentPanel.vue', 'src/features/updates/UpdateCenter.vue',
 'src/routes/ModelsRoute.vue', 'src/routes/OverviewRoute.vue', 'src/routes/SettingsRoute.vue',
 'src/features/backup/ModelSaveHistory.vue', 'src/features/backup/TransferBackups.vue',
 'src/features/sync/WebDavConnection.vue', 'src/features/sync/ConnectionDialog.vue',
 'src/features/sync/ImportedConnections.vue', 'src/components/ui/UiDialog.vue',
 'src/features/extensions/components/ExtensionDetailModal.vue']
served = []
for file in samples:
    url = 'http://127.0.0.1:52422/' + file + '?raw'
    with urllib.request.urlopen(url, timeout=12) as response: payload = response.read()
    text = payload.decode().split('export default ', 1)[1]
    raw = json.JSONDecoder().raw_decode(text)[0].encode()
    source = git('show', SHA + ':apps/desktop/ui/' + file)
    served.append({'url': url, 'sourcePath': 'apps/desktop/ui/' + file, 'responseSha256': digest(payload),
                   'rawSourceSha256': digest(raw), 'bdaSha256': digest(source), 'equal': raw == source})
assert all(x['equal'] for x in served), served
receipt = json.loads((PROTO / 'baseline-receipt.json').read_text())
proto_rows = []
for file, expected in receipt['sha256'].items():
    actual = digest((PROTO / file).read_bytes())
    proto_rows.append({'path': file, 'sha256': actual, 'receiptSha256': expected, 'equal': actual == expected})
# The original receipt may predate the separately documented typography sample
# and low-effects host changes. Retain every difference explicitly.
selected = []
for file in ['docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/model-config.js',
 'docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/model-config.css',
 'docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/index.html']:
    data = (PROTO / file).read_bytes()
    frozen = git('show', 'cfbd90c4b:' + file)
    selected.append({'path': file, 'sha256': digest(data), 'cfbdSha256': digest(frozen), 'equalCfbd': data == frozen})
result = {'createdAt': datetime.datetime.now(datetime.timezone.utc).isoformat(), 'productSource': SHA,
 'checkoutHead': git('rev-parse', 'HEAD').decode().strip(), 'servedProductOrigin': 'http://127.0.0.1:52422',
 'productionPreview52424Used': False, 'allSourceAndQaBytesEqualFrozen': True, 'files': rows, 'servedRawSamples': served,
 'prototypeOrigin': 'http://127.0.0.1:52429', 'prototypeRoot': str(PROTO),
 'prototypeBaseline': 'cfbd90c4b+1da4cabca+20261103compact-font+low-no-WebGL',
 'prototypeReceiptSha256': digest((PROTO/'baseline-receipt.json').read_bytes()),
 'prototypeReceiptFiles': proto_rows, 'prototypeSelectedSource': selected,
 'scope': 'Read-only source/loopback HTTP identity verification. No native bridge or daily configuration.'}
target = OUT / 'receipts' / 'source-identity.json'
target.write_text(json.dumps(result, ensure_ascii=False, indent=2) + '\n')
print(json.dumps({'sourceFiles': len(rows), 'servedSamples': len(served), 'prototypeReceiptDifferences':
 [x['path'] for x in proto_rows if not x['equal']], 'output': str(target)}, ensure_ascii=False))
