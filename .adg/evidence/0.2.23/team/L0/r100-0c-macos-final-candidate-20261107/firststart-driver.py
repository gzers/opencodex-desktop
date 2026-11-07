import json,os,subprocess,tempfile,time,plistlib,hashlib,pwd,signal,shutil
from pathlib import Path
repo=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
install=Path(tempfile.mkdtemp(prefix='ocxd023-0c-release-installed-',dir='/private/tmp')).resolve()
app=install/'OpenCodeX Desktop.app'
subprocess.run(['/usr/bin/ditto',str(repo/'apps/desktop/tauri/target/release/bundle/macos/OpenCodeX Desktop.app'),str(app)],check=True)
info=plistlib.loads((app/'Contents/Info.plist').read_bytes());assert info['CFBundleShortVersionString']=='0.2.23'
exe=app/'Contents/MacOS'/info['CFBundleExecutable']
root=Path(tempfile.mkdtemp(prefix='ocxd023-native-startup-',dir='/private/tmp')).resolve()
out=Path('/tmp/ocxd023-0c-unlocked-firststart-evidence');out.mkdir(exist_ok=True)
root.joinpath('home').mkdir();root.joinpath('tmp').mkdir()
report={'schemaVersion':1,'kind':'isolated release-bundle ordinary first start; provisioning plus visible environment screen','app':str(app),'root':str(root),'version':info['CFBundleShortVersionString'],'checks':[]}
proc=None
try:
    helper=root/'ax';cap=root/'capwin'
    for src,dst in [(repo/'test/smoke/ocx021-macos-native.swift',helper),(repo/'test/smoke/lib/capwin.swift',cap)]:
        p=subprocess.run(['/usr/bin/xcrun','swiftc',str(src),'-o',str(dst)],capture_output=True,text=True,timeout=60)
        (out/(dst.name+'-compile.log')).write_text(p.stdout+p.stderr);assert p.returncode==0
    def run(args):
        p=subprocess.run([str(x) for x in args],capture_output=True,text=True,timeout=30)
        if p.returncode:raise RuntimeError(p.stdout+p.stderr)
        return json.loads(p.stdout)
    report['permissions']=run([helper,'status'])
    report['visualChecks']='exact owned app focused; screenshot requires normal overview review' if not report['permissions']['locked'] else 'deferred: screen locked; bootstrap validation only'
    home=pwd.getpwuid(os.getuid()).pw_dir
    sb=root/'guard.sb';sb.write_text('(version 1)\n(allow default)\n(deny network*)\n(deny file-read* (require-all (subpath '+json.dumps(home)+') (require-not (subpath '+json.dumps(str(app))+'))))\n(deny file-write* (subpath '+json.dumps(home)+'))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n')
    env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','LANG':'en_US.UTF-8','USER':pwd.getpwuid(os.getuid()).pw_name,'HOME':str(root/'home'),'CFFIXED_USER_HOME':str(root/'home'),'TMPDIR':str(root/'tmp')+'/','TMP':str(root/'tmp'),'TEMP':str(root/'tmp'),'CODEX_HOME':str(root/'home/.codex')}
    deny=subprocess.run(['/usr/bin/sandbox-exec','-f',str(sb),'/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443'],env=env,capture_output=True,text=True,timeout=10)
    (out/'network-denial.log').write_text(deny.stdout+deny.stderr);assert deny.returncode!=0 and 'Operation not permitted' in deny.stderr
    with (out/'stdout.log').open('wb') as log:
        proc=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(sb),str(exe)],cwd=root,env=env,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
    report['pid']=proc.pid
    deadline=time.monotonic()+45
    while time.monotonic()<deadline:
        if proc.poll() is not None:raise RuntimeError('native app exited '+str(proc.returncode))
        if (root/'home/.codex-desktop/desktop/manager-state/data-root.json').exists():break
        time.sleep(.5)
    run([helper,'focus',proc.pid,exe])
    report['checks'].append('actual native process stayed alive and exact owned application explicitly focused')
    data=root/'home/.codex-desktop/desktop';up=root/'home/.codex-desktop/opencodex/home'
    pointer=json.loads((data/'manager-location.json').read_text())
    metadata=json.loads((data/'manager-state/data-root.json').read_text())
    assert pointer['activeRoot']==str(root/'home/.codex-desktop')
    assert str(metadata['structure_version'])=='2'
    assert up.is_dir()
    report['pointer']=pointer;report['metadata']=metadata
    report['checks'].append('actual isolated HOME U/desktop and U/opencodex/home split with fixed v2 pointer')
    for p in data.rglob('app.log'):
        txt=p.read_text();(out/'app.log').write_text(txt)
        report['startupEvent']=next((line for line in txt.splitlines() if 'startup version=' in line),'')
        assert 'startup version=0.2.23 commit=0c869d9497c102b714dea1ee89394f999da8c4ec' in report['startupEvent'],'wrong final candidate source stamp'
    until=time.monotonic()+240
    marker=app.parent/'runtime/opencodex/.opencodex-desktop-runtime.json'
    while time.monotonic()<until:
        assert proc.poll() is None,'native ordinary candidate exited'
        sourcePath=data/'manager-state/runtime.json'
        try: observed=json.loads(sourcePath.read_text())
        except (FileNotFoundError,json.JSONDecodeError): observed={}
        if observed.get('source')=='managed' and observed.get('resolved_version')=='2.97.0-desktop023.names.1':
            messages='\n'.join(p.read_text(errors='replace') for p in (data/'logs').glob('app.log'))
            if 'integrated runtime provisioned and verified' in messages:break
            if 'integrated runtime provisioning failed' in messages:raise AssertionError('actual worker failed after source publication')
        time.sleep(.5)
    package=app.parent/'runtime/opencodex/node_modules/@bitkyc08/opencodex/package.json'
    assert package.is_file(),'offline bundled package not provisioned'
    pkg=json.loads(package.read_text());assert pkg['version']=='2.97.0-desktop023.names.1'
    source=json.loads((data/'manager-state/runtime.json').read_text())
    assert source['source']=='managed' and source['resolved_version']==pkg['version'],'managed source receipt incomplete'
    assert source['path']==str(app.parent/'runtime/bin/ocx')
    report['provisionedRuntimeVersion']=pkg['version'];report['runtimeSource']=source
    report['checks'].append('actual normal startup provisions pinned offline runtime from copied release resources')
    assert (app.parent/'runtime/node/bin/node').is_file() and (app.parent/'runtime/bun/bun').is_file()
    assert not (root/'home/.opencodex').exists()
    assert not (root/'home/.codex-desktop/desktop/opencodex-home').exists()
    assert 'integrated runtime provisioned and verified' in messages,'actual startup worker terminal receipt absent'
    time.sleep(40)
    snap=run([helper,'snapshot',proc.pid,exe]);(out/'ordinary-first-screen-ax.json').write_text(json.dumps(snap,ensure_ascii=False,indent=2))
    windows=[w for w in snap['windows'] if w.get('onscreen') and w.get('layer')==0 and w.get('bounds',{}).get('Width',0)>500]
    if windows:
        subprocess.run([str(cap),str(windows[0]['id']),str(out/'ordinary-first-screen.png')],check=True,capture_output=True,timeout=20)
    report['checks'].append('no legacy .opencodex or retired D/opencodex-home created')
    for eventLog in data.rglob('app.log'):
        (out/'final-app.log').write_text(eventLog.read_text())
    axtext=json.dumps(snap,ensure_ascii=False)
    assert 'Homebrew' not in axtext and 'MissingNode' not in axtext,'stale system discovery guide'
    report['installation']=str(install)
    report['result']='pass'
except Exception as e:
    import traceback
    report['result']='fail';report['error']=str(e);(out/'exception.log').write_text(traceback.format_exc())
finally:
    if proc and proc.poll() is None:
        os.killpg(proc.pid,signal.SIGTERM)
        try:proc.wait(timeout=8)
        except subprocess.TimeoutExpired:os.killpg(proc.pid,signal.SIGKILL);proc.wait(timeout=8)
    report['ownedExit']=proc.returncode if proc else None
    report['binarySha256']=hashlib.sha256(exe.read_bytes()).hexdigest()
    report['cleanup']='task-owned subprocess exited; sandbox evidence root removed after recording'
    (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
    shutil.rmtree(root)
    shutil.rmtree(install)
    report['ownedInstallationRemoved']=not install.exists()
    assert report['ownedInstallationRemoved']
    (out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+chr(10))
print(json.dumps(report,ensure_ascii=False))
raise SystemExit(0 if report['result']=='pass' else 1)
