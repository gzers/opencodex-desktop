"""Offline packaging contracts. Mock hdiutil does not prove a real disk image."""
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import shutil
import subprocess
import tempfile
from types import SimpleNamespace
import unittest
from unittest.mock import patch
ROOT = Path(__file__).resolve().parents[1]
spec = importlib.util.spec_from_file_location('package_macos_024', ROOT / '.github/scripts/package_macos_024.py')
pack = importlib.util.module_from_spec(spec); spec.loader.exec_module(pack)


class Package024(unittest.TestCase):
    def setUp(self):
        self.temp = tempfile.TemporaryDirectory(prefix='ocxd024-package-unit-')
        self.addCleanup(self.temp.cleanup)
        self.root = Path(self.temp.name)
        self.app = self.root / 'sealed source' / pack.APP
        (self.app / 'Contents/MacOS').mkdir(parents=True)
        (self.app / 'Contents/MacOS/opencodex-desktop').write_bytes(b'never executed fixture')
        (self.app / 'Contents/MacOS/opencodex-desktop').chmod(0o755)
        (self.app / 'Contents/Info.plist').write_bytes(plistlib.dumps({'CFBundleShortVersionString':pack.VERSION,'CFBundleIdentifier':'fixture-only','CFBundleExecutable':'opencodex-desktop'}))
        self.resources = self.app / 'Contents/Resources/runtime-bundle'; self.resources.mkdir(parents=True)
        self.source = b'official fixture bytes not a real archive'
        self.platform = b'complete platform fixture bytes'
        self.identity = {'runtime':{'version':'2.99.0','sha256':hashlib.sha256(self.source).hexdigest()}}
        self.descriptor = {'schemaVersion':1,'platform':'darwin-arm64','runtimeVersion':'2.99.0','runtimeArchiveSha256':self.identity['runtime']['sha256'],'archiveName':'complete.tar.gz','archiveBytes':len(self.platform),'archiveSha256':hashlib.sha256(self.platform).hexdigest()}
        raw = json.dumps(self.descriptor).encode()
        (self.resources / 'runtime-bundle.manifest.json').write_bytes(raw)
        (self.resources / 'runtime-bundle.manifest.sha256').write_text(hashlib.sha256(raw).hexdigest()+'\n')
        (self.resources / 'official-runtime.tgz').write_bytes(self.source)
        (self.resources / 'complete.tar.gz').write_bytes(self.platform)
        self.output = self.root / 'artifacts'; self.output.mkdir()
        self.dmg = self.output / 'candidate.dmg'; self.evidence = self.output / 'evidence.json'
        self.calls=[]; self.mounted=False; self.readonly=True; self.corrupt=False

    def command(self, argv, **kwargs):
        self.calls.append(argv)
        self.assertIsNone(kwargs.get('shell'))
        if argv[0] == '/usr/bin/ditto':
            shutil.copytree(argv[-2],argv[-1],symlinks=True)
        elif argv[:2] == ['/usr/bin/hdiutil','create']:
            Path(argv[-1]).write_bytes(b'fixture image only')
        elif argv[:2] == ['/usr/bin/hdiutil','attach']:
            mount=Path(argv[argv.index('-mountpoint')+1]); shutil.copytree(mount.parent/'payload',mount,dirs_exist_ok=True,symlinks=True)
            if self.corrupt: (mount/pack.APP/'Contents/MacOS/opencodex-desktop').write_bytes(b'changed signed bytes')
            self.mounted=True
            return subprocess.CompletedProcess(argv,0,plistlib.dumps({'system-entities':[{'mount-point':str(mount)}]}),b'')
        elif argv[:2] == ['/usr/bin/hdiutil','detach']: self.mounted=False
        return subprocess.CompletedProcess(argv,0,b'',b'')

    def run_pack(self):
        with patch.object(pack.sys,'platform','darwin'),patch.object(pack,'policy',return_value=self.identity),patch.object(pack.common.subprocess,'run',side_effect=self.command),patch.object(pack.os.path,'ismount',side_effect=lambda _:self.mounted),patch.object(pack.os,'statvfs',return_value=SimpleNamespace(f_flag=os.ST_RDONLY if self.readonly else 0)):
            return pack.package(self.app,self.dmg,self.evidence)

    def test_single_app_readonly_identity_and_immutable_source(self):
        before=pack.common.tree_manifest(self.app); result=self.run_pack()
        self.assertEqual(pack.common.tree_manifest(self.app),before)
        self.assertEqual(result['layout'],'single-app')
        self.assertEqual(result['mountedTopLevel'],['Applications','Install.txt',pack.APP])
        self.assertEqual(result['sourceAppTreeSha256'],result['mountedAppTreeSha256'])
        self.assertEqual(result['resources']['officialRuntimeSha256'],self.identity['runtime']['sha256'])
        self.assertTrue(result['readOnlyVerified']);self.assertTrue(result['appNeverLaunched']);self.assertFalse(result['installationPerformed']);self.assertFalse(result['releaseAcceptance'])
        self.assertTrue(result['cleanup']['ownMountDetached']);self.assertTrue(result['cleanup']['scratchRemoved'])
        self.assertFalse(any('--sign' in c or c[0] in ['/usr/bin/open','/bin/launchctl'] for c in self.calls))
        attach=next(c for c in self.calls if c[:2]==['/usr/bin/hdiutil','attach'])
        self.assertIn('-readonly',attach);self.assertIn('-nobrowse',attach)
        self.assertIn('~/Applications',pack.INSTALL);self.assertNotIn('ENTIRE',pack.INSTALL)

    def test_changed_source_or_descriptor_pin_is_refused_before_image(self):
        for name in ['official-runtime.tgz','runtime-bundle.manifest.sha256','complete.tar.gz']:
            original=(self.resources/name).read_bytes();(self.resources/name).write_bytes(b'changed')
            with self.assertRaises(ValueError):self.run_pack()
            self.assertFalse(self.dmg.exists());self.assertEqual(json.loads(self.evidence.read_text())['status'],'failed')
            self.assertFalse(any(c[:2]==['/usr/bin/hdiutil','create'] for c in self.calls))
            self.evidence.unlink();self.calls=[];(self.resources/name).write_bytes(original)

    def test_mounted_mutation_or_writable_volume_rejects_and_detaches(self):
        for mode in ['corrupt','writable']:
            self.corrupt=mode=='corrupt';self.readonly=mode!='writable'
            with self.assertRaises(ValueError):self.run_pack()
            result=json.loads(self.evidence.read_text());self.assertEqual(result['status'],'failed');self.assertTrue(result['cleanup']['ownMountDetached']);self.assertTrue(result['cleanup']['scratchRemoved']);self.assertFalse(self.dmg.exists());self.evidence.unlink()

    def test_existing_destinations_wrong_version_or_app_symlink_never_replace_original(self):
        self.dmg.write_bytes(b'original')
        with self.assertRaises(FileExistsError):self.run_pack()
        self.assertEqual(self.dmg.read_bytes(),b'original');self.dmg.unlink()
        with self.assertRaises(ValueError):pack.package(self.app,self.dmg,self.evidence,'0.2.23')
        self.assertEqual(self.calls,[])
        with self.assertRaises(ValueError):pack.common.destination(self.app/'artifact.dmg')

    def test_symlink_or_hardlinked_runtime_resources_and_wrong_application_version_are_refused(self):
        source=self.resources/'official-runtime.tgz'; original=source.read_bytes();source.unlink();source.symlink_to(self.root/'external.tgz');(self.root/'external.tgz').write_bytes(original)
        with patch.object(pack,'policy',return_value=self.identity):
            with self.assertRaises(ValueError):pack.resources(self.app)
        source.unlink();source.write_bytes(original);os.link(source,self.root/'second-link')
        with patch.object(pack,'policy',return_value=self.identity):
            with self.assertRaises(ValueError):pack.resources(self.app)
        (self.root/'second-link').unlink()
        plist=self.app/'Contents/Info.plist';value=plistlib.loads(plist.read_bytes());value['CFBundleShortVersionString']='0.2.23';plist.write_bytes(plistlib.dumps(value))
        with self.assertRaises(ValueError):self.run_pack()
        self.assertFalse(self.dmg.exists());self.assertEqual(self.calls,[])

    def test_exact_release_preflight_rejects_any_component_version_drift(self):
        spec=importlib.util.spec_from_file_location('verify024',ROOT/'.github/scripts/verify_release_inputs_024.py');verify=importlib.util.module_from_spec(spec);spec.loader.exec_module(verify)
        fixture=self.root/'repo';(fixture/'apps/desktop/tauri').mkdir(parents=True);(fixture/'apps/desktop/ui').mkdir();(fixture/'.github').mkdir()
        (fixture/'apps/desktop/tauri/tauri.conf.json').write_text(json.dumps({'version':'0.2.24'}));(fixture/'apps/desktop/ui/package.json').write_text(json.dumps({'version':'0.2.24'}));(fixture/'apps/desktop/tauri/Cargo.toml').write_text('[package]'+chr(10)+'version="0.2.24"'+chr(10));(fixture/'.github/runtime-platforms-024.json').write_bytes((ROOT/'.github/runtime-platforms-024.json').read_bytes())
        self.assertFalse(verify.verify(fixture,'v0.2.24')['releaseAcceptance'])
        with self.assertRaises(ValueError):verify.verify(fixture,'v0.2.23')
        (fixture/'apps/desktop/ui/package.json').write_text(json.dumps({'version':'0.2.23'}))
        with self.assertRaises(ValueError):verify.verify(fixture,'v0.2.24')

    def test_new_release_pipeline_uses_official_inputs_and_single_app_without_maintenance_generation(self):
        workflow=(ROOT/'.github/workflows/release.yml').read_text()
        self.assertIn('stage_runtime_bundle_024.py',workflow);self.assertIn('package_macos_024.py',workflow)
        self.assertNotIn('stage_runtime_bundle_023.py',workflow);self.assertNotIn('package_macos_023.py',workflow)
        self.assertNotIn('build_runtime_lifecycle_023.py',workflow);self.assertNotIn('maintained-runtime:',workflow)
        self.assertIn('Verify Windows release installers',workflow)
        self.assertEqual(pack.policy()['runtime']['distribution'],'official-npm')


if __name__=='__main__':unittest.main()
