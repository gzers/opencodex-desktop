#!/usr/bin/env python3
"""Probe current model/default references without a turn, model override or rebuild.

Reuses an explicitly identified immutable production helper build. Only OLD
client model is seeded manually; after rename only the real official producer
may write the client/catalog. All network is denied, including loopback.
"""
import argparse
import importlib.util
import json
from pathlib import Path
import shutil
import socket
import sys
from types import SimpleNamespace

def main():
    sys.dont_write_bytecode = True
    source = Path(__file__).with_name('ocx021-owned-routing.py')
    spec = importlib.util.spec_from_file_location('owned_routing', source)
    module = importlib.util.module_from_spec(spec); spec.loader.exec_module(module)
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--build-evidence', type=Path, required=True)
    parser.add_argument('--source-freeze-head', required=True)
    args = parser.parse_args()
    previous = args.build_evidence.resolve()
    assert previous.is_relative_to(module.EVIDENCE.resolve())
    build = json.loads((previous / 'result.json').read_text())
    assert build['codeHead'] == args.source_freeze_head
    helper = Path(build['runtimeRoot']) / 'target/debug/ocx021-owned-routing'
    assert module.sha(helper) == build['helperSha256']
    options = SimpleNamespace(official=module.OFFICIAL, codex=module.CODEX,
        node=Path(shutil.which('node')), evidence=module.EVIDENCE,
        source_freeze_head=args.source_freeze_head, automatic_chain=True,
        manual_client=False, seed_installed_fixture=True,
        allow_owned_coordinator_files=True, reuse_build=previous)
    runner = module.Runner(options)
    files = []
    try:
        runner.result.update(scope='current client model/default reference consistency',
            immutableHelperSourceCommit=args.source_freeze_head, productionRecompiled=False,
            generatedTurnRequests=0, explicitModelOverrideAfterRename=False,
            releaseGatePassed=False)
        runner.prepare()
        runner.helper = runner.root / 'build/immutable-d20-save-helper'
        shutil.copy2(helper, runner.helper)
        runner.result['helperSha256'] = module.sha(runner.helper)
        # A closed owned port is merely a routing target in this no-generation probe.
        with socket.socket() as sock:
            sock.bind(('127.0.0.1', 0)); runner.proxy_port = sock.getsockname()[1]
        runner.fixture_port = runner.proxy_port
        files = module.owned_coordinator_files(runner.root)
        for path in files:
            assert not path.exists() and not path.is_symlink()
        runner.sb.write_text(module.sandbox_policy(runner.root, owned_coordinators=True))
        shutil.copyfile(runner.sb, runner.out / 'runtime.sb')
        module.dump(runner.root / 'official/config.json', {
            'port': runner.proxy_port, 'hostname': '127.0.0.1',
            'defaultProvider': 'Krill-1',
            'providers': {'Krill-1': {'adapter': 'openai-responses', 'authMode': 'key',
                'baseUrl': 'http://127.0.0.1:' + str(runner.proxy_port) + '/v1',
                'apiKey': module.KEY, 'models': module.RAW_MODELS,
                'selectedModels': module.RAW_MODELS[:2], 'liveModels': False,
                'allowPrivateNetwork': True, 'newModelPolicy': 'off'}},
            'modelPickerOrder': ['Krill-1/' + model for model in module.RAW_MODELS[:2]],
            'disabledModels': ['Krill-1/' + module.RAW_MODELS[2]],
            'clientIntegrations': {'codex': True, 'grok': False, 'claude': False},
            'claudeCode': {'systemEnv': False, 'enabled': False},
            'syncResumeHistory': False, 'serviceHealthRecovery': {'enabled': False},
            'tokenGuardian': {'enabled': False}, 'oauthOpenBrowser': False})
        module.dump(runner.root / 'home/.codex/opencodex-catalog.json', runner.bundled)
        runner.seed_installation()
        bun = runner.official.parents[1] / 'bun/bin/bun.exe'
        probe = runner.root / 'coordinator-probe.mjs'
        shutil.copyfile(source.with_name('ocx021-owned-routing.coordinator.mjs'), probe)
        runner.command([bun, probe, runner.official, runner.root, runner.proxy_port], 'old-installation-producer')
        config = runner.root / 'home/.codex/config.toml'
        old = 'Krill-1/' + module.RAW_MODELS[0]
        new = 'Krill/' + module.RAW_MODELS[0]
        # OLD current user selection fixture, before saving/renaming anything.
        config.write_text('model = ' + json.dumps(old) + '\n' + config.read_text())
        shutil.copyfile(config, runner.out / 'client-config-before-rename.toml')
        module.dump(runner.out / 'old-current-model-fixture.json', {
            'seededBeforeRename': True, 'source': 'synthetic current user config root model',
            'model': old, 'postRenameManualClientWrites': 0})
        rpc = module.Rpc(runner, 'codex-default-before')
        try:
            before = rpc.call(2, 'config/read', {'cwd': str(runner.work), 'includeLayers': False})
            module.dump(runner.out / 'codex-config-before.json', before)
            assert before['config']['model'] == old
        finally: rpc.close()
        runner.command([runner.helper], 'production-save')
        runner.phase = 'after-rename'
        shutil.copyfile(config, runner.out / 'client-config-after-save-before-producer.toml')
        producer = runner.root / 'real-producer.mjs'
        shutil.copyfile(source.with_name('ocx021-owned-routing.defaults.mjs'), producer)
        runner.command([bun, producer, runner.official, runner.proxy_port], 'official-producer-current-default')
        shutil.copyfile(config, runner.out / 'client-config-after-producer.toml')
        shutil.copyfile(runner.root / 'home/.codex/opencodex-catalog.json', runner.out / 'catalog-after-producer.json')
        rpc = module.Rpc(runner, 'codex-default-after')
        try:
            actual = rpc.call(2, 'config/read', {'cwd': str(runner.work), 'includeLayers': False})
            models = rpc.call(3, 'model/list', {'includeHidden': True, 'limit': 1000})
            # No model field, no config override, no turn/start and no generation.
            thread = rpc.call(4, 'thread/start', {'cwd': str(runner.work),
                'ephemeral': True, 'approvalPolicy': 'never', 'sandbox': 'read-only'})
            module.dump(runner.out / 'codex-config-after.json', actual)
            module.dump(runner.out / 'codex-models-after.json', models)
            module.dump(runner.out / 'codex-default-thread-start.json', thread)
        finally: rpc.close()
        selectors = [row['id'] for row in models['data']]
        runner.result.update(currentModelBefore=old, currentModelAfterProducer=actual['config']['model'],
            expectedRenamedModel=new, defaultThreadModelWithoutOverride=thread['model'],
            selectorsAfterProducer=selectors, currentModelInCatalog=actual['config']['model'] in selectors,
            explicitProducerFallbackReported=False, postRenameManualClientWrites=0,
            configDefaultConsistencyPassed=actual['config']['model'] == new and thread['model'] == new)
        runner.result['status'] = ('passed-current-default-consistency' if runner.result['configDefaultConsistencyPassed']
            else 'confirmed-stale-current-client-default')
        runner.log(runner.result['status'])
    except Exception as error:
        runner.result.update(status='current-default-probe-failed', error=str(error))
        runner.log('FAILED: ' + str(error)); return 1
    finally:
        runner.finish()
        cleaned = []
        for path in files:
            if path.is_file() and not path.is_symlink(): path.unlink(); cleaned.append(str(path))
        module.dump(runner.out / 'owned-coordinator-cleanup.json', {'removed': cleaned, 'otherFilesTouched': False})
    return 0

if __name__ == '__main__': sys.exit(main())
