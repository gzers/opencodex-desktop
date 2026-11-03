from pathlib import Path
import hashlib,json,subprocess,urllib.request,datetime
R=Path('/tmp/ocx020-final-review-agent/final-current-version-details')
C=Path('/Users/ezio/WorkSpace/Code/tools/opencodex-desktop/ocx-0219-fixes')
FINAL='b1750f5cf78b294b2edb34b9674d8c707ef8f810'
def h(p):return hashlib.sha256(p.read_bytes()).hexdigest()
def ref(p,**kw):return {'path':str(p),'sha256':h(p),**kw}
def save(name,obj):(R/name).write_text(json.dumps(obj,ensure_ascii=False,indent=2)+chr(10))
final=json.loads((R/'receipts/source-identity-b175.json').read_text());initial=json.loads((R/'receipts/source-identity-before.json').read_text())
f='apps/desktop/ui/src/features/updates/UpdateCenter.vue';data=(C/f).read_bytes();assert data==subprocess.check_output(['git','show',FINAL+':'+f],cwd=C);assert hashlib.sha256(data).hexdigest()==final['sourceSha256']
payload=urllib.request.urlopen('http://127.0.0.1:52422/src/features/updates/UpdateCenter.vue?raw',timeout=10).read().decode();raw=json.JSONDecoder().raw_decode(payload.split('export default ',1)[1])[0].encode();assert data==raw
other=[x['path'] for x in initial['sourceFiles'] if x['path']!=f and h(C/x['path'])!=x['sha256']];assert not other
save('receipts/source-identity-after.json',{'fixedCommit':FINAL,'sourceSha256':h(C/f),'servedSha256':hashlib.sha256(raw).hexdigest(),'otherProductionSourceChanges':other,'capturedAt':datetime.datetime.now(datetime.timezone.utc).isoformat()})
reports=[]
for engine in ['chromium','webkit']:
 for suite,name,expected,source in [('versions-and-frozen-progress','current-version-'+engine+'-r1',286,initial['sourceSha256']),('update-hosts','update-hosts-'+engine+'-r1',156,initial['sourceSha256']),('original-strict-scale','update-scale-contract-'+engine+'-r1',2,initial['sourceSha256']),('expanded-strict-scale','update-scale-expanded-'+engine+'-r1',372,initial['sourceSha256']),('final-latest-body','latest-body-'+engine+'-b175',212,final['sourceSha256'])]:
  p=R/'reports'/name/'report.json';x=json.loads(p.read_text());assert x['completed'] and not x.get('failure') and len(x['checks'])==expected and all(c['passed'] for c in x['checks'])
  bp=p.parent/'browser-boundary-and-fonts.json';b=json.loads(bp.read_text());assert not b['pageErrors'] and not b['resourceErrors'] and not b['blockedExternal']
  row=ref(p,suite=suite,engine=engine,assertions=expected,result='pass',sampledUpdateCenterSha256=source,boundary=ref(bp),binding='direct-final-source' if source==final['sourceSha256'] else 'prior-continuous-source-plus-explicit-final-delta-proof')
  if suite=='final-latest-body':
   assert len(x['states'])==16 and all(s['condition'] in ['same','lower','none','unknownCurrent'] for s in x['states']);row['scopeCorrection']='REVIEW_BODY_ONLY=1:16details states; no newer/pending rerun. Reused report envelope scope is broader; actualstates/checks define thisrun.'
  reports.append(row)
old=(R/'sources/UpdateCenter.current.vue').read_bytes()
reuse={'priorReviewedSha256':initial['sourceSha256'],'finalCommit':FINAL,'finalSha256':final['sourceSha256'],
 'scriptSetupIdentical':old.split(b'<template>')[0]==data.split(b'<template>')[0],
 'stylesIdentical':old.split(b'<style scoped>')[1]==data.split(b'<style scoped>')[1],
 'diff':ref(R/'sources/UpdateCenter-c212-to-b175.diff'),
 'delta':'OnlyReleaseNotes now has v-if=selectedCandidate and version prop no null fallback. Noother production file changed.',
 'newerFlowReason':'selectedCandidate is truthy in already-verifiednewer cases, so both old/new render sameReleaseNotes with sameexact version/props. Strictscale scenarios are newer candidates; CSSidentical.',
 'pendingFlowReason':'ReleaseNotes is inside !showingProgress. Existingpending/failedprogress cases never enter thissubtree; frozenversion logic is in identicalscript setup.',
 'affectedFlowReason':'same/lower/none with nullcandidate nowremove entireReleaseNotes; unknownCurrentwithremotecandidate stays truthy. Bothengines directfinalsource reran these16 states.',
 'reuseDoesNotRelabelOldRuns':'Originalreports retain c212source and their oldSHA. Thisproof binds unchangedcoverage tofinalcommit; no unchangedscale matrices rerun afterv-if.'}
