"""Build staging retains both the complete closure and the pinned recovery source."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SCRIPT=Path(__file__).resolve().parents[1]/'.github/scripts/stage_runtime_bundle_023.py'
spec=importlib.util.spec_from_file_location('stage_runtime_bundle_023',SCRIPT)
stage=importlib.util.module_from_spec(spec)
spec.loader.exec_module(stage)

class StageTests(unittest.TestCase):
    def test_stages_pinned_source_descriptor_and_resources_without_installing(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp)
            (root/'.github').mkdir()
            (root/'.github/runtime-platforms-023.json').write_text(json.dumps({'runtime':{'archive':'names.tgz'}}))
            (root/'apps/desktop/tauri').mkdir(parents=True)
            base=root/'base.tgz';base.write_bytes(b'pinned-original')
            output=root/'build'
            calls=[]
            def run(args,check):
                self.assertTrue(check)
                calls.append(args)
                if args[1].endswith('/build_runtime_023.py'):
                    d=Path(args[2]);d.mkdir();(d/'names.tgz').write_bytes(b'pinned-names')
                else:
                    d=Path(args[args.index('--output')+1]);d.mkdir()
                    archive=d/'runtime-bundle-0.2.23-win32-x64.tar.gz';archive.write_bytes(b'complete-platform-tree')
                    descriptor=d/'runtime-bundle.manifest.json';descriptor.write_bytes(b'{"schemaVersion":1}')
                    (d/'result.json').write_text(json.dumps({'archive':str(archive),'manifest':str(descriptor)}))
            with patch.object(stage,'ROOT',root),patch.object(stage,'BASE_SHA',stage.digest(base)),patch.object(stage.subprocess,'run',side_effect=run):
                result=stage.stage('win32-x64',root/'node.exe',root/'npm-cli.js',output,base)
            destination=Path(result['resourceDirectory'])
            self.assertEqual((destination/'maintained-runtime.tgz').read_bytes(),b'pinned-names')
            self.assertEqual((destination/'runtime-bundle-0.2.23-win32-x64.tar.gz').read_bytes(),b'complete-platform-tree')
            self.assertEqual((destination/'runtime-bundle.manifest.sha256').read_text().strip(),stage.digest(destination/'runtime-bundle.manifest.json'))
            config=json.loads(Path(result['tauriConfig']).read_text())
            self.assertEqual(config['bundle']['resources']['resources/runtime-bundle/'],'runtime-bundle/')
            self.assertEqual(config['bundle']['resources']['resources/LICENSE'],'resources/LICENSE')
            self.assertEqual(len(calls),2)
            self.assertFalse((root/'runtime').exists())

    def test_wrong_base_is_rejected_before_any_builder_or_resource_creation(self):
        with tempfile.TemporaryDirectory() as tmp:
            root=Path(tmp);base=root/'base.tgz';base.write_bytes(b'wrong')
            with patch.object(stage,'ROOT',root),patch.object(stage.subprocess,'run') as run:
                with self.assertRaisesRegex(ValueError,'base runtime digest differs'):
                    stage.stage('darwin-arm64',root/'node',root/'npm',root/'out',base)
                run.assert_not_called()
            self.assertFalse((root/'apps').exists())

if __name__=='__main__':unittest.main()
