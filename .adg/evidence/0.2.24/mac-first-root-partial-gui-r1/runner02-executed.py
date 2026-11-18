"""PID-scoped packaged GUI first-root cancellation and actual successor lifecycle.

Only synthetic channels/keys; explicit local official Start/Stop. No model API, WebDAV or daily HOME.
Keeps commands/AX/config evidence and removes only its own app/profile/processes.
"""
import argparse
import importlib.util
import json
import os
from pathlib import Path
import plistlib
import pwd
import shutil
import signal
import sqlite3
import hashlib
import base64
import subprocess
import tempfile
import time

ROOT = Path(__file__).resolve().parents[2]
_spec = importlib.util.spec_from_file_location('first_supply024', Path(__file__).with_name('macos024-first-supply.py'))
base = importlib.util.module_from_spec(_spec); _spec.loader.exec_module(base)


def write(path, value):
    path.write_text(json.dumps(value, ensure_ascii=False, indent=2) + chr(10))


def run(app, expected_sha, output, seed_host, flow="root-cancel"):
    if flow not in ('root-cancel','root-partial-cancel','stop-progress'):raise ValueError('unknown explicit GUI flow')
    app = app.resolve(strict=True)
    if app.name != base.APP or base.sha(app / 'Contents/MacOS/opencodex-desktop') != expected_sha:
        raise ValueError('explicit exact sealed app required')
    output = output.absolute(); output.mkdir(parents=True, mode=0o700, exist_ok=False)
    root = Path(tempfile.mkdtemp(prefix='ocxd024-gui-root-cancel-', dir='/private/tmp')); root.chmod(0o700)
    profile = root / 'profile'; profile.mkdir(mode=0o700)
    temporary = root / 'tmp'; temporary.mkdir(mode=0o700)
    copied = root / base.APP
    helper, capture = root / 'ax', root / 'capture'
    commands, snapshots, processes = [], [], []
    proc, log = None, None
    service_requested = False
    coordinator_paths = []
    coordinator_before = {}
    service_verified_stopped = False
    report = {'inputApp': str(app), 'inputSha256': expected_sha, 'releaseAcceptance': False,
              'dailyConfigurationModified': False, 'serviceActionRequested': False, 'result': 'incomplete',
              'scope': 'packaged GUI explicit first-root cancellation and official successor Start/Stop; no model requests', 'flow':flow}

    def command(argv, timeout=30, env=None, required=True):
        p = subprocess.run([str(x) for x in argv], cwd=root, env=env, capture_output=True, timeout=timeout)
        commands.append({'argv': [str(x) for x in argv], 'exitCode': p.returncode,
                         'stdout': p.stdout.decode(errors='replace')[:18000], 'stderr': p.stderr.decode(errors='replace')[:18000]})
        if p.returncode and required: raise RuntimeError('native command failed: ' + str(argv[0]) + ' / ' + p.stderr.decode(errors='replace')[:400])
        return p

    def snapshot(label):
        if proc.poll() is not None: raise RuntimeError('candidate exited before observation')
        p = command([helper, 'snapshot', proc.pid, exe], 12, required=False)
        if p.returncode == 3 and proc.poll() is None:
            # macOS may not have registered the just-created GUI process yet.
            # Reobserve THIS live handle; never start a replacement process.
            return None
        if p.returncode: raise RuntimeError('candidate AX observation failed')
        value = json.loads(p.stdout)
        if value.get('locked'): raise RuntimeError('desktop locked')
        snapshots.append(label); write(output / (str(len(snapshots)) + '-' + label + '.json'), value)
        return value

    def wait_for(label, predicate, seconds=30):
        end = time.monotonic() + seconds
        while time.monotonic() < end:
            value = snapshot(label)
            if value is not None and predicate(value): return value
            time.sleep(.6)
        raise RuntimeError('observation timeout: ' + label)

    def matches(value, label, role=None):
        return [n for n in value['nodes'] if (role is None or n.get('role') == role)
                and label in (n.get('title'), n.get('description')) and n.get('enabled', True)]

    def press(label):
        value = wait_for('find-' + label, lambda s: bool(matches(s, label, 'AXButton')))
        found = matches(value, label, 'AXButton')
        if label == '关闭' and len(found) == 2:
            # The same dialog has a header icon and an explicit footer close.
            # Both are PID/executable-scoped WebView buttons; choose the footer.
            found = [max(found,key=lambda n:n.get('frame',{}).get('y',-1))]
        if len(found) != 1: raise RuntimeError('ambiguous candidate button: ' + label)
        clicked = command([helper, 'press', proc.pid, exe, found[0]['path'], label], required=False)
        if clicked.returncode == 7 and b'AX tree changed; expected control label no longer matches' in clicked.stdout:
            # The native helper refused BEFORE AXPress. Reobserve this same
            # live PID/executable once; do not replay an admitted user action.
            fresh = wait_for('relocate-' + label, lambda v: bool(matches(v,label,'AXButton')))
            found = matches(fresh,label,'AXButton')
            if label == '关闭' and len(found) == 2: found = [max(found,key=lambda n:n.get('frame',{}).get('y',-1))]
            if len(found) != 1: raise RuntimeError('ambiguous refreshed candidate button: ' + label)
            command([helper,'press',proc.pid,exe,found[0]['path'],label])
        elif clicked.returncode:
            raise RuntimeError('native press failed without a safe pre-action retry: ' + label)


    def end_process():
        nonlocal proc, log
        if proc and proc.poll() is None:
            os.killpg(proc.pid, signal.SIGTERM)
            proc.wait(timeout=10)
        if proc:
            residues = base.owned_group(proc.pid)
            processes.append({'pid': proc.pid, 'exitCode': proc.returncode, 'residues': residues})
            if residues: raise RuntimeError('own process group residues remain')
        if log: log.close()
        proc, log = None, None

    try:
        command(['/usr/bin/ditto', '--rsrc', '--extattr', '--acl', app, copied], 120)
        exe = copied / 'Contents/MacOS/opencodex-desktop'
        command(['/usr/bin/codesign', '--verify', '--deep', '--strict', copied])
        for source, dest in [(ROOT / 'test/smoke/ocx021-macos-native.swift', helper), (ROOT / 'test/smoke/lib/capwin.swift', capture)]:
            command(['/usr/bin/xcrun', 'swiftc', source, '-o', dest], 90)
        status = json.loads(command([helper, 'status']).stdout); report['permissions'] = status
        if status.get('locked') or not status.get('axTrusted'): raise RuntimeError('unlocked AX desktop required')
        _, config = base.seed_preferences(profile)
        initial={'hostname':'127.0.0.1','port':0,'defaultProvider':'gui-fixture','defaultModelAliases':False,'providers':{'gui-fixture':{'adapter':'openai-chat','authMode':'key','apiKey':'synthetic-ui-unused','baseUrl':'http://127.0.0.1:1/v1','allowPrivateNetwork':True,'models':['gui-fixture-model'],'selectedModels':['gui-fixture-model'],'liveModels':False,'defaultAliases':False}}}
        write(config,initial);config.chmod(0o600)
        client=profile/'.codex';namespace=Path('/private/tmp')/('opencodex-runtime-v1-'+str(os.getuid()))
        digest=hashlib.sha256(str(client).encode()).hexdigest()
        state_db=str(client/'state_5.sqlite');h=str(client);history=hashlib.sha256((str(len(h))+':'+h+str(len(state_db))+':'+state_db).encode()).hexdigest()
        for name,key in [('native-write-locks',digest),('catalog-write-locks',digest),('history-write-locks',history)]:
            path=namespace/name/(key+'.sqlite')
            for suffix in ['', '-journal','-wal','-shm']:
                item=Path(str(path)+suffix);coordinator_paths.append(item);coordinator_before[str(item)]=os.path.lexists(item)
        if any(coordinator_before.values()):raise RuntimeError('fresh task coordinator identity already exists')
        write(output/'coordinators-before.json',coordinator_before)
        policy = root / 'candidate.sb'
        policy.write_text('(version 1) (allow default) (deny network*) (allow network-bind (local ip "localhost:*")) (allow network-inbound (local ip "localhost:*")) (allow network-outbound (remote ip "localhost:*")) (deny file-read* file-write* (subpath ' + json.dumps(pwd.getpwuid(os.getuid()).pw_dir) + ')) (deny file-read* file-write* (subpath "/Library/Keychains")) (deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))')
        shutil.copyfile(policy, output / 'candidate.sb')
        env = {'HOME': str(profile), 'CFFIXED_USER_HOME': str(profile), 'CODEX_HOME': str(profile / '.codex'),
               'PATH': '/usr/bin:/bin:/usr/sbin:/sbin', 'TMPDIR': str(temporary) + '/', 'LANG': 'en_US.UTF-8',
               'USER': pwd.getpwuid(os.getuid()).pw_name, 'LOGNAME': pwd.getpwuid(os.getuid()).pw_name}
        denied = subprocess.run(['/usr/bin/sandbox-exec', '-f', str(policy), '/usr/bin/nc', '-v', '-G', '1', '-z', '192.0.2.1', '443'], env=env, capture_output=True)
        report['networkGuard'] = {'exitCode': denied.returncode, 'stderr': denied.stderr.decode(errors='replace')}
        if denied.returncode == 0 or b'Operation not permitted' not in denied.stderr: raise RuntimeError('network denial unproven')
        def launch(generation):
            nonlocal proc,log
            log=(output/('application-'+str(generation)+'.log')).open('xb')
            proc=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(policy),str(exe)],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
            print(json.dumps({'generation':generation,'pid':proc.pid,'phase':'started'}),flush=True)
            current=profile/'Library/Application Support/OpenCodeX Desktop/runtime/current.json'
            wait_for('startup-'+str(generation),lambda v:current.is_file() and any('运行状态与常用操作' in str(n.get('value','')) or n.get('title')=='版本升级' for n in v['nodes']),180)
        def head():
            path=profile/'.codex-desktop/desktop/manager-state/client-session-index.sqlite'
            if not path.exists():return None
            c=sqlite3.connect('file:'+str(path)+'?mode=ro',uri=True,timeout=0)
            try:
                row=c.execute('select document from current_session where singleton=1').fetchone()
                return json.loads(row[0]) if row else None
            except sqlite3.OperationalError as e:
                if 'locked' in str(e):return None
                raise
            finally:c.close()
        launch(1)
        closed=None
        if flow in ('root-cancel','root-partial-cancel'):
            end_process()
            seed=root/'seed-host';shutil.copyfile(seed_host,seed);seed.chmod(0o700)
            command(['/usr/bin/codesign','--verify','--strict',seed])
            seed_env=dict(env,OCXD024_GUI_ROOT_CANCEL_SEED=str(root))
            if flow=='root-partial-cancel':seed_env['OCXD024_GUI_ROOT_PARTIAL_SEED']='1'
            command(['/usr/bin/sandbox-exec','-f',str(policy),str(seed),'--exact','prepare_exact_owned_first_root_for_real_gui','--ignored','--nocapture'],60,env=seed_env)
            shutil.copyfile(root/'seed-result.json',output/'seed-result.json');before_cancel=head();write(output/'head-before-cancel.json',before_cancel)
            partial_before=None
            if flow=='root-partial-cancel':
                fragment=profile/'.codex-desktop/desktop/manager-state/client-originals'/before_cancel['ticket']/'synthetic-unpublished-fragment.json'
                def partial_image():
                    info=fragment.stat();raw=fragment.read_bytes()
                    return {'device':info.st_dev,'inode':info.st_ino,'mode':info.st_mode & 0o777,'uid':info.st_uid,'gid':info.st_gid,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'rawBase64':base64.b64encode(raw).decode()}
                partial_before=partial_image();write(output/'partial-fragment-before.json',partial_before)
            launch(2);press('诊断')
            press('检查客户端恢复');press('取消启动准备')
            confirmation_text='已经捕获的文件会完整保留' if flow=='root-partial-cancel' else '保管这次启动新建的空目录'
            value=wait_for('first-root-confirmation',lambda v:any(confirmation_text in str(n.get('value','')) for n in v['nodes']))
            if status.get('screenCaptureAllowed'):
                windows=[w for w in value.get('windows',[]) if w.get('onscreen') and w.get('layer')==0]
                if windows:command([capture,windows[0]['id'],output/'cancel-confirmation.png'])
            cancel_attempts=[]
            for attempt in range(1,5):
                clicked=time.monotonic();press('确认取消启动准备')
                end=time.monotonic()+30;observed=None
                while time.monotonic()<end:
                    observed=snapshot('cancelled')
                    if observed is not None:
                        text='\n'.join(str(n.get('value')or n.get('title')or n.get('description')or'') for n in observed['nodes'])
                        if '原目录缺席已恢复' in text:break
                        if '运行状态正在核对，请稍后刷新' in text:break
                    time.sleep(.6)
                else:raise RuntimeError('cancellation observation timed out')
                if '原目录缺席已恢复' in text:
                    cancel_attempts.append({'attempt':attempt,'outcome':'completed'});break
                current_head=head()
                if current_head!=before_cancel:
                    raise RuntimeError('busy retry refused: original preparation changed')
                cancel_attempts.append({'attempt':attempt,'outcome':'preflight-busy-no-mutation','head':current_head})
                write(output/'explicit-cancel-attempts.json',cancel_attempts)
                if attempt==4:raise RuntimeError('status preflight busy after bounded explicit rechecks')
                time.sleep(.7);press('重新检查');press('取消启动准备')
            write(output/'explicit-cancel-attempts.json',cancel_attempts)
            closed=head();assert closed['state']=='root_cancellation_absent' and closed['completionSha256'] and not client.exists()
            write(output/'head-after-cancel.json',closed);report['guiCancellationObservedMs']=round((time.monotonic()-clicked)*1000)
            report['actualGuiCancellationPassed']=True
            if partial_before is not None:
                assert partial_image()==partial_before;write(output/'partial-fragment-after-cancel.json',partial_image());report['partialPreservedAfterGuiCancellation']=True
            # Close just the app dialog using its explicit WebView button.
            press('关闭');press('概览')
        if flow=='stop-progress':
            # Production startup supply and its first status are asynchronous;
            # observing readiness here precedes any lifecycle action/budget.
            wait_for('pre-action-ui-ready',lambda v:bool(matches(v,'启动 OpenCodex','AXButton')),120)
        if flow in ('root-cancel','root-partial-cancel'):
            # Reopening explicit inspection may reconcile an old UI latch using
            # final indexed previews + live status, never a repeated mutation.
            for recheck in range(4):
                value=snapshot('post-cancel-start-gate')
                if value is not None and matches(value,'启动 OpenCodex','AXButton'):break
                press('诊断');press('检查客户端恢复')
                end=time.monotonic()+12
                while time.monotonic()<end:
                    value=snapshot('closed-root-readonly-reconcile')
                    if value is not None:
                        content='\n'.join(str(n.get('value')or n.get('title')or'') for n in value['nodes'])
                        if '现在可以启动' in content:break
                        if matches(value,'刷新运行状态','AXButton'):
                            press('刷新运行状态')
                    time.sleep(.6)
                if head()!=closed:raise RuntimeError('readonly root reconciliation changed canonical state')
                report['explicitReadonlyPostCancelRechecks']=recheck+1
                press('关闭');press('概览')
            else:raise RuntimeError('closed-root current status not confirmed; no Start replayed')
        service_requested=True;report['serviceActionRequested']=True;press('启动 OpenCodex')
        home=profile/'.codex-desktop/opencodex/home';record=home/'runtime-port.json'
        wait_for('successor-service-ready',lambda v:record.exists() and head() is not None and head()['state']=='ready',60)
        running=head();r=json.loads(record.read_bytes());report['service']={'pid':r['pid'],'port':r['port']}
        assert running['references'] and (closed is None or running['ticket']!=closed['ticket']);write(output/'successor-ready-head.json',running)
        # Progress overlay may dismiss itself; close it only if visible.
        value=snapshot('before-stop')
        if value is not None and matches(value,'关闭','AXButton'):press('关闭')
        press('停止')
        wait_for('successor-service-stopped',lambda v:not record.exists() and head() is not None and head()['state']=='root_absent',45)
        stopped=head();write(output/'successor-stopped-head.json',stopped)
        assert stopped['completionSha256'] and not client.exists();service_verified_stopped=True
        def text_of(value):return '\n'.join(str(n.get('value')or n.get('title')or n.get('description')or'') for n in value['nodes'])
        completed=wait_for('stop-card-completed',lambda v:'停止已完成' in text_of(v),20)
        report['stopCompletedCardObserved']=True
        # Countdown intentionally pauses for hover/focus. Move both to the
        # existing Overview navigation, with native target-PID hit testing.
        command([helper,'focus',proc.pid,exe]);press('概览')
        value=wait_for('countdown-clear-focus',lambda v:bool(matches(v,'概览','AXButton')))
        nav=matches(value,'概览','AXButton')
        if len(nav)!=1:raise RuntimeError('ambiguous existing overview navigation')
        command([helper,'click',proc.pid,exe,nav[0]['path'],'概览'])
        snapshot('native-click-outside-completed-card')
        dismissed=wait_for('stop-card-dismissed',lambda v:'停止已完成' not in text_of(v) and '正在停止 OpenCodex' not in text_of(v) and bool(matches(v,'启动 OpenCodex','AXButton')),15)
        report['stopCardAutomaticallyDismissed']=True
        if status.get('screenCaptureAllowed'):
            windows=[w for w in dismissed.get('windows',[]) if w.get('onscreen') and w.get('layer')==0]
            if windows:command([capture,windows[0]['id'],output/'stop-card-dismissed.png'])
        if flow=='root-partial-cancel':
            assert partial_image()==partial_before;write(output/'partial-fragment-after-official-stop.json',partial_image());report['partialPreservedAfterSuccessorNormalStop']=True
        report.update(result='passed',actualGuiSuccessorStartStop=flow in ('root-cancel','root-partial-cancel'),actualGuiCancellationPassed=flow in ('root-cancel','root-partial-cancel'),normalStopAndOriginalAbsence=True)
        end_process()
    except BaseException as exc:
        report.update(result='failed', error=repr(exc))
        raise
    finally:
        try:
            if service_requested and not service_verified_stopped:
                report['liveServiceSceneRetained']=True
                if proc and proc.poll() is None:
                    try:
                        value=snapshot('failure-stop-check')
                        if value is not None and matches(value,'关闭','AXButton'):press('关闭')
                        press('概览');press('停止')
                        wait_for('failure-normal-stop',lambda v:not (profile/'.codex-desktop/opencodex/home/runtime-port.json').exists(),35)
                        service_verified_stopped=True
                    except Exception as e:report['normalStopCleanupError']=repr(e)
            if not service_requested or service_verified_stopped:end_process()
        except Exception as exc: report['cleanupError'] = repr(exc)
        config_path = profile / '.codex-desktop/opencodex/home/config.json'
        if config_path.is_file(): shutil.copyfile(config_path, output / 'final-synthetic-config.json')
        for name in ('app.log',):
            file = profile / '.codex-desktop/desktop/logs' / name
            if file.is_file(): shutil.copyfile(file, output / ('startup-' + name))
        report['processes'] = processes
        report['sourceAppUnchanged'] = base.sha(app / 'Contents/MacOS/opencodex-desktop') == expected_sha
        report['copiedAppUnchanged'] = (copied / 'Contents/MacOS/opencodex-desktop').is_file() and base.sha(copied / 'Contents/MacOS/opencodex-desktop') == expected_sha
        manager=profile/'.codex-desktop/desktop/manager-state'
        if manager.exists():
            for file in manager.rglob('*'):
                if file.is_file() and not file.is_symlink() and file.stat().st_size<16*1024*1024:
                    target=output/'raw-manager'/file.relative_to(manager);target.parent.mkdir(parents=True,exist_ok=True);shutil.copyfile(file,target)
        try:
            report['coordinatorCleanup']=[]
            if not report.get('cleanupError') and (not service_requested or service_verified_stopped) and all(not p['residues'] for p in processes):
                for path in coordinator_paths:
                    if not os.path.lexists(path):continue
                    if path.suffix!='.sqlite':raise RuntimeError('coordinator sidecar retained; cleanup refused')
                    before=path.lstat()
                    if path.is_symlink() or before.st_uid!=os.getuid() or before.st_nlink!=1 or before.st_mode&0o777!=0o600:raise RuntimeError('coordinator identity/protection refused')
                    c=sqlite3.connect(path.as_uri()+'?mode=rw',uri=True,timeout=0)
                    try:c.execute('begin exclusive');c.execute('rollback')
                    finally:c.close()
                    raw=path.read_bytes();after=path.lstat()
                    if (before.st_dev,before.st_ino)!=(after.st_dev,after.st_ino):raise RuntimeError('coordinator replaced')
                    row={'path':str(path),'device':after.st_dev,'inode':after.st_ino,'bytes':len(raw),'sha256':hashlib.sha256(raw).hexdigest(),'rawBase64':base64.b64encode(raw).decode()}
                    report['coordinatorCleanup'].append(row);write(output/'coordinator-raw.json',report['coordinatorCleanup']);path.unlink()
                if any(os.path.lexists(p) for p in coordinator_paths):raise RuntimeError('owned coordinator residue')
                shutil.rmtree(root)
        except Exception as exc:report['cleanupError']=repr(exc)
        report['temporaryRoot'] = str(root); report['temporaryRootAbsent'] = not root.exists()
        write(output / 'commands.json', commands); write(output / 'summary.json', report)
    return report


if __name__ == '__main__':
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--app', type=Path, required=True)
    parser.add_argument('--expected-sha256', required=True)
    parser.add_argument('--output', type=Path, required=True)
    parser.add_argument('--seed-host', type=Path, required=True)
    parser.add_argument('--flow',choices=['root-cancel','root-partial-cancel','stop-progress'],default='root-cancel')
    args = parser.parse_args()
    result = run(args.app, args.expected_sha256, args.output, args.seed_host, args.flow)
    print(json.dumps({k:v for k,v in result.items() if k != 'coordinatorCleanup'}, ensure_ascii=False), flush=True)