assert reuse['scriptSetupIdentical'] and reuse['stylesIdentical']
save('continuous-source-reuse.json',reuse)
save('reports/index.json',{'fixedCommit':FINAL,'actualUpdateCenterSha256':final['sourceSha256'],'reports':reports,'continuousSourceReuse':ref(R/'continuous-source-reuse.json'),'failures':[],'noNativeOrReleaseClaim':True})
commands=[]
for engine in ['chromium','webkit']:
 commands.extend(json.loads((R/'receipts'/f'existing-runs-{engine}.json').read_text()))
 commands.append({'name':'current-version-'+engine+'-r1','cwd':str(R),'command':f'AUDIT_ENGINE={engine} AUDIT_OUT={R}/reports/current-version-{engine}-r1 AUDIT_SOURCE=working-UpdateCenter-sha256:{initial["sourceSha256"]} TMPDIR={R}/browser-temp node {R}/harness/current-version-matrix.mjs > {R}/reports/current-version-{engine}-r1.log 2>&1','preservedExecutedHarness':ref(R/'harness/current-version-matrix-r1.mjs'),'exitCode':0})
 commands.append({'name':'latest-body-'+engine+'-b175','cwd':str(R),'command':f'AUDIT_ENGINE={engine} AUDIT_OUT={R}/reports/latest-body-{engine}-b175 REVIEW_BODY_ONLY=1 REVIEW_SOURCE_IDENTITY={R}/receipts/source-identity-b175.json AUDIT_SOURCE={FINAL} TMPDIR={R}/browser-temp node {R}/harness/current-version-matrix.mjs > {R}/reports/latest-body-{engine}-b175.log 2>&1','preservedExecutedHarness':ref(R/'harness/current-version-matrix.mjs'),'exitCode':0})
save('exact-commands.json',{'commands':commands,'allExecuted':True,'noActionsNativeBuildOrPublication':True})
source= subprocess.check_output(['git','show','cfbd90c4b:docs/04-项目资料/01-项目支撑资料/01-产品原型/原型/maintenance-ui.js'],cwd=C).decode();(R/'sources/frozen-prototype-showUpdates.txt').write_text(chr(10).join(source.splitlines()[147:154])+chr(10))
mapping={'issue':'Currentversion cachedsame/lower/none update details wronglypresent candidateconfirmation','status':'pass-reviewed-final-source','fixedCommit':FINAL,'actualUpdateCenterSha256':final['sourceSha256'],
 'scope':'Actual production OverviewRoute and SettingsRoute, manager and official targets; explicit memory-onlybrowserIPC/data. No product/docs modification or native/network/install/release actions.',
 'finalDirectEvidence':[x for x in reports if x['suite']=='final-latest-body'],
 'priorVerifiedContinuousEvidence':[x for x in reports if x['suite']!='final-latest-body'],
 'sourceReuseProof':ref(R/'continuous-source-reuse.json'),
 'verifiedBehaviors':{'sameLowerNone':'Currentversion only, no candidatearrow, details-onlytitle, close-onlyfooter, noinstall/restartinstruction orrelease-notes block.',
 'newer':'Current→exactnewer version, confirmationtitle andappropriate action/explanation retained; prior fullmatrix reused with unchangedtruthyReleaseNotes props.',
 'unknownCurrent':'Unknown→exactremote candidate andexactmatching release description remainreadable, actualAvailablefalse, noinstallconfirmation; finaldirect16state runincludes4unknown cases/engine.',
 'progress':'Capturedcurrent/candidate survivecachechanges anddismiss/reopen forbothmanageranddeveloperofficialtargets. Managerpendingrestart retains0.2.19→0.2.20 even whenlatestcachebecomes0.2.21. Officialpending/failure retains2.83→2.84 despitecache2.85.',
 'scale':'Bothhosts900x600/1180x760*100/150/200%, strict34/51/68 andoriginal200%ratio retained; fullCSSidentical andcandidate-truthyDOMidentical toverifiedsource.'},
 'prototype':ref(R/'sources/frozen-prototype-showUpdates.txt'),
 'retiredPackageBoundary':'34a is retainedhistory only. These browserreports bindfinalsource; do notlabel old34apackage asfinalacceptance.',
 'blockingFindings':[],'releaseReadinessNotEvaluated':'NewfullCI/package/OTA/publication remain main-thread responsibility.'}
