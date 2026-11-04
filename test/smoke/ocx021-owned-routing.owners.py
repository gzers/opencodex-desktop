#!/usr/bin/env python3
"""Candidate owner wiring spike only; never a current-product acceptance pass.

Actual owners: bodyless capability reload(new), safe GET + no-op roster PUT,
and Codex config/value/write CAS. Reuses a named immutable d20 save helper.
No build, global restart, post-rename manual client/catalog edit, or --model.
"""
import argparse
import base64
import hashlib
import hmac
import importlib.util
import json
import os
from pathlib import Path
import shutil
import sys
import time
from types import SimpleNamespace
import urllib.error
import urllib.request

sys.dont_write_bytecode = True
source = Path(__file__).with_name('ocx021-owned-routing.py')
spec = importlib.util.spec_from_file_location('owned_routing', source)
base = importlib.util.module_from_spec(spec); spec.loader.exec_module(base)

def fingerprint(path):
    value = path.stat()
    return {'sha256': base.sha(path), 'bytes': value.st_size,
            'inode': value.st_ino, 'mtimeNs': value.st_mtime_ns}

class OwnerSpike(base.Runner):
    def build(self):
        prior = self.args.immutable_build.resolve()
        assert prior.is_relative_to(base.EVIDENCE.resolve())
        metadata = json.loads((prior / 'result.json').read_text())
        assert metadata['codeHead'] == self.args.immutable_core
        binary = Path(metadata['runtimeRoot']) / 'target/debug/ocx021-owned-routing'
        assert base.sha(binary) == metadata['helperSha256']
        self.helper = self.root / 'build/immutable-save-helper'
        shutil.copy2(binary, self.helper)
        self.result.update(scope='candidate real-owner wiring spike', candidateOnly=True,
            productAcceptancePass=False, releaseGatePassed=False, productionRecompiled=False,
            immutableSaveCore=self.args.immutable_core, immutableBuildEvidence=str(prior),
            helperSha256=base.sha(self.helper), globalRestartEndpointCalled=False)
        shutil.copyfile(prior / 'source-after-build.json', self.out / 'immutable-save-helper-source.json')
        for name in ['helper-Cargo.toml', 'helper-Cargo.lock']:
            shutil.copyfile(prior / name, self.out / name)
        self.log('Using exact immutable d20 production save helper; no build')
    def run_true_route_guards(self):
        self.result['productionTrueRouteGuards'] = {'scope': 'not rerun; independent earlier d20 evidence',
            'evidence': str(self.args.immutable_build / 'production-true-route-guards.stdout')}
    def client_models(self, label):
        if label == 'codex-before':
            client = self.root / 'home/.codex/config.toml'
            # The only manual client edit: an OLD current selection fixture.
            client.write_text('model = "Krill-1/' + base.RAW_MODELS[0] + '"\n' + client.read_text())
            shutil.copyfile(client, self.out / 'old-current-model-fixture.toml')
            self.result['oldCurrentModelSeededBeforeRename'] = True
        return super().client_models(label)
    def attested(self):
        record_path = self.root / 'official/runtime-port.json'
        original = record_path.read_bytes(); record = json.loads(original)
        assert record['port'] == self.proxy_port and record['pid'] > 0
        nonce = base64.urlsafe_b64encode(os.urandom(32)).decode().rstrip('=')
        req = urllib.request.Request(f'http://127.0.0.1:{self.proxy_port}/healthz',
            headers={'x-opencodex-attestation-challenge': nonce})
        with self.http.open(req, timeout=8) as response:
            health = json.load(response); proof = response.headers.get('x-opencodex-attestation-proof', '')
        wanted = base64.urlsafe_b64encode(hmac.new(record['attestationSecret'].encode(),
            f'opencodex-local-management-v1\n{nonce}\n{record["pid"]}\n{record["port"]}'.encode(), hashlib.sha256).digest()).decode().rstrip('=')
        assert hmac.compare_digest(proof, wanted)
        assert health['service'] == 'opencodex' and health['pid'] == record['pid']
        assert record_path.read_bytes() == original
        return original, record, nonce, health
    def owner(self, method, path, label, body=None, reload_name=None):
        assert (method, path) in {('GET', '/api/config'), ('PUT', '/api/providers'),
                                  ('POST', '/api/providers/reload'), ('POST', '/api/sync')}
        original, record, nonce, health = self.attested()
        headers = {}
        if reload_name:
            assert path == '/api/providers/reload' and health['providerReloadCapability'] == 'v1'
            expires = int(time.time() * 1000) + 10000
            payload = f'opencodex-local-provider-reload-v1\n{nonce}\nPOST\n{path}\n{reload_name}\n{record["pid"]}\n{record["port"]}\n{expires}'
            capability = base64.urlsafe_b64encode(hmac.new(record['attestationSecret'].encode(),
                payload.encode(), hashlib.sha256).digest()).decode().rstrip('=')
            headers.update({'x-opencodex-provider-reload-expected-pid': str(record['pid']),
                'x-opencodex-provider-reload-nonce': nonce, 'x-opencodex-provider-reload-expires-at': str(expires),
                'x-opencodex-provider-reload-name': reload_name, 'x-opencodex-provider-reload-capability': capability})
        else:
            token = (self.root / 'official/admin-api-token').read_text().strip()
            assert token.startswith('ocx_admin_') and len(token) == 53
            headers['Authorization'] = 'Bearer ' + token
        encoded = json.dumps(body).encode() if body is not None else (b'' if method == 'POST' else None)
        if body is not None: headers['Content-Type'] = 'application/json'
        if method == 'POST': headers['Content-Length'] = str(len(encoded))
        if body is not None:
            # DTOs must never transfer provider secrets or authorization material.
            wire = json.dumps(body)
            assert base.KEY not in wire and 'ocx_admin_' not in wire
            def check(value):
                if isinstance(value, dict):
                    assert not set(value).intersection({'apiKey', 'apiKeyPool', 'headers', 'accessToken', 'refreshToken'})
                    for child in value.values(): check(child)
                elif isinstance(value, list):
                    for child in value: check(child)
            check(body)
            base.dump(self.out / (label + '-request.json'), body)
        request = urllib.request.Request(f'http://127.0.0.1:{self.proxy_port}{path}', method=method,
                                        data=encoded, headers=headers)
        try: response = self.http.open(request, timeout=55)
        except urllib.error.HTTPError as error: response = error
        with response: status, result = response.code, json.load(response)
        assert (self.root / 'official/runtime-port.json').read_bytes() == original
        base.dump(self.out / (label + '.json'), {'method': method, 'path': path,
            'attestationVerified': True, 'httpStatus': status, 'result': result})
        self.result['commands'].append({'name': label, 'method': method, 'path': path, 'httpStatus': status})
        self.log(f'{label}: HTTP {status}')
        if status != 200: raise RuntimeError(f'{label}: HTTP {status}: {result}')
        return result
    def automatic_chain(self):
        self.result.update(postRenameManualClientOrCatalogWrites=0, explicitModelOverrideAfterRename=False,
            currentProductStillRequiresFix=True, candidateOwnersPassed=False)
        raw = self.root / 'official/config.json'
        saved = raw.read_bytes(); disk = json.loads(saved); disk_fp = fingerprint(raw)
        assert set(disk['providers']) == {'Krill'} and disk['defaultProvider'] == 'Krill'
        stale = self.owner('GET', '/api/config', 'live-before-owner')
        self.result['liveBefore'] = {'providers': list(stale['providers']), 'defaultProvider': stale['defaultProvider']}
        assert 'Krill-1' in stale['providers'] and stale['defaultProvider'] == 'Krill-1'
        self.owner('POST', '/api/providers/reload', 'reload-new-provider', reload_name='Krill')
        reloaded = self.owner('GET', '/api/config', 'live-after-single-reload')
        assert 'Krill' in reloaded['providers']
        self.result['liveAfterSingleReload'] = {'providers': list(reloaded['providers']), 'defaultProvider': reloaded['defaultProvider']}
        assert raw.read_bytes() == saved, 'single provider reload changed original config bytes'
        derived = {'hasApiKey', 'hasHeaders', 'initialModelSelection', 'xaiResponsesOptInState'}
        rows, removed = {}, {}
        for name in disk['providers']:
            row = reloaded['providers'][name]
            removed[name] = sorted(set(row).intersection(derived))
            rows[name] = {key: value for key, value in row.items() if key not in derived}
        editor = {'defaultProvider': disk['defaultProvider'], 'providers': rows}
        body = {'baseline': editor, 'next': json.loads(json.dumps(editor))}
        assert body['baseline'] == body['next']
        base.dump(self.out / 'batch-dto-provenance.json', {'source': 'GET /api/config new row after capability reload',
            'providerSetSource': 'exact provider keys from already saved owned disk config',
            'defaultSource': 'already saved owned disk config', 'removedOnlyExplicitDerivedFields': removed,
            'baselineEqualsNext': True, 'secretsTransferred': False})
        before_put = fingerprint(raw)
        batch = self.owner('PUT', '/api/providers', 'noop-roster-owner', body)
        after_put = fingerprint(raw)
        assert raw.read_bytes() == saved, 'no-op roster owner changed original config bytes'
        assert before_put == after_put == disk_fp, 'no-op roster owner rewrote config despite matching content'
        live = self.owner('GET', '/api/config', 'live-after-batch-owner')
        self.result['liveAfterBatch'] = {'providers': list(live['providers']), 'defaultProvider': live['defaultProvider']}
        assert set(live['providers']) == {'Krill'} and live['defaultProvider'] == 'Krill'
        base.dump(self.out / 'disk-noop-proof.json', {'afterSave': disk_fp, 'beforePut': before_put,
            'afterPut': after_put, 'bytesEqual': True, 'responseHasChangedField': 'changed' in batch,
            'changedFalseBasis': 'baseline equals next; original bytes/hash/inode/mtime unchanged; published changed=false branch still adopts live candidate'})
        sync = self.owner('POST', '/api/sync', 'official-producer-after-owners')
        assert sync['status'] == 'applied' and sync['ok'] and sync['refreshOutcome'] == 'committed'
        assert raw.read_bytes() == saved, 'catalog convergence changed saved config content'
        client = self.root / 'home/.codex/config.toml'
        shutil.copyfile(client, self.out / 'client-before-cas.toml')
        old_model = 'Krill-1/' + base.RAW_MODELS[0]
        new_model = 'Krill/' + base.RAW_MODELS[0]
        rpc = base.Rpc(self, 'codex-cas-owner')
        try:
            config = rpc.call(2, 'config/read', {'cwd': str(self.work), 'includeLayers': False})
            assert config['config']['model'] == old_model
            origin = config['origins']['model']
            assert origin['name']['type'] == 'user' and origin['name'].get('profile') is None
            assert Path(origin['name']['file']).resolve() == client.resolve()
            params = {'keyPath': 'model', 'value': new_model, 'mergeStrategy': 'replace',
                'filePath': origin['name']['file'], 'expectedVersion': origin['version']}
            base.dump(self.out / 'actual-cas-owner-request.json', {'method': 'config/value/write', 'params': params})
            written = rpc.call(3, 'config/value/write', params)
            base.dump(self.out / 'actual-cas-owner-result.json', written)
            assert written['status'] == 'ok' and written.get('overriddenMetadata') is None
            assert Path(written['filePath']).resolve() == client.resolve()
            reread = rpc.call(4, 'config/read', {'cwd': str(self.work), 'includeLayers': False})
            base.dump(self.out / 'config-read-after-cas.json', reread)
            assert reread['config']['model'] == new_model
            assert reread['origins']['model']['version'] == written['version']
            # A meaningful CAS negative: the old version must not restore the old model.
            committed = client.read_bytes()
            try:
                rpc.call(5, 'config/value/write', dict(params, value=old_model))
            except RuntimeError as error:
                self.result['staleCasRejected'] = True
                base.dump(self.out / 'stale-cas-rejection.json', {'error': str(error), 'configBytesUnchanged': client.read_bytes() == committed})
            else: raise RuntimeError('official CAS owner accepted a stale expectedVersion')
            assert client.read_bytes() == committed
        finally: rpc.close()
        shutil.copyfile(client, self.out / 'client-after-cas.toml')
        fresh = base.Rpc(self, 'codex-after-owners')
        try:
            cfg = fresh.call(2, 'config/read', {'cwd': str(self.work), 'includeLayers': False})
            models = fresh.call(3, 'model/list', {'includeHidden': True, 'limit': 1000})
            thread = fresh.call(4, 'thread/start', {'cwd': str(self.work), 'ephemeral': True,
                'approvalPolicy': 'never', 'sandbox': 'read-only'})
            base.dump(self.out / 'fresh-client-config.json', cfg)
            base.dump(self.out / 'fresh-client-models.json', models)
            base.dump(self.out / 'fresh-default-thread.json', thread)
            ids = [row['id'] for row in models['data']]
            assert cfg['config']['model'] == thread['model'] == new_model
            assert new_model in ids and not any(model.startswith('Krill-1/') for model in ids)
        finally: fresh.close()
        assert not self.seen
        status, _, _ = self.command([self.codex, 'exec', '--json', '--skip-git-repo-check',
            '--sandbox', 'read-only', '--ephemeral', '--cd', self.work,
            '--output-last-message', self.root / 'owner-spike-last-message.txt',
            'Return the fixture text. Do not call any tools.'], 'default-model-local-turn', timeout=75, check=False)
        last = self.root / 'owner-spike-last-message.txt'
        if last.is_file(): shutil.copyfile(last, self.out / 'owner-spike-last-message.txt')
        requests = self.seen
        assert status == 0 and last.read_text().strip() == base.MARKER
        assert len(requests) == 1 and requests[0]['path'] == '/v1/responses'
        assert requests[0]['model'] == base.RAW_MODELS[0] and requests[0]['syntheticAuthorization']
        usage = [json.loads(line) for line in (self.root / 'official/usage.jsonl').read_text().splitlines() if line.strip()]
        matching = [row for row in usage if row.get('requestedModel') == new_model]
        base.dump(self.out / 'actual-owner-routing-usage.json', matching)
        assert len(matching) == 1 and matching[0]['provider'] == 'Krill'
        assert matching[0]['resolvedModel'] == base.RAW_MODELS[0]
        assert matching[0]['routeDecision']['routeKind'] != 'default-provider'
        assert raw.read_bytes() == saved
        self.result.update(status='candidate-real-owners-spike-passed', candidateOwnersPassed=True,
            noOpBatchRawDiskUnchanged=True, liveOldProviderRemoved=True, liveDefaultUpdated=True,
            casOwnerWroteCurrentModel=True, freshDefaultModelWithoutOverride=thread['model'],
            rawModelAtFixture=requests[0]['model'], actualUsageProvider=matching[0]['provider'],
            actualRouteKind=matching[0]['routeDecision']['routeKind'], localPosts=1,
            localResponseConsumed=True, productAcceptancePass=False, releaseGatePassed=False)

