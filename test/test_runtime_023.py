"""Review/build guards plus optional full-archive source execution (never a service).

python3 test/test_runtime_023.py --archive OLD_TGZ --original ORIGINAL_TGZ \
  --bun ABSOLUTE_BUN --dependencies EXISTING_NODE_MODULES --output FRESH_TEMP
Without these flags, source integration is explicitly skipped, never reported as passed.
"""
import argparse
import copy
import importlib.util
import io
import json
from pathlib import Path
import subprocess
import sys
import tarfile
import tempfile
import unittest
from unittest.mock import patch

ROOT = Path(__file__).resolve().parents[1]
sys.dont_write_bytecode = True
spec = importlib.util.spec_from_file_location('runtime023_builder', ROOT/'.github/scripts/build_runtime_023.py')
builder = importlib.util.module_from_spec(spec)
spec.loader.exec_module(builder)
OPTIONS = None


class ReviewGuards(unittest.TestCase):
    def test_review_patch_and_paths_agree(self):
        review = json.loads(builder.REVIEW.read_bytes())
        self.assertEqual(builder.sha(builder.PATCH.read_bytes()), review['patchSha256'])
        self.assertEqual(tuple(row['path'] for row in review['files']), builder.SOURCE_PATHS)
        headers = [line[6:] for line in builder.PATCH.read_text().splitlines() if line.startswith('+++ b/')]
        self.assertEqual(headers, list(builder.SOURCE_PATHS))

    def test_wrong_base_rejected_before_output(self):
        with tempfile.TemporaryDirectory(prefix='ocxd023-runtime-name-', dir=builder.TEMP_ROOT) as directory:
            root = Path(directory)
            (root/'wrong.tgz').write_bytes(b'wrong base')
            with self.assertRaisesRegex(ValueError, 'Pinned maintained archive changed'):
                builder.build(root/'output', root/'wrong.tgz')
            self.assertFalse((root/'output').exists())

    def test_archive_path_escape_and_links_rejected(self):
        for name, link in [('package/../../escape', False), ('package/link', True)]:
            raw = io.BytesIO()
            with tarfile.open(fileobj=raw, mode='w:gz') as archive:
                member = tarfile.TarInfo(name)
                if link:
                    member.type, member.linkname = tarfile.SYMTYPE, '/tmp/escape'
                archive.addfile(member)
            with self.assertRaises(ValueError):
                builder.read_archive(raw.getvalue())


