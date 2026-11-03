from pathlib import Path
import json,hashlib,subprocess,copy,datetime
R=Path('/tmp/ocx020-final-review-agent')
D=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/governance-main')
C=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes')
E=D/'.adg/evidence/0.2.20'
SHA='bda7c92da4e1f2e02e6bc2c641e0747748b98dd5'
def digest(p): return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p,**extra): return dict(path=str(p),sha256=digest(p),**extra)
def save(p,x): (R/p).write_text(json.dumps(x,ensure_ascii=False,indent=2)+chr(10))
reports=[]
for suite in ['model-dialogs','extensions','secondary','model-completion','agent-hosts','update-hosts']:
 for engine in ['chromium','webkit']:
  n=f'{suite}-{engine}-r'+('4' if suite=='update-hosts' and engine=='webkit' else '1')
  p=R/'reports'/n/'report.json';x=json.loads(p.read_text());b=json.loads((p.parent/'browser-boundary-and-fonts.json').read_text())
  assert x['completed'] and not x['failure'] and all(y['passed'] for y in x['checks'])
  assert not b['pageErrors'] and not b['resourceErrors'] and not b['blockedExternal']
  assert len(b['observations'])==len(x['screens']) and all(y['fontContractPass'] for y in b['observations'])
  reports.append(ref(p,suite=suite,engine=engine,result='pass',checks=len(x['checks']),screens=len(x['screens']),scope=x['scope'],fontsAndBoundary=ref(p.parent/'browser-boundary-and-fonts.json')))
prototypes=[]
for engine in ['chromium','webkit']:
 p=R/'reports'/f'prototype-entries-{engine}-r5'/'report.json';x=json.loads(p.read_text());b=json.loads((p.parent/'browser-boundary-and-fonts.json').read_text())
 assert x['completed'] and not x['failure'] and all(y['passed'] for y in x['checks']) and not b['pageErrors'] and not b['resourceErrors'] and not b['blockedExternal']
 prototypes.append(ref(p,engine=engine,checks=len(x['checks']),screens=len(x['screens']),scope=x['scope']))
negative=[]
for engine in ['chromium','webkit']:
 p=R/'reports'/f'update-scale-{engine}-r1'/'report.json';x=json.loads(p.read_text());assert x['result']=='fail' and all(not y['passed'] for y in x['checks'])
 negative.append(ref(p,engine=engine,result='fail',checks=x['checks']))
blocker={'id':'FR-020-UPDATE-SCALE','priority':'P1','releaseBlocking':True,'status':'confirmed-product-shared-scale-contract-failure','acceptanceIds':['AC-020-19','AC-020-20'],'s20':['S20-19','S20-20'],
 'title':'UpdateCenter in Overview and Settings does not follow shared interface scale',
 'actual':'Same1180x760/theme/material:100% to200%, shell heading33 to66px; UpdateCenter footer button34 to34px. Root/shell zoom=2, modal zoom=1, modal outside app-window.',
 'source':[{'path':str(C/'apps/desktop/ui/src/features/updates/UpdateCenter.vue'),'line':225},{'path':str(C/'apps/desktop/ui/src/styles/base.css'),'line':35},{'path':str(C/'apps/desktop/ui/src/app/appearance/scale.ts'),'line':17}],
 'evidence':negative,'action':'Fix product shared-scale/overlay integration and recheck both hosts, or obtain explicit requirement change. Do not infer scale acceptance from fitting geometry. This agent did not modify product.'}
diag=json.loads((R/'reports/prototype-host-diagnostics-r1/report.json').read_text())
host={'id':'FR-020-PROTOTYPE-HOST','releaseBlocking':False,'classification':'probe-viewport-versus-prototype-window-size-mismatch',
 'originalFailure':ref(R/'reports/prototype-entries-chromium-r4/report.json'),
 'cause':'Browser900x600 did not resize the frozen prototype window1180x760, which was centered atx=-140. Assistant absolute coordinates belong to #mpAssistantHost/.window, not the browser viewport. No draggable-position cause.',
 'source':'Exact frozen cfbd90c4b model-config.css526-530:host absolute inset0; panel absolute right24/bottom24 width360/height480; model-config.js1617,1620-1630. Source hashes equal before and after.',
 'diagnosticEvidence':ref(R/'reports/prototype-host-diagnostics-r1/report.json'),'rawProbe':diag['samples'][0],'explicitProductWindowProbe':diag['samples'][1],
 'method':'Existing compact-type QA convention:review canvas1700x1000; in-memory .window width/height set to1180x760 or900x600, then check the panel against that measured frame. No floating-panel coordinates or source files changed.',
 'finalPrototypeEvidence':prototypes,'independentProductEvidence':[x for x in reports if x['suite']=='agent-hosts'],
 'limits':'Prototype reference100% only; no claim of full prototype200% scaling or exact cross-product pixel parity. Actual product accessibility/layout separately verified. No future modules adopted.'}