def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--immutable-build', type=Path, required=True)
    parser.add_argument('--immutable-core', required=True)
    args = parser.parse_args()
    options = SimpleNamespace(official=base.OFFICIAL, codex=base.CODEX,
        node=Path(shutil.which('node')), evidence=base.EVIDENCE, source_freeze_head=None,
        automatic_chain=True, manual_client=False, seed_installed_fixture=True,
        allow_owned_coordinator_files=True, reuse_build=None,
        immutable_build=args.immutable_build, immutable_core=args.immutable_core)
    runner = OwnerSpike(options)
    exit_code = 0
    try:
        runner.prepare(); runner.run()
    except Exception as error:
        runner.result.update(status='candidate-real-owners-spike-failed', error=str(error),
                             candidateOwnersPassed=False, productAcceptancePass=False)
        runner.log('FAILED: ' + str(error)); exit_code = 1
    finally:
        runner.finish()
        cleaned = []
        for raw in runner.result.get('ownedCoordinatorExactFiles', []):
            path = Path(raw)
            if path.is_file() and not path.is_symlink(): path.unlink(); cleaned.append(raw)
        base.dump(runner.out / 'owned-coordinator-cleanup.json', {'removed': cleaned, 'otherFilesTouched': False})
    return exit_code

if __name__ == '__main__': sys.exit(main())
