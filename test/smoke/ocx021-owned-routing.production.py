#!/usr/bin/env python3
"""Isolated production parent verification. No runtime without an explicit mode.

OLD installation/default fixture -> real save_with_context -> real public
prepare_saved_rename_restart -> fresh Codex without model override -> local
raw-model/new-provider route. No post-save test-only owner repair. A separate
mock-restart commit test uses a COPY and retains the actually prepared pending.
Run only once after parent cargo/check and explicit source freeze.
--copy-gate-only reruns solely the synthetic restart-ack copy fixture for an
already completed real production chain. It never repeats save/prepare/turn.
--default-npm-client exercises standard cached npm .bin/codex with copied
Node on controlled PATH and no forced native runtime. Production resolves the
same-package native owner; observers/exec keep invoking the original launcher.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import shutil
import subprocess
import sys
import time
from types import SimpleNamespace

sys.dont_write_bytecode = True
source = Path(__file__).with_name('ocx021-owned-routing.py')
spec = importlib.util.spec_from_file_location('owned_routing', source)
base = importlib.util.module_from_spec(spec); spec.loader.exec_module(base)

def fingerprint(path):
    stat = path.stat()
    return {'sha256':base.sha(path), 'bytes':stat.st_size, 'inode':stat.st_ino, 'mtimeNs':stat.st_mtime_ns}

class ProductionParent(base.Runner):
    def run_true_route_guards(self):
        # The main cache_refresh tests now exercise private production helpers.
        # Do not clone those helpers to make external test imports compile.
        # Parent's actual cargo checks remain their own source-bound evidence.
        self.result['parentChecks'] = {'source':'user-reported parent checks, not rerun by external helper',
            'cacheRefresh': '19/19 reported before final freeze',
            'liveAdapter': '15/15 reported before final freeze',
            'scope':'must be accompanied by parent source-bound final cargo/check receipts'}
    def helper_json(self, mode, label, extra=(), policy=None, timeout=60):
        _, out, _ = self.command([self.helper, mode, *extra],label,timeout=timeout,policy=policy)
        return json.loads(out)
    def bind_observer(self, observed):
        binding = observed['bindings']
        assert binding['attested'] is True
        integration, runtime = binding['codexIntegration'], binding['codexRuntime']
        assert integration['clientId']=='codex' and integration['installed'] is True
        assert integration['state'] in ['current','stale'], 'official old installation not current/stale; no fake installed state allowed'
        reported = Path(integration['configPath'])
        assert reported.is_absolute() and '..' not in reported.parts and reported.name=='config.toml'
        config = reported.resolve(strict=True)
        assert config.is_relative_to(self.root) and config.parent.name=='.codex'
        codex_home, home = config.parent, config.parent.parent
        assert home.is_relative_to(self.root) and codex_home.is_relative_to(home)
        raw = runtime['path']
        assert runtime['version'] and isinstance(raw,str)
        if self.default_npm_client:
            # settings.path is a display DTO and may be redacted. Compare it
            # against the actual official resolver's display projection, but
            # preserve both raw returns. Never de-mask/substitute this DTO for
            # production. Observers use the fixture's lexical npm symlink.
            trace = self.npm_resolver_trace
            selected = trace['rawRuntime']
            assert trace['actualPublishedResolver'] and not trace['seamsInjected']
            assert self.result['initialDefaultNpmRuntime']['source'] == 'path'
            assert runtime['source'] in ['path','configured'] and selected['source'] in ['path','configured']
            if runtime['source'] == 'configured' or selected['source'] == 'configured':
                persisted = trace['persistedRuntimeAtRead']
                assert persisted is not None and persisted['origin'] == 'discovered'
                assert persisted['command'] == self.npm_layout['launcher']
            assert runtime['version'] == selected['version'] == self.npm_layout['packageVersion']
            assert raw == trace['displayPath'], 'actual server display path disagrees with unmodified official resolver'
            executable = Path(selected['command'])
            assert executable.is_absolute() and '..' not in executable.parts
            assert str(executable) == self.npm_layout['launcher']
            assert executable.is_symlink() and executable.resolve() == Path(self.npm_layout['selector'])
            assert base.sha(executable.resolve()) == self.npm_layout['selectorSha256']
            assert not set(base.FORCED_CODEX_RUNTIME_ENV).intersection(self.env)
            assert self.env['PATH'] == self.npm_layout['path']
        else:
            # Native-mode historical runs: resolve only against the bound
            # HOME, never expanduser() or substitute a different config/model.
            executable = home / raw[2:] if raw.startswith(('~/','~\\')) else Path(raw)
            assert executable.is_absolute() and '..' not in executable.parts
            executable = executable.resolve(strict=True)
            assert executable.is_file()
            assert base.sha(executable)==self.observer_binary_sha, 'server selected a different/unverified native Codex runtime'
        if hasattr(self,'bound_config'):
            assert config==self.bound_config and executable==self.codex and home==self.bound_home
        self.codex=executable
        self.bound_config,self.bound_codex_home,self.bound_home=config,codex_home,home
        assert Path(self.env['CODEX_HOME']).resolve()==codex_home and Path(self.env['HOME']).resolve()==home
        self.result['officialClientBinding']={'rawCodexRuntime':runtime,'rawIntegration':integration,
            'resolvedExecutable':str(executable),'resolvedConfig':str(config),
            'resolvedCodexHome':str(codex_home),'tildeExpansionHome':str(home),
            'source':'actual attested settings + actual integration; no injected response',
            'observerLaunchMode':'unmodified npm selector via PATH Node' if self.default_npm_client else 'explicit native',
            'settingsDisplayPathReplacedForProduction':False,
            'nativeConfigurationOwnerResolution':'production prepare only; observer never resolves/replaces native owner'}
    def client_models(self,label):
        if label!='codex-before': return super().client_models(label)
        if self.default_npm_client:
            self.npm_resolver_trace=self.record_default_npm_resolver('official-default-npm-runtime-trace')
        else:self.observer_binary_sha=base.sha(self.codex)
        state=self.helper_json('observe-production-bindings','official-binding-before-rename')
        self.bind_observer(state)
        rpc=base.Rpc(self,'codex-unseeded-current-config')
        try:
            current=rpc.call(2,'config/read',{'cwd':str(self.work),'includeLayers':True})
            base.dump(self.out/'unseeded-effective-config.json',current)
            assert current['config'].get('profile') in [None,''], 'unbound active profile must fail before any rename/write'
            assert current['config'].get('model') in [None,''], 'fresh fixture must not overwrite a pre-existing user choice'
        finally:rpc.close()
        original=self.bound_config.read_text()
        self.bound_config.write_text('model = "Krill-1/'+base.RAW_MODELS[0]+'"\n'+original)
        shutil.copyfile(self.bound_config,self.out/'old-current-default-fixture.toml')
        base.dump(self.out/'old-default-fixture-source.json',{'seededBeforeRename':True,
            'configPathFromServer':str(self.bound_config),'currentSelection':'Krill-1/'+base.RAW_MODELS[0],
            'activeProfile':None,'postRenameManualClientWritesAllowed':False})
        rpc=base.Rpc(self,'codex-before-production-save')
        try:
            config=rpc.call(2,'config/read',{'cwd':str(self.work),'includeLayers':True})
            models=rpc.call(3,'model/list',{'includeHidden':True,'limit':1000})
            base.dump(self.out/'old-effective-config.json',config)
            base.dump(self.out/'old-client-models.json',models)
            assert config['config']['model']=='Krill-1/'+base.RAW_MODELS[0]
            assert config['origins']['model']['name']['type']=='user'
            assert config['origins']['model']['name'].get('profile') is None
            assert Path(config['origins']['model']['name']['file']).resolve()==self.bound_config
            return [row['id'] for row in models['data']]
        finally:rpc.close()
    def automatic_chain(self):
        self.result.update(scope='actual production prepare integration',productionPrepareInvoked=True,
            candidateOnly=False,globalRestartEndpointCalled=False,postRenameManualClientWrites=0,
            postRenameTestOnlyOwnerRepairCalls=0,explicitExecModelOverride=False,
            releaseGatePassed=False,desktopGuiVerified=False)
        raw=self.root/'official/config.json'
        disk_before=raw.read_bytes();disk_fp=fingerprint(raw)
        pending_path=self.root/'manager/manager-state/provider-client-renames.json'
        pending_before=pending_path.read_bytes();pending=json.loads(pending_before)
        assert self.result['productionSave']['clientRenamePending']['id']==pending['id']
        assert any(row['from']=='Krill-1' and row['to']=='Krill' for row in pending['bindings'])
        base.dump(self.out/'pending-after-production-save.json',pending)
        receipt=self.helper_json('production-prepare','actual-production-prepare')
        assert receipt['entry']=='cache_refresh::prepare_saved_rename_restart' and receipt['success']
        assert receipt['pendingId']==pending['id'] and receipt['pendingRetained'] is True
        assert pending_path.read_bytes()==pending_before, 'production prepare acknowledged pending before actual restart'
        assert raw.read_bytes()==disk_before and fingerprint(raw)==disk_fp, 'production prepare rewrote saved provider config'
        self.result['productionPrepare']=receipt
        state=self.helper_json('observe-production-bindings','official-binding-after-prepare')
        self.bind_observer(state)
        live=state['bindings']['publicConfig']
        assert set(live['providers'])=={'Krill'} and live['defaultProvider']=='Krill'
        base.dump(self.root/'safe-public-state-for-commit.json',live)
        base.dump(self.out/'saved-disk-identity-through-production-prepare.json',{
            'before':disk_fp,'after':fingerprint(raw),'bytesUnchanged':True})
        shutil.copyfile(self.bound_config,self.out/'client-after-production-prepare.toml')
        catalog=self.bound_codex_home/'opencodex-catalog.json'
        shutil.copyfile(catalog,self.out/'catalog-after-production-prepare.json')
        rpc=base.Rpc(self,'fresh-codex-after-production-prepare')
        wanted='Krill/'+base.RAW_MODELS[0]
        try:
            cfg=rpc.call(2,'config/read',{'cwd':str(self.work),'includeLayers':True})
            models=rpc.call(3,'model/list',{'includeHidden':True,'limit':1000})
            thread=rpc.call(4,'thread/start',{'cwd':str(self.work),'ephemeral':True,
                'approvalPolicy':'never','sandbox':'read-only'})
            for name,data in [('effective-config-after-prepare.json',cfg),('models-after-prepare.json',models),('default-thread-after-prepare.json',thread)]:
                base.dump(self.out/name,data)
            assert cfg['config']['model']==thread['model']==wanted
            ids=[row['id'] for row in models['data']]
            assert wanted in ids and not any(model.startswith('Krill-1/') for model in ids)
            assert Path(cfg['origins']['model']['name']['file']).resolve()==self.bound_config
        finally:rpc.close()
        assert not self.seen
        status,_,_=self.command([self.codex,'exec','--json','--skip-git-repo-check','--sandbox','read-only',
            '--ephemeral','--cd',self.work,'--output-last-message',self.root/'production-prepare-last-message.txt',
            'Return the fixture text. Do not call any tools.'],'production-prepare-default-local-turn',timeout=75,check=False)
        last=self.root/'production-prepare-last-message.txt'
        if last.is_file():shutil.copyfile(last,self.out/'production-prepare-last-message.txt')
        assert status==0 and last.read_text().strip()==base.MARKER
        assert len(self.seen)==1 and self.seen[0]['model']==base.RAW_MODELS[0] and self.seen[0]['syntheticAuthorization']
        usage=[json.loads(line) for line in (self.root/'official/usage.jsonl').read_text().splitlines() if line.strip()]
        routed=[row for row in usage if row.get('requestedModel')==wanted]
        base.dump(self.out/'actual-production-prepare-routing.json',routed)
        assert len(routed)==1 and routed[0]['provider']=='Krill' and routed[0]['resolvedModel']==base.RAW_MODELS[0]
        assert routed[0]['routeDecision']['routeKind']=='explicit-provider'
        assert pending_path.read_bytes()==pending_before
        # Safe simulated completion: only a copied manager/home talks to a new
        # synthetic attested fixture. The real intent is retained for real ack.
        policy=self.root/'safe-commit-simulation.sb'
        policy.write_text(base.sandbox_policy(self.root)+
            '(allow network-outbound (remote ip "localhost:*"))\n'
            '(allow network-bind (local ip "localhost:*"))\n'
            '(allow network-inbound (local ip "localhost:*"))\n')
        shutil.copyfile(policy,self.out/'safe-commit-simulation.sb')
        simulated=self.helper_json('simulate-owned-restart-commit','production-safe-commit-simulation',
            extra=(receipt['pendingId'],),policy=policy)
        assert simulated['simulationPassed'] and simulated['originalPreparedPendingRetained']
        assert pending_path.read_bytes()==pending_before
        self.result.update(status='passed-production-prepare-and-safe-commit-copy',
            productionPrepareIntegrationPassed=True,currentDefaultMatchesNewDirectory=True,
            actualUsageProvider='Krill',rawModelAtEndpoint=base.RAW_MODELS[0],localPosts=1,
            localResponseConsumed=True,pendingId=receipt['pendingId'],actualPendingRetained=True,
            actualPendingPath=str(pending_path),safeRestartCommitCopy=simulated,
            automaticClientIntegrationVerified=True,automaticDirectoryGatePassed=True,
            actualDesktopRestartVerified=False,releaseGatePassed=False)
        if self.default_npm_client:
            self.result.update(defaultNpmProductionPreparePassed=True,
                upstreamInitialRuntimeSource='path',actualObserverAndExecEntry=str(self.codex),
                upstreamSettingsRuntimeSource=self.result['officialClientBinding']['rawCodexRuntime']['source'],
                isolatedNode=str(self.node),forcedNativeRuntimeVariablesPresent=False,
                postRenameClientConfigManuallyRepaired=False,
                productionOwnerSamePackageNativeResolution='exercised by actual production prepare; no test owner substitution')

def copy_gate_only(evidence,freeze):
    """Fix/rerun only this smoke's copied restart transport fixture.

    The completed real prepare/client/model-turn evidence is required and is
    never re-executed or overwritten. The linked product inputs must still match
    that run's exact freeze. Only the helper's commit-fixture module is rebuilt.
    """
    run=evidence.resolve()
    if not run.is_relative_to(base.EVIDENCE.resolve()):raise ValueError('copy gate evidence outside authorized directory')
    original=json.loads((run/'result.json').read_text())
    if original['sourceFreezeHead']!=freeze:raise ValueError('copy gate freeze differs from original production run')
    identity=base.verify_source_freeze(freeze)
    prepared=json.loads((run/'actual-production-prepare.stdout').read_text())
    assert prepared['success'] and prepared['pendingRetained'] and not prepared['globalRestartInvoked']
    cfg=json.loads((run/'effective-config-after-prepare.json').read_text())
    thread=json.loads((run/'default-thread-after-prepare.json').read_text())
    route=json.loads((run/'actual-production-prepare-routing.json').read_text())
    wanted='Krill/'+base.RAW_MODELS[0]
    assert cfg['config']['model']==thread['model']==wanted
    assert len(route)==1 and route[0]['provider']=='Krill' and route[0]['resolvedModel']==base.RAW_MODELS[0]
    assert (run/'production-prepare-last-message.txt').read_text().strip()==base.MARKER
    assert original['allOwnedChildrenReaped'] and all(value==61 for value in original['ownedPortConnectResultsAfterCleanup'].values())
    root=Path(original['runtimeRoot'])
    pending_path=root/'manager/manager-state/provider-client-renames.json'
    actual_pending=pending_path.read_bytes()
    assert json.loads(actual_pending)['id']==prepared['pendingId']
    supplement=run/('copy-gate-only-'+time.strftime('%Y%m%d-%H%M%S'))
    supplement.mkdir(exist_ok=False)
    base.dump(supplement/'source-freeze-before.json',identity)
    stage=root/'safe-restart-commit-copy'
    if stage.exists():stage.rename(root/('safe-restart-commit-copy-preserved-'+time.strftime('%H%M%S')))
    module=Path(__file__).with_name('ocx021-owned-routing.commit.rs')
    destination=root/'build/src/production-commit-simulation.rs'
    shutil.copyfile(module,supplement/'corrected-commit-fixture.rs')
    shutil.copyfile(destination,supplement/'previous-commit-fixture.rs')
    shutil.copyfile(module,destination)
    original_binary=root/'target/debug/ocx021-owned-routing'
    binary=root/'build/copy-gate-only-helper'
    real_binary=root/'original-real-prepare-helper'
    if not real_binary.exists():shutil.copy2(original_binary,real_binary)
    assert base.sha(real_binary)==original['helperSha256']
    library=root/'target/debug/deps/libopencodex_desktop_lib.rlib'
    library_before=base.sha(library)
    env=json.loads((run/'environment.json').read_text())
    toolchain=Path('/Users/ezio/.rustup/toolchains/stable-aarch64-apple-darwin/bin')
    # Reuse exactly the extern/native paths recorded by the real build. Running
    # Cargo again could rebuild the product merely because a test-only commit
    # advanced the embedded HEAD; direct rustc compiles only this smoke helper.
    required={'opencodex_desktop_lib','axum','serde_json','base64','hmac','sha2','keyring','tokio'}
    externs,native_paths={},set()
    for line in (run/'build.stdout').read_text().splitlines():
        try:row=json.loads(line)
        except ValueError:continue
        if row.get('reason')=='compiler-artifact' and row['target']['name'] in required:
            files=[name for name in row['filenames'] if name.endswith('.rlib')]
            if files:externs[row['target']['name']]=files[0]
        if row.get('reason')=='build-script-executed':native_paths.update(row.get('linked_paths',[]))
    assert set(externs)==required
    command=['/usr/bin/sandbox-exec','-f',str(root/'build.sb'),str(toolchain/'rustc'),
        '--crate-name','ocx021_owned_routing','--edition=2021',str(root/'build/src/main.rs'),
        '--crate-type','bin','--emit=link','-C','debuginfo=0',
        '-L','dependency='+str(root/'target/debug/deps'),'-o',str(binary)]
    for name,path in sorted(externs.items()):command.extend(['--extern',name+'='+path])
    for path in sorted(native_paths):command.extend(['-L',path])
    with (supplement/'helper-only-build.stdout').open('w') as stdout,(supplement/'helper-only-build.stderr').open('w') as stderr:
        result=subprocess.run(command,env=env,cwd=root/'work',stdin=subprocess.DEVNULL,stdout=stdout,stderr=stderr,timeout=60)
    base.dump(supplement/'helper-only-build-command.json',{'argv':command,'exitCode':result.returncode})
    if result.returncode:raise RuntimeError('copy fixture helper build failed: '+str(supplement))
    library_after=base.sha(library)
    if library_before!=library_after:raise RuntimeError('product library changed during helper-only correction; do not relabel the real runtime evidence')
    base.verify_source_freeze(freeze)
    policy=root/'safe-commit-simulation.sb'
    command=['/usr/bin/sandbox-exec','-f',str(policy),str(binary),'simulate-owned-restart-commit',prepared['pendingId']]
    environment=json.loads((run/'environment.json').read_text())
    result=subprocess.run(command,env=environment,cwd=root/'work',stdin=subprocess.DEVNULL,capture_output=True,text=True,timeout=60)
    (supplement/'safe-commit-only.stdout').write_text(result.stdout)
    (supplement/'safe-commit-only.stderr').write_text(result.stderr)
    base.dump(supplement/'safe-commit-command.json',{'argv':command,'exitCode':result.returncode})
    simulated=json.loads(result.stdout)
    assert result.returncode==0 and simulated['simulationPassed']
    assert simulated['incompleteRestartRetainedPendingCopy'] and simulated['successfulRestartAcknowledgedPendingCopy']
    assert simulated['originalPreparedPendingRetained'] and not simulated['realGlobalRestartCalled']
    assert simulated['restartHttpCallsToSyntheticFixture']==2
    assert pending_path.read_bytes()==actual_pending
    assert not (root/'safe-restart-commit-copy/manager/manager-state/provider-client-renames.json').exists()
    final=base.verify_source_freeze(freeze)
    assert final['currentProductSha256']==identity['currentProductSha256']
    base.dump(supplement/'source-freeze-after.json',final)
    report={'status':'passed-copy-restart-commit-only','sourceFreeze':freeze,'originalProductionRun':str(run),
        'realProductionHelperSha256':original['helperSha256'],'correctedCopyGateHelperSha256':base.sha(binary),
        'productLibraryRlibSha256Before':library_before,'productLibraryRlibSha256After':library_after,
        'productionLibraryRecompiled':False,'realSavePrepareClientAndModelRouteRepeated':False,
        'fixtureFix':'copy homeBinding uses production store::sha256; mock catalog receipt includes required committed/changed/degraded/notices fields; no guard/source/mapping/exit assertion weakened',
        'simulation':simulated,'actualPendingId':prepared['pendingId'],'actualPendingRetained':True,
        'actualPendingSha256':base.sha(pending_path),'productFilesStillExactFreeze':True,
        'actualDesktopRestartVerified':False,'releaseGatePassed':False}
    base.dump(supplement/'result.json',report)
    print(json.dumps(report,indent=2))
    return 0

def main():
    parser=argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--run',action='store_true')
    parser.add_argument('--default-npm-client',action='store_true',
        help='standard cached npm .bin symlink + JS selector and independent Node, no CODEX_CLI_PATH/native forcing; requires a new explicit product freeze')
    parser.add_argument('--check-source-freeze-only',action='store_true',
        help='read-only commit/product identity check; no temp runtime, build or official command')
    parser.add_argument('--copy-gate-only',type=Path,
        help='rerun only copied restart-ack simulation for an already successful real prepare/client/route; no real owner replay')
    parser.add_argument('--source-freeze-head')
    parser.add_argument('--cache-target',type=Path,default=base.CACHE_TARGET)
    args=parser.parse_args()
    if sum([args.run,args.check_source_freeze_only,args.copy_gate_only is not None])>1:
        parser.error('choose identity-only, single real run or independent copy gate')
    if args.copy_gate_only:
        if args.default_npm_client:parser.error('copy-only uses the archived actual runtime environment; do not change its client mode')
        if not args.source_freeze_head:parser.error('same complete real-run freeze SHA required')
        return copy_gate_only(args.copy_gate_only,args.source_freeze_head)
    if args.check_source_freeze_only:
        if not args.source_freeze_head:parser.error('complete new parent freeze SHA required')
        identity=base.verify_source_freeze(args.source_freeze_head)
        print(json.dumps({key:value for key,value in identity.items() if key!='currentProductSha256'},indent=2))
        return 0
    if not args.run:
        print(json.dumps({'status':'ready-explicit-product-freeze-required','helperMode':'production-prepare',
            'clientMode':'default-npm-client' if args.default_npm_client else 'explicit-native',
            'productionEntry':'cache_refresh::prepare_saved_rename_restart(root,context)',
            'pendingAck':'only copy in safe sync_cache_after_save(true) transport simulation; original retained',
            'sourceFreeze':'explicit complete new SHA; worktree compared even when HEAD equals freeze',
            'testOnlyHeadAdvance':'allowed only when guarded product files still match freeze',
            'globalRestartAllowed':False,'postRenameTestOnlyOwnerRepairAllowed':False,
            'runtimeOrCompilerInvokedByThisMode':False},indent=2));return 0
    if not args.source_freeze_head:parser.error('explicit parent source freeze SHA required')
    options=SimpleNamespace(official=base.OFFICIAL,codex=base.CODEX,node=Path(shutil.which('node')),
        evidence=base.EVIDENCE,source_freeze_head=args.source_freeze_head,automatic_chain=True,
        production_parent=True,manual_client=False,seed_installed_fixture=True,
        default_npm_client=args.default_npm_client,
        allow_owned_coordinator_files=True,reuse_build=None,cache_target=args.cache_target)
    runner=ProductionParent(options)
    status=0
    try:runner.prepare();runner.run()
    except Exception as error:
        runner.result.update(status='failed-production-prepare-check',error=str(error),releaseGatePassed=False)
        runner.log('FAILED: '+str(error));status=1
    finally:
        runner.finish()
        removed=[]
        for raw in runner.result.get('ownedCoordinatorExactFiles',[]):
            path=Path(raw)
            if path.is_file() and not path.is_symlink():path.unlink();removed.append(raw)
        base.dump(runner.out/'owned-coordinator-cleanup.json',{'removed':removed,'otherFilesTouched':False})
    if runner.result['status']!='passed-production-prepare-and-safe-commit-copy':return 1
    return status

if __name__=='__main__':sys.exit(main())
