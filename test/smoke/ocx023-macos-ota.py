#!/usr/bin/env python3
"""Explicit 022 -> 023 signed OTA, new-path/no-migration readback and manual rollback.

prepare never launches an app or queries the update channel. execute requires
the actual signed public beta assets and publication receipt before any work.
Uses the existing original-payload/native-input/TLS/automatic-PID machinery.
"""
import argparse
import hashlib
import importlib.util
import json
import os
from pathlib import Path
import re
import shutil
import signal
import subprocess
import time
import traceback

HERE = Path(__file__).resolve().parent
SPEC = importlib.util.spec_from_file_location('original_native_ota', HERE / 'ocx021-macos-ota.py')
base = importlib.util.module_from_spec(SPEC)
SPEC.loader.exec_module(base)
BASELINE = '1b5c8402dc340540711efa2c2c2037bce673a479'
BASE_ARCHIVE = 'e2d2f58f679757c9b8bc1b7c97531accbe1eccfd9afe5159ded042354887b590'
BASE_EXE = '027692f55a05d0563e88d1e7291280cb806f6556bb32680f6e7257bbcd24638c'


def validate_inputs(args):
    if not Path(args.out).is_absolute() or not Path(args.docs_root).is_absolute():
        raise ValueError('Explicit absolute evidence/docs roots are required')
    target = [args.source_sha, args.publication_receipt, args.target_archive, args.target_signature]
    if args.mode == 'execute':
        if not args.parent_beta_published or not re.fullmatch('[0-9a-f]{40}', args.source_sha or '') or not all(target):
            raise ValueError('Actual beta publication receipt, exact source and original signed target assets are required')
        for path in target[1:]:
            if not Path(path).is_absolute() or not Path(path).is_file():
                raise ValueError('Original target asset/receipt must be an existing absolute file')
    elif args.parent_beta_published or any(target):
        raise ValueError('prepare cannot accept publication/target inputs')