# This script imports the delivered src tree as-is. Only external execution/egress
# is blocked; no source function, module, catalog result or provider discovery is mocked.
SOURCE_TEST = r'''
import assert from 'node:assert/strict';
import { readFileSync, writeFileSync } from 'node:fs';
import { syncBuiltinESMExports } from 'node:module';
const externalAttempts = [];
let networkAttempts = 0;
globalThis.fetch = async (...args) => { networkAttempts++; throw Error('Network forbidden in runtime023 test'); };
// ESM exports are immutable; mutate the built-in CJS object and synchronize.
const cp = (await import('node:module')).createRequire(import.meta.url)('node:child_process');
for (const key of ['spawn', 'spawnSync', 'exec', 'execSync', 'execFile', 'execFileSync', 'fork']) {
  cp[key] = (...args) => { externalAttempts.push({key, command:String(args[0])}); throw Error('External process forbidden'); };
}
syncBuiltinESMExports();
for (const key of ['spawn', 'spawnSync']) {
  Bun[key] = (...args) => { externalAttempts.push({key:'Bun.'+key}); throw Error('External process forbidden'); };
}
const checks = [];
function check(label, actual, expected) { assert.deepEqual(actual, expected, label); checks.push(label); console.log('PASS '+label); }
const { loadConfig } = await import('./src/config.ts');
const { gatherRoutedModelsForCatalogGather } = await import('./src/codex/catalog/routed-gather.ts');
const { gatherFlightKey } = await import('./src/codex/catalog/gather-capture.ts');
const { applyProviderConfigHints } = await import('./src/codex/catalog/model-hints.ts');
const { deriveEntry } = await import('./src/codex/catalog/derive-entry.ts');
const { buildCatalogEntriesFromObservedState } = await import('./src/codex/catalog/build-entries.ts');
const { syncCatalogModels } = await import('./src/codex/catalog/retained-sync.ts');
const { routedSlug, decodeRoutedModelId } = await import('./src/providers/slug-codec.ts');
const { OPENAI_CODEX_PROVIDER_ID, OPENAI_API_PROVIDER_ID } = await import('./src/providers/openai-tiers.ts');
const configPath = process.env.OPENCODEX_HOME+'/config.json';
const catalogPath = process.env.CODEX_HOME+'/catalog.json';
let snapshotNumber=0;
const unknown = {future:{nested:['保留',42],active:false}};
const ids = ['model-1','vendor/original-model'];
let config = {
  port:4096, defaultProvider:'krill', defaultModelAliases:false, futureRoot:unknown,
  providers:{krill:{name:'Krill',adapter:'openai-chat',baseUrl:'https://runtime023.invalid/v1',
    liveModels:false, models:ids, selectedModels:ids, defaultAliases:false, futureProvider:unknown}}
};
writeFileSync(catalogPath, JSON.stringify({futureCatalog:unknown, models:[]}));
writeFileSync(process.env.CODEX_HOME+'/config.toml','model_catalog_json = '+JSON.stringify(catalogPath)+'\n');
function save() {writeFileSync(configPath,JSON.stringify(config));}
async function gathered() {
  save(); const loaded=loadConfig();
  check('config preserves provider name '+String(config.providers.krill?.name), loaded.providers.krill?.name, config.providers.krill?.name);
  check('config unknown root preserved',loaded.futureRoot,unknown);
  check('config unknown provider preserved',loaded.providers.krill?.futureProvider,unknown);
  const models = await gatherRoutedModelsForCatalogGather(loaded,{authStoreBuffer:null});
  return {loaded,models};
}
function entries(models) {
  return buildCatalogEntriesFromObservedState({template:null,gptSlugs:[],goModels:models,
    wsEnabled:false,multiAgentMode:'default',exactComboSlugs:new Set(),accountSelectors:[],
    suppressedBareNativeSlugs:new Set(),disabledNativeAccountSlugs:new Set(),multiAgentV2Enabled:false});
}
async function syncExpected(name) {
  config.providers.krill.name=name;
  const {loaded,models}=await gathered();
  const material=models.filter(m=>m.provider==='krill');
  check('original wire ids '+String(name),material.map(m=>m.id),ids);
  const generated=entries(models);
  for(const id of ids) {
    const slug=routedSlug('krill',id), row=generated.find(m=>m.slug===slug);
    check('selector display '+String(name)+' '+id,row?.display_name,name.trim() ? name.trim()+'/'+id : slug);
    check('route codec remains '+id,decodeRoutedModelId(slug.split('/')[1],ids),id);
  }
  const result=await syncCatalogModels(loaded);
  check('actual catalog committed '+String(name),result.refreshOutcome,'committed');
  check('actual catalog path in H Codex home',result.path,catalogPath);
  const disk=JSON.parse(readFileSync(catalogPath,'utf8'));
  writeFileSync(process.env.RUNTIME023_EVIDENCE+'/catalog-'+(++snapshotNumber)+'.json',JSON.stringify(disk,null,2));
  check('catalog unknown root preserved',disk.futureCatalog,unknown);
  for(const id of ids) {
    const slug=routedSlug('krill',id);
    check('on-disk name '+String(name)+' '+id,disk.models.find(m=>m.slug===slug)?.display_name,
      name.trim() ? name.trim()+'/'+id : slug);
  }
  const persisted=JSON.parse(readFileSync(configPath,'utf8'));
  check('persisted unknown root unchanged',persisted.futureRoot,unknown);
  check('persisted unknown provider unchanged',persisted.providers.krill.futureProvider,unknown);
  return {loaded,models,disk};
}
const first=await syncExpected('Krill');
const oldKey=gatherFlightKey(first.loaded);
const second=await syncExpected('KrillRenamed');
check('on-disk native unchanged after rename',second.disk.models.find(m=>m.slug==='gpt-5.5'),
  first.disk.models.find(m=>m.slug==='gpt-5.5'));
check('rename separates gather flight',gatherFlightKey(second.loaded)!==oldKey,true);
check('rename keeps route set',second.models.filter(m=>m.provider==='krill').map(m=>[m.provider,m.id]),
  first.models.filter(m=>m.provider==='krill').map(m=>[m.provider,m.id]));
await syncExpected('中文渠道');
await syncExpected('Krill Team Space');
await syncExpected('  中文 / Team  ');
await syncExpected('   ');
config.providers.krill.name='中文渠道';
config.providers.krill.modelDisplayNames={'model-1':'精确模型显示'};
let flight=await gathered();
check('modelDisplayNames wins',entries(flight.models).find(m=>m.slug==='krill/model-1')?.display_name,'精确模型显示');
check('explicit display sync committed',(await syncCatalogModels(flight.loaded)).refreshOutcome,'committed');
check('explicit display on disk',JSON.parse(readFileSync(catalogPath,'utf8')).models.find(m=>m.slug==='krill/model-1')?.display_name,'精确模型显示');
config.providers.krill.alias='short';
config.providers.krill.modelAliases={'vendor/original-model':'picked'};
flight=await gathered();
let generated=entries(flight.models);
check('provider alias keeps prior default',generated.find(m=>m.slug==='krill/model-1')?.display_name,'精确模型显示');
check('explicit model/provider aliases win',generated.find(m=>m.slug==='krill/vendor-original-model')?.display_name,'short/picked');
check('alias retains original wire id',flight.models.find(m=>m.id==='vendor/original-model')?.provider,'krill');
check('explicit alias sync committed',(await syncCatalogModels(flight.loaded)).refreshOutcome,'committed');
check('explicit alias on disk',JSON.parse(readFileSync(catalogPath,'utf8')).models.find(m=>m.slug==='krill/vendor-original-model')?.display_name,'short/picked');
delete config.providers.krill.modelDisplayNames;
flight=await gathered();
check('providerAlias retains historical raw label',entries(flight.models).find(m=>m.slug==='krill/model-1')?.display_name,'krill/model-1');
const p={...config.providers.krill}; delete p.alias; delete p.modelAliases;
const oldMaterial=applyProviderConfigHints('krill',p,{provider:'krill',id:'model-1'});
check('suppressed providerAlias retains canonical label',deriveEntry(null,'krill/model-1','',5,
  {...oldMaterial,providerAlias:null}).display_name,'krill/model-1');
const renamed=applyProviderConfigHints('krill',{...p,name:'新名字'},oldMaterial);
check('cached material reprojects current name',deriveEntry(null,'krill/model-1','',5,renamed).display_name,'新名字/model-1');
const removed={...p};delete removed.name;
check('cached name removed with config field',applyProviderConfigHints('krill',removed,oldMaterial).providerDisplayName,undefined);
for(const name of [undefined,null,42,{},'']) {
  const hinted=applyProviderConfigHints('krill',{...p,name},{provider:'krill',id:'model-1'});
  check('invalid or absent name ignored '+JSON.stringify(name),deriveEntry(null,'krill/model-1','',5,hinted).display_name,'krill/model-1');
}
const native=deriveEntry(null,'gpt-5.5','',9);
for(const id of [OPENAI_CODEX_PROVIDER_ID,OPENAI_API_PROVIDER_ID]) {
  const hinted=applyProviderConfigHints(id,{adapter:'openai-responses',name:'不可覆盖官方'},{provider:id,id:'gpt-5.5'});
  check('official provider name excluded '+id,hinted.providerDisplayName,undefined);
}
const nativeAgain=deriveEntry(null,'gpt-5.5','',9);
check('native official entry unchanged',nativeAgain,native);
check('native marketing label preserved',native.display_name!=='不可覆盖官方/gpt-5.5',true);
const combo={provider:'combo',id:'combo-1',alias:'gpt-5.5',nativeAlias:true,providerDisplayName:'忽略'};
check('bare native alias not projected',deriveEntry(null,'gpt-5.5','',5,combo).display_name,'gpt-5.5');
const template={...native, futureTemplate:unknown};
const before=structuredClone(template);
const row=deriveEntry(template,'krill/model-1','',5,renamed);
check('template path projects name',row.display_name,'新名字/model-1');
check('template unknown field preserved',row.futureTemplate,unknown);
check('template input unmodified',template,before);
// English legal-key rename is Desktop's responsibility; runtime consumes that saved key as-is.
config.providers={KrillNew:{...p,name:'KrillNew'}};config.defaultProvider='KrillNew';
save();
const legalModels=await gatherRoutedModelsForCatalogGather(loadConfig(),{authStoreBuffer:null});
check('saved English runtime key used',legalModels.map(m=>m.provider),['KrillNew','KrillNew']);
check('saved English display',entries(legalModels).find(m=>m.slug==='KrillNew/model-1')?.display_name,'KrillNew/model-1');
check('saved English key sync committed',(await syncCatalogModels(loadConfig())).refreshOutcome,'committed');
check('saved English key on disk',JSON.parse(readFileSync(catalogPath,'utf8')).models.find(m=>m.slug==='KrillNew/model-1')?.display_name,'KrillNew/model-1');
check('no network attempted',networkAttempts,0);
console.log('BLOCKED_EXTERNAL_PROBES '+JSON.stringify(externalAttempts));
console.log('SOURCE_RESULT '+JSON.stringify({checks:checks.length,networkAttempts,externalExecution:'blocked',
  runtime:process.cwd(),configPath,catalogPath,bun:process.execPath,bunVersion:Bun.version}));
'''


