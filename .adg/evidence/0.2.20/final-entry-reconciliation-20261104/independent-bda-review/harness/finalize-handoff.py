from pathlib import Path
import json,hashlib,datetime
R=Path('/tmp/ocx020-final-review-agent')
SHA='bda7c92da4e1f2e02e6bc2c641e0747748b98dd5'
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p):return {'path':str(p),'sha256':h(p)}
def write(name,value):(R/name).write_text(json.dumps(value,ensure_ascii=False,indent=2)+chr(10))
commands=[]
for engine in ['chromium','webkit']:
 commands.append({'kind':'matrix-orchestrator','command':f'python3 {R}/harness/run-matrices.py {engine}','cwd':str(R),'executed':True,'preservedHarness':ref(R/'harness/run-matrices.py'),'exitCode':0})
 for item in json.loads((R/'receipts'/f'matrix-runs-{engine}.json').read_text()):
  suite=item['suite'];temp=R/'browser-temp'/f'{suite}-{engine}'
  commands.append({'kind':'matrix-child','argv':['node',str(R/'harness'/f'{suite}.mjs')],'cwd':str(R),
   'envOverrides':{'AUDIT_ENGINE':engine,'AUDIT_OUT':item['output'],'AUDIT_SOURCE':SHA,'TMPDIR':str(temp)},
   'executed':True,'exitCode':item['exitCode'],'executionReceipt':ref(Path(item['output'])/'execution.json'),
   'executionLog':ref(Path(item['output'])/'execution.log'),'preservedHarness':ref(R/'harness'/f'{suite}.mjs')})
runs=[
 ('entry-probe-r1','entry-probe.mjs','entry-probe-r1.mjs','webkit',1,'probe-outdated-prototype-tab-selector'),
 ('entry-probe-r2','entry-probe.mjs','entry-probe.mjs','chromium',0,'diagnostic-not-acceptance'),
 ('update-hosts-webkit-r1','update-hosts.mjs','update-hosts-r1.mjs','webkit',1,'probe-used-disabled-version-row-instead-of-progress-entry'),
 ('update-hosts-webkit-r2','update-hosts.mjs','update-hosts-r2.mjs','webkit',1,'probe-check-failure-after-pending-restart-status-priority'),
 ('update-hosts-webkit-r3','update-hosts.mjs','update-hosts-r3.mjs','webkit',1,'probe-compared-settings-hash-without-section'),
 ('update-hosts-webkit-r4','update-hosts.mjs','update-hosts.mjs','webkit',0,'bounded-state-geometry-pass-not-shared-zoom-pass'),
 ('update-hosts-chromium-r1','update-hosts.mjs','update-hosts.mjs','chromium',0,'bounded-state-geometry-pass-not-shared-zoom-pass'),
 ('agent-hosts-chromium-r1','agent-hosts.mjs','agent-hosts.mjs','chromium',0,'bounded-states-current-font-layout-pass'),
 ('agent-hosts-webkit-r1','agent-hosts.mjs','agent-hosts.mjs','webkit',0,'bounded-states-current-font-layout-pass'),
 ('prototype-entries-chromium-r1','prototype-entries.mjs','prototype-entries-r1.mjs','chromium',1,'probe-settings-button-accessible-name'),
 ('prototype-entries-chromium-r2','prototype-entries.mjs','prototype-entries-r2.mjs','chromium',1,'probe-wrong-settings-tab-role'),
 ('prototype-entries-chromium-r3','prototype-entries.mjs','prototype-entries-r3.mjs','chromium',1,'probe-wrong-update-detail-action-selector'),
 ('prototype-entries-webkit-r3','prototype-entries.mjs','prototype-entries-r3.mjs','webkit',1,'probe-wrong-update-detail-action-selector'),
 ('prototype-entries-chromium-r4','prototype-entries.mjs','prototype-entries-r4.mjs','chromium',1,'viewport900-with-default-prototype-window1180'),
 ('prototype-entries-webkit-r4','prototype-entries.mjs','prototype-entries-r4.mjs','webkit',1,'viewport900-with-default-prototype-window1180'),
 ('prototype-entries-chromium-r5','prototype-entries.mjs','prototype-entries.mjs','chromium',0,'explicit-prototype-product-window-frame-pass'),
 ('prototype-entries-webkit-r5','prototype-entries.mjs','prototype-entries.mjs','webkit',0,'explicit-prototype-product-window-frame-pass'),
 ('update-scale-chromium-r1','update-scale-contract.mjs','update-scale-contract.mjs','chromium',1,'confirmed-product-failure-shared-zoom'),
 ('update-scale-webkit-r1','update-scale-contract.mjs','update-scale-contract.mjs','webkit',1,'confirmed-product-failure-shared-zoom'),
 ('prototype-host-diagnostics-r1','prototype-host-diagnostics.mjs','prototype-host-diagnostics.mjs','chromium',0,'read-only-layout-host-cause-diagnosis'),
]
for name,executed,preserved,engine,code,meaning in runs:
 out=R/'reports'/name;log=R/'reports'/(name+'.log')
 command=f'AUDIT_ENGINE={engine} AUDIT_OUT={out} AUDIT_SOURCE={SHA} TMPDIR={R}/browser-temp node {R}/harness/{executed} > {log} 2>&1'
 commands.append({'kind':'browser-direct','name':name,'commandAsExecuted':command,'cwd':str(R),'executed':True,'exitCode':code,
  'meaning':meaning,'preservedExecutionVersion':ref(R/'harness'/preserved),'engineWrapper':ref(R/'harness/engine.mjs'),
  'log':ref(log),'reportDirectory':str(out),'replayCaution':'Original command referred to the working harness name. Use the preservedExecutionVersion for historical reproducibility, never overwrite original report directories.'})