native=E/'font-final-candidate-bda7c92d/native-connection-consumers'
n=json.loads((native/'result.json').read_text());assert n['result']=='pass' and n['sourceCommit']==SHA
first=json.loads((native/'first-saved-connection.json').read_text())['endpoints'][0]
second=json.loads((native/'second-saved-connection.json').read_text())['endpoints'][0]
active=json.loads((native/'activated-connection.json').read_text())['endpoints'][0]
stable=['ref_id','account_key','purpose','created_at','service_name']
assert all(first['credential_ref'][k]==second['credential_ref'][k]==active['credential_ref'][k] for k in stable)
connection={'id':'nativeConnectionConsumersFinal','status':'pass-recorded-macos-final-package-consumers','evidence':ref(native/'result.json'),'sourceCommit':SHA,'tests':n['actualNativeTests'],
 'artifacts':[ref(p) for p in sorted(native.iterdir()) if p.is_file() and p.suffix in ['.json','.jsonl','.txt','.jpg']],
 'verifiedStableCredentialFields':stable,'credentialCorrection':{'webdavPassword':'macOS legacy local plaintext file backend protected by0600/0700. backend=keychain is historical metadata, not proof of OS keyring.',
 'contentProtection':'Independent model_config/transfer_protection uses native keyring. Do not generalize connection tests as full system-keychain acceptance.',
 'source':[{'path':str(C/'apps/desktop/tauri/src/infrastructure/credentials.rs'),'lines':[73,75,115,117,166,170]},{'path':str(C/'apps/desktop/tauri/src/modules/model_config/transfer_protection.rs'),'lines':[35,50,204]}]},
 'tls':n['trustBoundary'],'catalogBoundary':n['importBoundary'],'realSyncExecuted':False,'executionOwner':'main-thread; review agent read only'}
log=E/'implementation-20261102/ocx020-precommit-rust.log'
need=['restores_only_affected_channels_to_draft_preserves_independent_edits_and_replay_does_not_replace_later_input','overlapping_draft_and_changed_confirmation_are_refused_without_runtime_or_library_writes','save_requires_exact_config_and_secret_readback','save_retry_keeps_full_intent_and_observes_real_error']
lines=[{'line':i,'text':s} for i,s in enumerate(log.read_text().splitlines(),1) if any(t in s for t in need)];assert len(lines)==4 and all(' ... ok' in s['text'] for s in lines)
history={'id':'historyRecoveryDraftCorrection','status':'backend-real-files-and-browser-preview-cancel; final-native-UI-accept-not-established',
 'removeAssociation':'ModelSaveHistory is not TransferBackups. Mixed-resource scope restore does not establish model history recovery-draft acceptance.',
 'existingActualBackendEvidence':ref(log,matchingLines=lines,scope='Temporary ModelStore/SyncConfigStore actual disk files. Endpoint Credentials is explicit in-memory adapter. Not nativeUI/keyring.'),
 'frozenFinalBackendJobIndex':ref(E/'font-final-candidate-bda7c92d/final-ci-completed.json',scope='Full workspace backend jobs success onbda; not direct native UI history-accept receipt.'),
 'currentUiEvidence':[x for x in reports if x['suite']=='secondary'],
 'verifiedUi':['default pointer restore off','comparison required','explicit recovery-draft action','cancel does not attach recovery','current-font layout/actions fit'],
 'missingEvidence':'No direct final-package UI successful history accept into recovery draft/owner return found in inspected unarchived0.2.20 AX/results/logs. This is an evidence-layer gap, not a demonstrated execution failure.',
 'action':'Use recovery.rs actual backend tests and browser preview/cancel evidence, retain native-accept unproven unless main thread supplies its direct receipt.'}