@unittest.skipUnless(OPTIONS is not None, 'Full source integration requires explicit archive/Bun/dependency/output flags')
class FullSourceIntegration(unittest.TestCase):
    @classmethod
    def setUpClass(cls):
        args = OPTIONS
        cls.output = args.output.resolve()
        builder.require(not cls.output.exists(), 'A fresh test output directory is required')
        builder.require(args.bun.is_absolute() and args.bun.is_file(), 'Absolute installed Bun binary required')
        builder.require(args.dependencies.is_dir(), 'Installed dependency directory is missing')
        cls.report = builder.build(cls.output/'delivery', args.archive, args.original)
        cls.files, cls.modes = builder.read_archive((cls.output/'delivery'/cls.report['archive']).read_bytes())
        cls.runtime = cls.output/'I/runtime'
        for name, data in cls.files.items():
            target = cls.runtime/name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            target.chmod(cls.modes[name])
        (cls.runtime/'node_modules').symlink_to(args.dependencies.resolve(), target_is_directory=True)

    def run_sources(self, runtime, label):
        args = OPTIONS
        isolated = self.output/label
        for name in ('home', 'opencodex', 'codex', 'claude', 'bin', 'tmp', 'snapshots'):
            (isolated/name).mkdir(parents=True)
        # Fresh environment: never inherit proxy credentials, real config homes or PATH.
        env = {'HOME': str(isolated/'home'), 'USERPROFILE': str(isolated/'home'),
               'OPENCODEX_HOME': str(isolated/'opencodex'), 'CODEX_HOME': str(isolated/'codex'),
               'CLAUDE_CONFIG_DIR': str(isolated/'claude'), 'PATH': str(isolated/'bin'),
               'TMPDIR': str(isolated/'tmp'), 'XDG_CONFIG_HOME': str(isolated/'home/.config'),
               'XDG_CACHE_HOME': str(isolated/'home/.cache'), 'OCX_TEST_HOME_GUARD': '1',
               'OCX_REAL_HOME': str(Path.home()), 'OCX_NO_DESKTOP_HANDOFF': '1',
               'RUNTIME023_EVIDENCE': str(isolated/'snapshots'),
               'CODEX_CLI_PATH': str(isolated/'bin/absent-codex')}
        result = subprocess.run([str(args.bun), '--no-env-file', '-e', SOURCE_TEST],
                                cwd=runtime, env=env, capture_output=True, text=True, timeout=50)
        return result

    def test_01_delivered_source_config_to_disk_catalog(self):
        result = self.run_sources(self.runtime, 'H')
        (self.output/'runtime023-source-test.log').write_text(result.stdout + result.stderr)
        self.assertEqual(result.returncode, 0, (result.stdout + result.stderr)[-8000:])
        markers = [line.removeprefix('SOURCE_RESULT ') for line in result.stdout.splitlines()
                   if line.startswith('SOURCE_RESULT ')]
        self.assertEqual(len(markers), 1)
        summary = json.loads(markers[0])
        self.assertGreater(summary['checks'], 70)
        self.assertEqual(summary['networkAttempts'], 0)
        (self.output/'runtime023-source-test-summary.json').write_bytes(builder.json_bytes(summary))

    def test_02_untouched_archive_and_compatibility_hashes(self):
        self.assertEqual(self.report['installation'], 'desktop-integrated-runtime')
        policy = json.loads((ROOT/'.github/maintained-runtime.json').read_bytes())
        self.assertEqual(policy['installation'], self.report['installation'])
        old, old_modes = builder.read_archive(OPTIONS.archive.read_bytes())
        self.assertEqual(set(self.files), set(old))
        self.assertEqual(self.modes, old_modes)
        changed = sorted(name for name in self.files if self.files[name] != old[name])
        self.assertEqual(changed, sorted(['package.json', builder.COMPATIBILITY, *builder.SOURCE_PATHS]))
        records = {row['path']: row['sha256'] for row in json.loads(self.files[builder.COMPATIBILITY])['files']}
        for name in ['package.json', *builder.SOURCE_PATHS]:
            self.assertEqual(records[name], builder.sha(self.files[name]))
        self.assertEqual(self.files['bin/ocx.mjs'], old['bin/ocx.mjs'])
        self.assertTrue(self.report['reproducibility']['equalBytes'])

    def test_03_review_and_patch_drift_fail_closed(self):
        old, _ = builder.read_archive(OPTIONS.archive.read_bytes())
        review = json.loads(builder.REVIEW.read_bytes())
        wrong = copy.deepcopy(review)
        wrong['files'][0]['beforeSha256'] = '0'*64
        with self.assertRaisesRegex(ValueError, 'preimage drift'):
            builder.patched_files(old, wrong)
        with tempfile.TemporaryDirectory(prefix='ocxd023-runtime-name-', dir=builder.TEMP_ROOT) as directory:
            changed = Path(directory)/'changed.patch'
            changed.write_bytes(builder.PATCH.read_bytes()+b'\n')
            with patch.object(builder, 'PATCH', changed), self.assertRaisesRegex(ValueError, 'Reviewed patch changed'):
                builder.patched_files(old, review)

    def test_04_original_source_exposes_name_regression(self):
        old, modes = builder.read_archive(OPTIONS.archive.read_bytes())
        baseline = self.output/'baseline-I/runtime'
        for name, data in old.items():
            target = baseline/name
            target.parent.mkdir(parents=True, exist_ok=True)
            target.write_bytes(data)
            target.chmod(modes[name])
        (baseline/'node_modules').symlink_to(OPTIONS.dependencies.resolve(), target_is_directory=True)
        result = self.run_sources(baseline, 'baseline-H')
        (self.output/'runtime023-baseline-regression.log').write_text(result.stdout + result.stderr)
        self.assertNotEqual(result.returncode, 0)
        self.assertIn('selector display Krill model-1', result.stderr)
        self.assertIn('krill/model-1', result.stderr)


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    for flag in ('archive', 'original', 'bun', 'dependencies', 'output'):
        parser.add_argument('--'+flag, type=Path)
    args, remaining = parser.parse_known_args()
    if any(vars(args).values()):
        if not all(getattr(args, key) for key in ('archive', 'bun', 'dependencies', 'output')):
            parser.error('--archive, --bun, --dependencies and --output must be supplied together')
        OPTIONS = args
        # Decorators ran before CLI parsing; enable only the explicitly configured suite.
        FullSourceIntegration.__unittest_skip__ = False
    unittest.main(argv=[sys.argv[0], *remaining], verbosity=2)