class Ota023(base.OtaRun):
    def __init__(self, args):
        super().__init__(args)
        # An independent install folder is the 023 program contract. The OLD
        # signed app starts here too; no relocation/compatibility code is added.
        (self.container / 'installed').mkdir(mode=0o700)
        self.app = self.container / 'installed/OpenCodeX Desktop/OpenCodeX Desktop.app'
        self.exe = self.app / 'Contents/MacOS/opencodex-desktop'
        self.rollback_process = None
        self.rollback_log = None
        self.report.update(app=str(self.app), scriptSha256=base.sha(Path(__file__)),
                           migration='none; old files preserved and new D/H initialized separately',
                           rollbackExecuted=False, fullRuntimeReadyAccepted=False)
        self.flush()

    def restarted_log(self):
        return self.root / 'data/desktop/logs/app.log'

    def restarted_business_readback(self):
        # Called only AFTER old-key signed production download/install and
        # exact same-path automatic new PID, source/whole bundle verification.
        desktop = self.root / 'data/desktop'
        home = self.root / 'data/opencodex/home'
        pointer = json.loads((desktop / 'manager-location.json').read_text())
        metadata = json.loads((desktop / 'manager-state/data-root.json').read_text())
        assert pointer == {'schemaVersion': 1, 'activeRoot': str(self.root / 'data')}
        assert str(metadata['structure_version']) == '2' and home.is_dir()
        self.press('模型'); self.press('渠道模型')
        screen = self.wait(lambda s: '暂无渠道' in self.text(s), 25, 'same-auto-process-new023-empty-channels')
        assert 'ota021-preserve' not in self.text(screen), 'Old channel was silently migrated'
        fresh_config = home / 'config.json'
        if fresh_config.exists():
            assert not json.loads(fresh_config.read_text()).get('providers'), 'Unexpected providers in new HOME'
        assert not (desktop / 'opencodex-home').exists()
        after = {rel: base.sha(self.root / rel) for rel in self.business}
        base.dump(self.out / 'after-ota-old-file-hashes.json', after)
        assert after == self.before, 'Old data/preferences/backup preimages changed during update'
        # The old handoff belongs to the old root. New software must not scan
        # or import it just to manufacture a new-path completion receipt.
        assert not (desktop / 'manager-state/update-handoff.json').exists()
        assert self.inspect(self.pid)['launchTime'] == self.restarted['launchTime']
        assert self.report['launchCalls'] == 1
        self.capture('same-auto-process-new023-empty-channels')
        self.check('same-automatic-process-new-paths-no-migration', oldPid=self.old.pid,
                   automaticRestartPid=self.pid, startupSource=self.args.source_sha,
                   sameAutomaticPidThroughoutReadback=True, manualRelaunches=0,
                   oldBusinessFilesUnchanged=len(after), desktop=str(desktop), home=str(home),
                   channelsEmpty=True, legacyHandoffNotImported=True, wholeSignedBundleEqual=True,
                   completeRuntimeReadiness='not claimed by sandbox OTA; normal bundle tests remain required')

    def rollback_original(self):
        # Manual rollback is a SEPARATE explicit test action after automatic
        # OTA readback. Never use this launch to replace a missing auto restart.
        assert self.report['automaticRestartPid'] == self.pid
        identity = self.inspect(self.pid)
        process = next(p for p in base.process_rows() if p['pid'] == self.pid)
        assert process['command'] == str(self.exe) and process['pgid'] == identity['pgid']
        pgid = process['pgid']
        assert pgid not in [0, 1, os.getpgrp(), 80916]
        members = [p for p in base.process_rows() if p['pgid'] == pgid]
        assert all(p['pid'] in self.owned or p['command'] == str(self.exe) for p in members)
        os.killpg(pgid, signal.SIGTERM)
        deadline = time.monotonic() + 10
        while time.monotonic() < deadline and any(p['pgid'] == pgid for p in base.process_rows()):
            time.sleep(.1)
        assert not any(p['pgid'] == pgid for p in base.process_rows()), 'Automatic new process did not exit for manual rollback'
        old_data = {rel: base.sha(self.root / rel) for rel in self.business}
        new_data = base.inventory(self.root / 'data')
        # Keep exact rollback inputs outside the disposable container even if
        # a native observer fails before the old process mounts its WebView.
        for rel in self.business:
            target = self.out / 'rollback-inputs' / rel
            target.parent.mkdir(parents=True, exist_ok=True)
            shutil.copyfile(self.root / rel, target)
        rollback = self.container / 'rollback'
        rollback.mkdir(mode=0o700)
        self.app = rollback / 'OpenCodeX Desktop.app'
        base.cmd(['/usr/bin/ditto', self.baseline, self.app]).check_returncode()
        assert base.inventory(self.app) == self.baseline_inventory
        self.exe = self.app / 'Contents/MacOS/opencodex-desktop'
        self.rollback_log = open(self.out / 'manual-rollback-process.log', 'wb')
        self.rollback_process = subprocess.Popen(
            ['/usr/bin/sandbox-exec', '-f', str(self.profile), str(self.exe), '-ApplePersistenceIgnoreState', 'YES'],
            env=self.env, cwd=self.root, stdin=subprocess.DEVNULL,
            stdout=self.rollback_log, stderr=subprocess.STDOUT, start_new_session=True)
        self.pid = self.rollback_process.pid
        self.owned[self.pid] = {'pid': self.pid, 'pgid': self.pid, 'command': str(self.exe)}
        self.report['rollbackExecuted'] = True
        self.report['rollbackLaunchCalls'] = 1
        registration_deadline = time.monotonic() + 10
        while True:
            if self.rollback_process.poll() is not None:
                raise base.Blocked('Original rollback process exited before native registration')
            if any(p['pid'] == self.pid for p in self.find_apps()):
                break
            if time.monotonic() >= registration_deadline:
                raise base.Blocked('Original rollback process was not registered in the native app list')
            time.sleep(.1)
        self.wait(lambda s: any(n.get('role') == 'AXWebArea' for n in s['nodes']), 35, 'manual-old-original-rollback-mounted')
        self.ax('focus'); self.press('模型'); self.press('渠道模型')
        readback = self.wait(lambda s: 'ota021-preserve' in self.text(s), 25, 'manual-old-original-rollback-channels')
        self.capture('manual-old-original-rollback-channels')
        assert 'ota021-preserve' in self.text(readback)
        assert {rel: base.sha(self.root / rel) for rel in self.business} == old_data == self.before
        assert base.inventory(self.root / 'data') == new_data, 'Rollback touched the new-path data'
        self.verify_bundle(self.app, '0.2.22', BASELINE, 'manual-rollback-original022')
        self.check('manual-rollback-original022-preserves-both-roots', oldBusinessFiles=len(old_data),
                   oldChannelReadThroughNativeUi=True, oldFileHashesUnchanged=True,
                   newDataTreeUnchanged=True, rollbackAppExactlyOriginal=True,
                   automaticDowngrade=False, migration=False, separateRollbackLaunch=True)

    def close(self):
        try:
            super().close()
        finally:
            if self.rollback_process:
                try: self.rollback_process.wait(timeout=5)
                finally:
                    if self.rollback_log: self.rollback_log.close()
            # Evidence lives OUTSIDE this unique task-owned container. Check
            # no owned PID remains before removing its copies/user roots.
            remaining = [p for p in base.process_rows() if p['pid'] in self.owned]
            if not remaining and self.container.parent == Path('/private/tmp') and self.container.name.startswith('ocx021-mac-ota-'):
                container = str(self.container)
                shutil.rmtree(self.container)
                base.dump(self.out / 'owned-container-cleanup.json', {'removed': container, 'remainingOwnedProcesses': []})
            else:
                base.dump(self.out / 'owned-container-cleanup.json', {'removed': False, 'remainingOwnedProcesses': remaining})


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--mode', choices=['prepare', 'execute'], default='prepare')
    parser.add_argument('--docs-root', required=True, help='Explicit documentation worktree, not a user data directory')
    parser.add_argument('--out', required=True, help='Absolute evidence parent; each run gets a fresh UUID')
    parser.add_argument('--baseline-archive', default='/tmp/ocx022-release-20261106/assets/OpenCodeX.Desktop_aarch64.app.tar.gz')
    parser.add_argument('--baseline-signature', default='/tmp/ocx022-release-20261106/assets/OpenCodeX.Desktop_aarch64.app.tar.gz.sig')
    parser.add_argument('--parent-beta-published', action='store_true')
    parser.add_argument('--source-sha')
    parser.add_argument('--publication-receipt')
    parser.add_argument('--target-archive')
    parser.add_argument('--target-signature')
    args = parser.parse_args()
    try: validate_inputs(args)
    except ValueError as error: parser.error(str(error))
    base.DOCS = Path(args.docs_root).resolve(strict=True)
    if not (base.DOCS / '.adg/project.json').is_file(): parser.error('Explicit docs worktree has no project binding')
    base.HISTORY = base.DOCS / '.adg/evidence/0.2.20/final-candidate-updates-20261104'
    base.FROM_VERSION, base.TO_VERSION = '0.2.22', '0.2.23'
    base.BASELINE_SOURCE, base.BASELINE_ARCHIVE_SHA, base.BASELINE_EXE_SHA = BASELINE, BASE_ARCHIVE, BASE_EXE
    base.TAG_PREFIX = 'https://github.com/gzers/opencodex-desktop/releases/download/v0.2.23/'
    run = Ota023(args)
    def interrupt(signum, frame): raise KeyboardInterrupt('Owned OTA interrupted by signal ' + str(signum))
    signal.signal(signal.SIGTERM, interrupt)
    try:
        run.prepare()
        if args.mode == 'execute':
            run.publication(); run.launch(); run.baseline_saves(); run.install(); run.restart_readback()
            run.rollback_original()
            run.report['completed'] = True
    except (Exception, KeyboardInterrupt) as error:
        run.report['failure'] = str(error)
        run.report['failureType'] = 'blocked' if isinstance(error, (base.Blocked, KeyboardInterrupt)) else 'fail'
        run.check('run-stopped', run.report['failureType'], reason=str(error))
        (run.out / 'failure.log').write_text(traceback.format_exc())
    finally: run.close()
    return 0 if run.report['result'] in ['pass', 'prepared-awaiting-parent-publication'] else (1 if run.report['result'] == 'fail' else 2)


if __name__ == '__main__': raise SystemExit(main())
