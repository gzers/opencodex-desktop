from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
source=(BASE/'content.rs').read_text(encoding='utf-8')
for name in ['StartContentScope','StoppedContentScope']:
    block=source.split('struct '+name+' {',1)[1].split('\n}',1)[0]
    assert 'pub' not in block and 'Serialize' not in block and 'Deserialize' not in block
assert '#[derive' not in source
assert 'originals_require_unresumed(end)?' in source and 'OriginalActionKind::Start' in source
assert 'original_stop_action(end)' in source and 'same_original_owner' in source
assert 'original_content_processes(owner, end)' in source and 'routes.declared(owner.home_path(), end)?' in source
assert 'root.creation_filetime' in source and 'not_before' in source and 'not_after = Utc::now()' in source
core=(BASE/'tree/anchored/capture.rs').read_text(encoding='utf-8').split('fn restore_original_stopped(',1)[1].split('/// Native planning',1)[0]
assert core.index('ProductionStop::bind(')<core.index('assess_native_stopped_series(')<core.index('self.content_assessment = Some(assessment)')<core.index('capture_retirement(')<core.index('restore_qualified_stopped(')
assessment=(BASE/'tree/anchored/observation/series.rs').read_text(encoding='utf-8')
assert 'struct NativeContentAssessment {' in assessment and 'scope_digest: String' in assessment
original=assessment.split('pub(in crate::infrastructure::windows_client_originals024) fn assess_tree_series(',1)[1].split('pub(in crate::infrastructure::windows_client_originals024::tree::anchored) fn resume_trees',1)[0]
assert original.count('boundary(trees, claim)?')==2 and 'verify_custody_anchors' in original and 'archive_for_sequence' in original
prod=(BASE/'v3/journal/production.rs').read_text(encoding='utf-8')
assert '#[derive' not in prod and 'Deserialize' not in prod
assert all(token in prod for token in ['NormalExit','Absent','StopAccepted','RoutePin','StoppedContentScope','stopped_content_observations','stopped_source'])
restore=prod.split('fn restore_qualified_stopped(',1)[1]
assert restore.index('assessment.matches(')<restore.index('prepare_native_recovery(')<restore.index('recover_native_batch(')<restore.index('release_native_recovery(')<restore.index('seal_original_history(')
assert restore.count('guard.check(self, end)?')>=6
allcallers=[]
for p in BASE.rglob('*.rs'):
    for n,line in enumerate(p.read_text(encoding='utf-8').splitlines(),1):
        if '.restore_qualified_stopped(' in line:allcallers.append(dict(path=p.relative_to(CODE).as_posix(),line=n))
assert allcallers and all(row['path'].endswith('tree/anchored/capture.rs') for row in allcallers)
unchanged=[]
for path in ['apps/desktop/tauri/Cargo.toml','apps/desktop/tauri/Cargo.lock','apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs',
 'apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs','apps/desktop/tauri/src/modules/runtime/windows_lifecycle024_contract.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/bounded.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/handle_release.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination.rs','apps/desktop/tauri/src/infrastructure/windows_client_originals024/coordination/segments.rs']:
    raw=(CODE/path).read_bytes();prior=subprocess.check_output(['git','-C',str(CODE),'show',f'c259886cf:{path}'])
    assert raw.replace(b'\r\n',b'\n')==prior.replace(b'\r\n',b'\n'),path
    unchanged.append(dict(path=path,bytes=len(raw),sha256=sha(raw)))
gate=(BASE/'mod.rs').read_text(encoding='utf-8').split('pub(crate) fn require_durable_capture()',1)[1].split('/// Fixed NEW-session',1)[0]
assert 'Err(' in gate and 'Ok(' not in gate
with (ROOT/'production-boundary-verification.json').open('x',encoding='utf-8') as out:json.dump(dict(sourceBase='c259886cf',
    actualScopeConstructorsNoSerdeCloneOrPublicFields=True,actualNativeStartThenStopOwnerFamilyAndRoutesRequired=True,
    actualPolicyBeforeCurrentFrameTransfer=True,unchangedSharedPolicyAndCompleteBothTreeBarriers=True,
    actualTypedSourceStopGuardAroundNativeRecoveryReleaseHistory=True,productionRestoreActualCallers=allcallers,
    unchangedInputs=unchanged,publicStartOpen=False,ordinaryStopOrTypedClosedPublished=False,
    productionFactoriesRuntimeExecuted=False),out,indent=2)
print(json.dumps(dict(policyBeforeFrameTransfer=True,typedStopGuarded=True,unchangedInputs=len(unchanged),publicStartOpen=False)))