write('exact-commands.json',{'productSource':SHA,'allCommandsPreviouslyExecuted':True,'noNewBrowserRunsDuringHandoff':True,
 'commands':commands,'nonBrowserWork':'Read-only source/hash evidence assembly and allowed-directory report writes; no build/native/publish actions.'})
handoff={'state':'handed-off-product-fix-owned-by-main-thread','createdAt':datetime.datetime.now(datetime.timezone.utc).isoformat(),
 'productSource':SHA,'stable':'paused-by-main-thread-until-shared-scale-fix-and-review',
 'confirmedProductReleaseBlocker':{'id':'FR-020-UPDATE-SCALE','owners':['OverviewRoute UpdateCenter','SettingsRoute UpdateCenter'],
 'source':'apps/desktop/ui/src/features/updates/UpdateCenter.vue:225 Teleport to body outside scaled .app-window',
 'evidence':[ref(R/'reports/update-scale-chromium-r1/report.json'),ref(R/'reports/update-scale-webkit-r1/report.json')],
 'actual':'100%→200% shell heading33→66px while update footer button34→34px, modalZoom1 vs shellZoom2'},
 'prototypeClassification':'viewport vs frozen .window mismatch; not draggable coordinates; not an actual product release blocker',
 'prototypeDiagnosis':ref(R/'prototype-host-diagnosis.json'),
 'historyRecoveryEvidenceGap':'Backend actualtempfile plus browserpreview/cancel; directfinalnativeUIacceptsuccess not found/claimed. Separate from confirmed producterror.',
 'connectionEvidence':'Main-thread actualfinalMac consumer included; seededcatalog not transfer; legacyWebDAVfilebackend separate from transfer_protection nativekeyring',
 'actualDualPlatformOta':'Main-thread pass receipts archived and included, not an outstanding gate',
 'preservedResults':{'productPassingAssertions':1680,'productScreenshots':394,'prototypePassingAssertions':48,
 'sourceIdentity':ref(R/'receipts/source-identity.json'),'sourceAfter':ref(R/'receipts/source-identity-after.json'),
 'index':ref(R/'reports/index.json'),'allFiles':'artifact-manifest.json and SHA256SUMS'},
 'integration':{'review':ref(R/'FINAL-REVIEW.md'),'mappingCorrection':ref(R/'entry-evidence-map.corrections.json'),
 'completeProposedMap':ref(R/'entry-evidence-map.reviewed.json'),'nativeCorrections':ref(R/'native-evidence-corrections.json'),'exactCommands':ref(R/'exact-commands.json')},
 'nextReviewOnlyOnUserRequest':{'scope':'Only UpdateCenter in OverviewRoute and SettingsRoute against actual fixed source identity; bothengines,100/200%, sameviewport/theme/material; validate actionreachability, ownerreturn, sharedscaling.',
 'focusedHarnesses':['harness/update-scale-contract.mjs','harness/update-hosts.mjs'],
 'mustNotReuseBdaAsFixedSource':'Record fixed commit/sourcehash and verify its servedbytes before calling any fixed-source run passed. Existingbda reports remain unchanged.',
 'avoid':'Do not rerun model/extension/secondary/Agent/prototype passing matrices or realOTA without a new change/concern requiring them; do not run closeout/publish scripts from this agent.'}}
write('handoff.json',handoff)
(R/'HANDOFF.md').write_text('''最终交接：主线程接手UPDATE-SCALE产品修复，stable暂停。本轮未再次执行浏览器QA。

实际产品阻塞：两个宿主的UpdateCenter从body Teleport脱离.app-window共享缩放；100%→200%按钮34→34px，主壳标题33→66px。双引擎反例与执行命令见handoff.json及exact-commands.json。

原型r4只属于viewport与打样.window尺寸不一致；不是可拖拽坐标，也不是实际产品回归。冻结JS/CSS前后相等，原型源未修改。原始失败、宿主诊断和正确frame通过全部保留。

当前通过结果1680产品断言/394截图、48原型断言均保留，不再重复。主线程真实Mac连接消费者、凭据后端纠正、双平台actualOTA已纳入。历史恢复草稿仍只声明真实后端+浏览器证据，没有的最终nativeUI成功回执未冒称通过。

整合入口：FINAL-REVIEW.md、entry-evidence-map.corrections.json、entry-evidence-map.reviewed.json、native-evidence-corrections.json。handoff.json保存这些产物摘要。

后续仅在用户明确要求后复核修复源的UpdateCenter两个宿主，复用update-scale-contract.mjs和update-hosts.mjs。先记录新产品提交及服务字节，不把bda报告复用为修复后通过；输出新目录，不覆盖当前失败或通过。无需重跑无变化的模型/扩展/次要/Agent/原型矩阵。
''')
files=[]
for p in sorted(R.rglob('*')):
 if p.is_file() and 'browser-temp' not in p.relative_to(R).parts and p.name not in ['artifact-manifest.json','SHA256SUMS']:
  files.append({'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':h(p)})
write('artifact-manifest.json',{'root':str(R),'files':files})
(R/'SHA256SUMS').write_text(''.join(x['sha256']+'  '+x['path']+chr(10) for x in files))
print(json.dumps({'handoff':str(R/'handoff.json'),'exactCommandRecords':len(commands),'preservedFiles':len(files),'newBrowserRuns':0}))