save('mapping-correction.json',mapping)
save('handoff.json',{'result':'pass-with-explicit-continuous-source-reuse','fixedCommit':FINAL,'UpdateCenterSha256':final['sourceSha256'],
 'directFinalPerEngine':{'latestBodyAssertions':212,'actualStates':16,'conditions':['same','lower','none','unknownCurrent'],'hosts':['overview','settings'],'targets':['manager','official']},
 'priorReviewedPerEngine':{'fullVersionAndPendingMatrix':286,'originalUpdateHosts':156,'originalStrictScale':2,'expandedGeometryRatioScroll':372},
 'unchangedScaleRerunAfterVif':False,'sourceReuseProof':ref(R/'continuous-source-reuse.json'),'sourceIdentity':ref(R/'receipts/source-identity-after.json'),
 'evidenceIndex':ref(R/'reports/index.json'),'mappingCorrection':ref(R/'mapping-correction.json'),'exactCommands':ref(R/'exact-commands.json'),'actualFailures':[],
 'limits':'Browserfixtures only; no native/product/docs/publish actions. No claimofnewpackage/fullCI/stable completion.'})
(R/'FINAL-REVIEW.md').write_text('''复核通过：b1750f5cf78b294b2edb34b9674d8c707ef8f810。

最终UpdateCenter SHA-256：627cbb18faa5621615ae0e76f6554f9a8e87f485b74da75eaa2df1d2d2e0cf81。工作文件、提交对象、Vite实际返回字节一致。

最终v-if后仅补受影响body：Chromium/WebKit各212项通过，两个实际生产宿主×manager/official×same/lower/none/unknownCurrent共16态/引擎。same/lower/none只有当前版本、详情标题、关闭按钮，没有箭头/确认/重启说明，也没有.release-notes整块。unknownCurrent有remote候选时精确说明仍可读、无确认。页面/资源/外网错误均0。已目检最终same与unknown截图。

连续复用：前一实际SHA c2125ea96b20b501327fe4e01c01e06fa74b20dca526bc7cdbe776ae0e2cb26d 已双引擎各通过286项完整版本/pending冻结合同、156项update-hosts、2项原strictscale、372项宽窄几何/比例/滚动。最终与它仅差ReleaseNotes一行v-if及取消null回退；script setup和CSS逐字节相同。newer候选恒truthy、渲染props相同；pending从不进入!showingProgress子树，故冻结版本合同不变。按用户要求未重复无变化scale矩阵。旧报告保留原SHA，复用依据在continuous-source-reuse.json及精确diff，不冒称旧轮次直接运行了最终字节。

相对34a的完整源码差异保存在sources/UpdateCenter-b175-vs34a.diff；冻结原型maintenance-ui.js148-154依据也已保存。未采用未来能力或扩大无关文案范围。

本轮相关实际失败0，审阅范围内未发现新阻塞。所有报告/截图/命令/源码hash见handoff.json、mapping-correction.json、reports/index.json。仅写本目录，未改产品/docs/发布。新包及其完整CI/原生验收由主线程继续；旧34a包只保留历史证据。
''')
files=[]
for p in sorted(R.rglob('*')):
 if p.is_file() and 'browser-temp' not in p.relative_to(R).parts and p.name not in ['SHA256SUMS','artifact-manifest.json']:
  files.append({'path':str(p.relative_to(R)),'bytes':p.stat().st_size,'sha256':h(p)})
save('artifact-manifest.json',{'root':str(R),'files':files});(R/'SHA256SUMS').write_text(''.join(x['sha256']+'  '+x['path']+chr(10) for x in files))
print(json.dumps({'result':'pass','commit':FINAL,'sha256':final['sourceSha256'],'finalBodyPerEngine':212,'priorPerEngine':[286,156,2,372],'failures':0,'handoff':str(R/'handoff.json'),'files':len(files)}))
