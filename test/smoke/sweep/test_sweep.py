"""Behavior tests for continuation, honest state and retained failure evidence."""
import importlib.util
import json
from pathlib import Path
import tempfile
import unittest
HERE=Path(__file__).resolve().parent

def module(name):
    spec=importlib.util.spec_from_file_location(name,HERE/(name+'.py'))
    value=importlib.util.module_from_spec(spec);spec.loader.exec_module(value);return value
summary,collector=module('summary'),module('collect')
PLAN=json.loads((HERE/'plan.json').read_text())['steps']

class SweepTests(unittest.TestCase):
    def test_missing_execution_fails_all_green_still_cannot_accept(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory);plan=[('01','early'),('02','later')]
            (root/'02-later.json').write_text(json.dumps(dict(step='02',name='later',status='pass',accepted=False,diagnosticOnly=True)))
            result=summary.summarize(root,plan)
            self.assertEqual((result['failed'],result['passed']),(1,1));self.assertFalse(result['accepted'])
            before=(root/'01-early.json').read_bytes();summary.summarize(root,plan)
            self.assertEqual(before,(root/'01-early.json').read_bytes())
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            for row in PLAN:
                (root/f"{row['step']}-{row['name']}.json").write_text(json.dumps({**row,'status':'pass','accepted':False,'diagnosticOnly':True}))
            result=summary.summarize(root,[(r['step'],r['name'])for r in PLAN])
            self.assertEqual(result['failed'],0);self.assertFalse(result['accepted'])

    def test_accepted_or_not_ready_never_turns_green(self):
        for change in [dict(accepted=True),dict(status='not-ready'),dict(name='other'),dict(diagnosticOnly=False)]:
            with tempfile.TemporaryDirectory() as directory:
                root=Path(directory)
                (root/'01-probe.json').write_text(json.dumps(dict(step='01',name='probe',status='pass',accepted=False,diagnosticOnly=True)|change))
                with self.assertRaises(ValueError):summary.summarize(root,[('01','probe')])

    def test_supply_failure_does_not_suppress_paths_ipc_or_exact_cleanup(self):
        with tempfile.TemporaryDirectory() as directory:
            root=Path(directory)
            def put(path,data):
                p=root/path;p.parent.mkdir(parents=True,exist_ok=True);p.write_text(json.dumps(data))
            for i in (1,2):
                for stage in ['private-bundle-supply','runtime-readiness','path-snapshot','same-user-ipc','peer-business-data','cross-user-ipc','cross-user-dpapi','same-sid-single-instance']:
                    failed=stage in ['private-bundle-supply','runtime-readiness']
                    put(f'evidence/two-user/user{i}/sweep-{stage}.json',dict(status='fail'if failed else'pass',firstError='actual source unresolved'if failed else'',accepted=False,diagnosticOnly=True))
            put('evidence/two-user/two-user-native.json',dict(runtimeProxyPortIsolation='not-verified',desktopAclRestored=True,evidenceAclRestored=True,sealedCandidateUnchanged=True,cleanup=[dict(result='account-and-profile-removed',sid='SID1'),dict(result='account-and-profile-removed',sid='SID2')]))
            for kind in ['msi','nsis']:
                for mode in ['default','custom']:
                    put(f'evidence/{kind}-{mode}/installation-owner-cleanup.json',dict(removed=True))
                    put(f'evidence/{kind}-{mode}/installer-verification.json',dict(result='pass',defaultUninstallChecks=[dict(result='pass',exitCode=0,executableRemoved=True,userDataRetained=True)],installers=[dict(uninstallExitCode=0,uninstallRemovedExecutable=True,defaultUninstallRetainedData=True)]))
            collector.collect(root)
            rows={p.stem:json.loads(p.read_text())for p in(root/'sweep').glob('*.json')}
            for name in ['12-private-bundle-supply','13-runtime-readiness','14-runtime-ports']:self.assertEqual(rows[name]['status'],'fail')
            for name in ['15-two-user-paths','16-same-user-ipc','17-cross-user-ipc','18-account-and-process-cleanup','19-uninstall-and-retention']:self.assertEqual(rows[name]['status'],'pass',name)
            self.assertTrue(all(r['accepted']is False for r in rows.values()))
            before=(root/'sweep/12-private-bundle-supply.json').read_bytes()
            with self.assertRaises(FileExistsError):collector.collect(root)
            self.assertEqual(before,(root/'sweep/12-private-bundle-supply.json').read_bytes())

    def test_every_gate_always_continues_and_complete_artifact(self):
        workflow=(HERE.parents[2]/'.github/workflows/windows023-sweep.yml').read_text()
        for block in workflow.split('      - name: ')[1:]:
            name=block.splitlines()[0]
            if name[:2].isdigit():
                self.assertIn('if: always()',block,name);self.assertIn('continue-on-error: true',block,name)
        self.assertIn('artifact.py',workflow);self.assertIn('/artifact/**',workflow);self.assertIn('summary.py',workflow)
        self.assertIn('github.run_attempt',workflow);self.assertNotIn('Upload verified Windows installers',workflow)

if __name__=='__main__':unittest.main(verbosity=2)
