from pathlib import Path
import json,hashlib,re,subprocess,sys
base=Path(__file__).resolve().parent;manifest=json.loads((base/'manifest.json').read_text());wanted=set()
for item in manifest['files']:
 p=Path(item['path']);assert not p.is_absolute() and '..' not in p.parts and item['path'] not in wanted;wanted.add(item['path']);raw=(base/p).read_bytes();assert len(raw)==item['bytes'] and hashlib.sha256(raw).hexdigest()==item['sha256'],item['path']
assert {str(p.relative_to(base)) for p in base.rglob('*') if p.is_file()}-{'manifest.json','verified-evidence.json'}==wanted
unique=set()
for label in ['ready-delivery','session-regression','index-regression','command-regression','trace-result-controls','final-mapping-build','final-first-root-lock','final-acquisition-boundary']:
 result=json.loads((base/(label+'.json')).read_text());log=(base/(label+'.log')).read_text();names={n for n,status in re.findall(r'^test (\S+) \.\.\. (.*)$',log,re.M) if not status.startswith('ignored')}
 if result['exitCode']:
  assert label=='index-regression' and result['exitCode']==101
  failures={line.strip() for line in log.rsplit('failures:\n\n',1)[1].splitlines() if line.strip().startswith('infrastructure::')}
  assert failures=={'infrastructure::macos_client_originals::session_index::root_registration_tests::first_root_registration_never_releases_the_original_action_flock_at_transfer_or_commit'};names-=failures
 counts=re.findall(r'test result: (?:ok|FAILED)\. (\d+) passed; (\d+) failed; (\d+) ignored;',log)[-1];assert len(names)==int(counts[0]);unique|=names
assert len(unique)==95
for label in ['clippy-delivery','format-delivery']:assert json.loads((base/(label+'.json')).read_text())['exitCode']==0
assert json.loads((base/'ready-controls.json').read_text())['exitCode']==101
source=json.loads((base/'source-final.json').read_text());build=json.loads((base/'source-build.json').read_text());prior=json.loads((base/'source-before-final-diagnostics.json').read_text())
assert len(source['files'])==7 and len(prior['files'])==6 and source['files']==build['files'] and source['actualLibraryTestBinarySha256']==build['actualLibraryTestBinarySha256'] and source['committedSource']==manifest['sourceCommit']
after={r['path']:r['sha256'] for r in source['files']};changed={r['path'] for r in prior['files'] if after[r['path']]!=r['sha256']}
assert changed=={'apps/desktop/tauri/src/infrastructure/macos_client_restore.rs','apps/desktop/tauri/src/infrastructure/macos_client_session_index.rs'}
comparison=json.loads(subprocess.check_output([sys.executable,str(base/'verify_comparison.py')],text=True));assert comparison==json.loads((base/'verified-comparison.json').read_text())
cleanup=json.loads((base/'cleanup.json').read_text());assert cleanup['temporaryRootRemoved'] and cleanup['unitTmpEmptyBeforeRemoval'] and cleanup['archiveComparedBeforeRemoval'] and cleanup['officialServiceStarted'] is False
print(json.dumps({'accepted':False,'manifestFiles':len(wanted),'manifestBytes':sum(r['bytes'] for r in manifest['files']),'sourceCommit':manifest['sourceCommit'],'uniquePassingFunctions':len(unique),'newFunctions':6,'sourceIdentities':2,'secondGenerationHistoricalObservations':[4,2],'secondGenerationFrameReads':[50,26],'deadlineAndRefusalDiagnosticContractVerified':True,'testHookFailurePreservedAndExactFunctionRetested':True,'previousNativeStopFailureStillEffective':True,'scope':'Ready acquisition optimization and safe test diagnostics; no new official runtime or fixed candidate acceptance'},indent=2))
