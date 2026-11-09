import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
from unittest.mock import patch

SOURCE=Path(__file__).resolve().parent/'smoke/collect_installer_replay.py'
spec=importlib.util.spec_from_file_location('collect_replay',SOURCE)
collector=importlib.util.module_from_spec(spec);spec.loader.exec_module(collector)

class CollectorTests(unittest.TestCase):
    def test_keeps_nested_receipts_and_never_traverses_private_trees(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);e=root/'evidence';e.mkdir();(e/'startup').mkdir()
            (e/'startup/receipt.json').write_text('{"result":"pass"}')
            private=e/'startup/sandbox-removed-user';private.mkdir();(private/'secret.json').write_text('private')
            (root/'diagnostic.json').write_text('{"accepted":false}')
            original=Path.iterdir
            def checked(path):
                self.assertNotEqual(path,private,'private sandbox traversed')
                return original(path)
            with patch.object(Path,'iterdir',checked):rows=collector.collect(root)
            self.assertEqual({x['path'] for x in rows},{'evidence/startup/receipt.json','diagnostic.json'})
            manifest=json.loads((root/'artifact/evidence-manifest.json').read_text())
            self.assertFalse(manifest['accepted']);self.assertEqual(len(manifest['excluded']),1)
            self.assertFalse((root/'artifact/evidence/startup/sandbox-removed-user').exists())
            with self.assertRaises(FileExistsError):collector.collect(root)

    def test_evidence_symlink_is_refused(self):
        with tempfile.TemporaryDirectory() as temp:
            root=Path(temp);(root/'evidence').mkdir();outside=root/'outside.json';outside.write_text('keep')
            try:(root/'evidence/link.json').symlink_to(outside)
            except OSError:self.skipTest('OS denied symbolic link fixture')
            with self.assertRaisesRegex(ValueError,'symlink'):collector.collect(root)
            self.assertEqual(outside.read_text(),'keep')

if __name__=='__main__':unittest.main()
