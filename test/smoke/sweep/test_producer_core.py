"""Focused diagnosis never promotes an untouched product gate or unknown exit."""
import contextlib
import io
import json
from pathlib import Path
import tarfile
import tempfile
import unittest
from unittest.mock import patch
import producer_core as p


class ProducerCoreTests(unittest.TestCase):
    def test_extraction_refuses_escape_links_collisions_and_special_files(self):
        for bad in ['../escape', '/absolute', 'C:escape', 'runtime\\escape', 'link', 'duplicate', 'fifo']:
            with self.subTest(bad=bad), tempfile.TemporaryDirectory() as directory:
                root=Path(directory);archive=root/'input.tgz'
                with tarfile.open(archive,'w:gz') as out:
                    if bad=='duplicate':
                        out.addfile(tarfile.TarInfo('same'));out.addfile(tarfile.TarInfo('SAME'))
                    else:
                        row=tarfile.TarInfo(bad)
                        if bad=='link':row.type=tarfile.SYMTYPE;row.linkname='../outside'
                        if bad=='fifo':row.type=tarfile.FIFOTYPE
                        out.addfile(row)
                with self.assertRaises(AssertionError):p.extract(archive,root/'out')

    def test_extracts_regular_exact_bytes_without_execution(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);archive=root/'input.tgz'
            with tarfile.open(archive,'w:gz') as out:
                row=tarfile.TarInfo('runtime/package.json');row.size=2;out.addfile(row,io.BytesIO(b'{}'))
            p.extract(archive,root/'out')
            self.assertEqual((root/'out/runtime/package.json').read_bytes(),b'{}')
            with self.assertRaises(AssertionError):p.extract(archive,root/'out')

    def test_unknown_cleanup_keeps_exact_fixture_and_original_failure(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'evidence').mkdir();(root/'sweep').mkdir();fixture=root/'work/fixture';fixture.mkdir(parents=True)
            (fixture/'owned.txt').write_text('retain')
            (root/'evidence/producer-report.json').write_text(json.dumps(dict(fixtureSafeToRemove=False)))
            with self.assertRaises(SystemExit):p.record(root,'18','owned-producer-cleanup',p.cleanup)
            receipt=root/'sweep/18-owned-producer-cleanup.json';before=receipt.read_bytes()
            self.assertEqual(json.loads(before)['status'],'fail');self.assertTrue(fixture.exists())
            with self.assertRaises(FileExistsError):p.record(root,'18','owned-producer-cleanup',p.cleanup)
            self.assertEqual(before,receipt.read_bytes())

    def test_all_missing_receipts_fail_and_technical_summary_never_accepts(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'sweep').mkdir()
            with contextlib.redirect_stdout(io.StringIO()):p.finish(root)
            s=p.core.read(root/'sweep/summary.json')
            self.assertEqual((s['passed'],s['failed']),(0,4))
            self.assertFalse(s['accepted']);self.assertFalse(s['dualSidProductAcceptance'])
            self.assertEqual(s['hypothesis']['verdict'],'inconclusive')
            with self.assertRaises(FileExistsError), contextlib.redirect_stdout(io.StringIO()):p.finish(root)

    def test_core_focus_references_untouched_gates_but_never_invents_their_receipts(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);(root/'sweep').mkdir()
            for step,name in p.PLAN:
                p.core.write(root/f'sweep/{step}-{name}.json',dict(step=step,name=name,status='pass',firstError='',accepted=False,diagnosticOnly=True))
            p.core.write(root/'sweep/hypothesis.json',dict(id='H',claim='bounded claim',refutation='refute on failure',startedAt=0,predictions=[dict(step='13',status='pass')],unchangedPassingEvidence=[dict(gate='dual SID',evidence='prior original artifact')]))
            with contextlib.redirect_stdout(io.StringIO()):p.finish(root)
            s=p.core.read(root/'sweep/summary.json');self.assertEqual(s['passed'],4);self.assertFalse(s['accepted'])
            self.assertFalse(s['dualSidProductAcceptance']);self.assertFalse(any(row['step']=='11' for row in s['steps']))
            self.assertEqual(s['hypothesis']['verdict'],'not-refuted')

if __name__=='__main__':unittest.main(verbosity=2)