sources=[]
for file in ['apps/desktop/tauri/src/modules/model_config/recovery.rs','apps/desktop/tauri/src/modules/model_config/runtime.rs','apps/desktop/tauri/src/modules/sync/config.rs','apps/desktop/tauri/src/commands/model_config.rs']:
 data=subprocess.check_output(['git','show',SHA+':'+file],cwd=C);changed=subprocess.check_output(['git','diff','--name-only','35c06338',SHA,'--',file],cwd=C).decode()
 sources.append({'path':file,'bdaSha256':hashlib.sha256(data).hexdigest(),'equal35c06338ToBda':not bool(changed)})
history['backendSourceReuse']=sources
ota=[]
for name in ['font-final-candidate-bda7c92d/windows-adjacent-ota-41683704589/summary.json','font-final-candidate-bda7c92d/macos-adjacent-ota/ota-result.json']:
 p=E/name;x=json.loads(p.read_text());assert x['result']=='pass' and x['sourceCommit']==SHA;ota.append(ref(p,status='pass-main-thread-actual-OTA'))
save('reports/index.json',{'productSource':SHA,'passingBrowserReports':reports,'passingPrototypeReports':prototypes,'productAssertionsPassed':1680,'productScreenshots':394,'prototypeAssertionsPassed':48,'negativeScaleEvidence':negative,'scope':'Counts are actual observations, not total functional/native coverage. Shared zoom failure remains despite font-token/geometry passes.'})
save('prototype-host-diagnosis.json',host)
save('native-evidence-corrections.json',{'productSource':SHA,'connectionConsumers':connection,'historyRecoveryDraft':history,'actualOta':ota,'stable':'main-thread promotion pending; no publish action here'})
base=E/'final-entry-reconciliation-20261104/entry-evidence-map.json';m=json.loads(base.read_text());updated=copy.deepcopy(m)
updated['reviewAgentSourceIdentity']=ref(R/'receipts/source-identity.json');updated['reviewAgentReportIndex']=ref(R/'reports/index.json')
updated['passingEvidence']['nativeConnectionConsumersFinal']={'path':str(native/'result.json'),'sha256':digest(native/'result.json'),'scope':'Actual finalmacOS UI/HTTPS/save/reopen/activation; explicitseed catalog only; legacy file credentials.'}
updated['passingEvidence']['historyBackendRealFiles']={'path':str(log),'sha256':digest(log),'scope':'Actual temporary-file recovery draft backend tests, not native UI','matchingLines':lines[:2]}
for x in reports:updated['passingEvidence']['reviewAgent-'+x['suite']+'-'+x['engine']]={'path':x['path'],'sha256':x['sha256'],'checks':x['checks'],'scope':x['scope'],'fontSamples':x['fontsAndBoundary']}
changes=[]
for row in updated['static']:
 f=row['file'];before=copy.deepcopy(row)
 if f.endswith('ModelSaveHistory.vue'):
  row['evidenceGroups']=['secondary','reviewAgent-secondary-chromium','reviewAgent-secondary-webkit','historyBackendRealFiles'];row['evidenceBoundary']='Actual backend recovery files plus currentbrowser preview/cancel. Finalnative UIaccept not established.'
 elif f.endswith(('ConnectionDialog.vue','ImportedConnections.vue')):
  row['evidenceGroups']=[x for x in row['evidenceGroups'] if x!='native final-package custom export/skip/source/results']+['nativeConnectionConsumersFinal','reviewAgent-secondary-chromium','reviewAgent-secondary-webkit'];row['evidenceBoundary']='ActualmacOS finalUI save/reopen/blankpassword/activation; seededcatalog not package-transfer import; legacyfile WebDAV credentials.'
 elif '/features/models/' in f or f.endswith('/ModelsRoute.vue'):
  row['evidenceGroups']+=['reviewAgent-model-dialogs-chromium','reviewAgent-model-dialogs-webkit','reviewAgent-model-completion-chromium','reviewAgent-model-completion-webkit']
 elif f.endswith('/ExtensionsRoute.vue'):row['evidenceGroups']+=['reviewAgent-extensions-chromium','reviewAgent-extensions-webkit','reviewAgent-secondary-chromium','reviewAgent-secondary-webkit']
 if row!=before:
  row['currentTypography']='bda actual rendered captures+computed font values at recorded states/scales; reports/index.json. Historical screenshots not final typography proof.'
  changes.append({'match':{k:row[k] for k in ['file','component','line']},'beforeEvidenceGroups':before['evidenceGroups'],'replace':copy.deepcopy(row)})
supplement=[]
for owner,file,line in [('overview','OverviewRoute.vue',337),('settings','SettingsRoute.vue',851)]:
 supplement.append({'file':'apps/desktop/ui/src/routes/'+file,'line':line,'component':'UpdateCenter','owner':owner,'prototypeReference':'frozen maintenance-ui.js145-207 showUpdates/progressDialog/applyDialog; settings-upgrade apply-dialog',
  'evidenceGroups':['reviewAgent-update-hosts-chromium','reviewAgent-update-hosts-webkit'],'prototypeEvidence':prototypes,'status':'state/geometry pass; shared-scale contract fails','blockingFinding':blocker['id']})
supplement.append({'file':'apps/desktop/ui/src/routes/ModelsRoute.vue','line':215,'component':'AgentPanel','contexts':['existing-template editor','new-template editor'],'prototypeReference':'Exactcfbd90c4b model-config.js1617,1620-1630 andmodel-config.css526-557',
 'evidenceGroups':['reviewAgent-agent-hosts-chromium','reviewAgent-agent-hosts-webkit'],'prototypeEvidence':prototypes,'status':'recorded browserstates/currentfontlayout passed','excludedAsFutureConversation':False,'limits':'No actual modelgeneration/research/native persistence; no claim of full pixel/control parity.'})
updated['supplementalConsumers']=supplement;updated['reviewFindings']=[blocker];updated['prototypeHostDiagnosis']=host;updated['credentialStorageCorrection']=connection['credentialCorrection'];updated['actualOtaEvidence']=ota
updated['scopeCountNote']='Original37 tags+26 dynamic calls remain scanner scope; three supplemental customconsumer references separate, not three unique windows.'
updated['existingAssistantNotExcluded']={'source':'cfbd90c4b model-config.js assistantHTML and ModelsRoute.vue215','meaning':'Existing adopted modeltemplate assistant is distinct from excluded futureconversation management/policy redesign.'}
save('entry-evidence-map.reviewed.json',updated)
save('entry-evidence-map.corrections.json',{'base':ref(base),'productSource':SHA,'mode':'Reviewable additive/replacement proposal; shared original unchanged','changedStaticEntries':changes,'supplementalConsumers':supplement,'reportIndex':ref(R/'reports/index.json'),'nativeCorrections':ref(R/'native-evidence-corrections.json'),'blockingFindings':[blocker],'prototypeHostDiagnosis':ref(R/'prototype-host-diagnosis.json'),'fullCandidateMap':str(R/'entry-evidence-map.reviewed.json'),'doNotClose':['review:prototype-parity','AC-020-19','AC-020-20'],'historyNativeAccept':'Pendingdirectreceipt; not claimed from mixedresourcerestore'})
failed=[]
for p in sorted((R/'reports').glob('*/report.json')):
 x=json.loads(p.read_text())
 if x.get('failure') or x.get('result')=='fail':failed.append(ref(p,failure=x.get('failure'),result=x.get('result')))
save('receipts/retained-failures.json',{'failures':failed,'policy':'All first/intermediate failing reports/screens/logs preserved; prototype host corrections distinct from productscale failure.'})
save('review-summary.json',{'productSource':SHA,'createdAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),'productPassingAssertions':1680,'prototypePassingAssertions':48,'productScreenshots':394,'blockingFindings':[blocker],'prototype900OverflowReleaseBlocking':False,'historyNativeUiAcceptance':'DirectfinalnativeUIreceipt not found, actualbackend+browser layer only','nativeConnection':'main-threadfinalpackage verified and mapped','actualDualPlatformOta':'main-threadreceipts pass','stable':'pendingmain-thread','mapping':str(R/'entry-evidence-map.corrections.json')})
print(json.dumps({'productPass':1680,'prototypePass':48,'screens':394,'blockingFinding':blocker['id'],'output':str(R)},ensure_ascii=False))
