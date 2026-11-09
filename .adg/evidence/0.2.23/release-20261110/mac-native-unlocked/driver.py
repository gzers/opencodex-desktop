import argparse, hashlib, json, os, plistlib, pwd, shutil, signal, subprocess, tempfile, time
from pathlib import Path

p=argparse.ArgumentParser();p.add_argument('--app',type=Path,required=True);p.add_argument('--source',required=True);p.add_argument('--out',type=Path,required=True);a=p.parse_args()
repo=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
assert not a.out.exists();a.out.mkdir(parents=True)
root=Path(tempfile.mkdtemp(prefix='ocxd023-signed-native-',dir='/private/tmp')).resolve()
install=root/'installation';install.mkdir();app=install/'OpenCodeX Desktop.app'
report={'accepted':False,'source':a.source,'checks':[],'result':'fail','scope':'Actual signed release copy: normal first startup/offline provisioning/native pages; proxy never started'}
proc=None
def sha(path):return hashlib.sha256(path.read_bytes()).hexdigest()
def run(argv,**kwargs):return subprocess.run([str(x) for x in argv],capture_output=True,text=True,timeout=60,**kwargs)
def inventory(path):return {str(x.relative_to(path)):sha(x) for x in path.rglob('*') if x.is_file()}
try:
    subprocess.run(['/usr/bin/ditto',str(a.app),str(app)],check=True)
    original=inventory(a.app);assert inventory(app)==original
    checked=run(['/usr/bin/codesign','--verify','--deep','--strict',app]);(a.out/'codesign.log').write_text(checked.stdout+checked.stderr);assert checked.returncode==0
    info=plistlib.loads((app/'Contents/Info.plist').read_bytes());assert info['CFBundleShortVersionString']=='0.2.23'
    exe=app/'Contents/MacOS'/info['CFBundleExecutable'];assert a.source.encode() in exe.read_bytes()
    report['binarySha256']=sha(exe)
    ax=root/'ax';cap=root/'cap'
    for src,dst in [(repo/'test/smoke/ocx021-macos-native.swift',ax),(repo/'test/smoke/lib/capwin.swift',cap)]:
        result=run(['/usr/bin/xcrun','swiftc',src,'-o',dst]);assert result.returncode==0,result.stderr
    permission=json.loads(run([ax,'status']).stdout);report['permissions']=permission
    assert permission['axTrusted'] and permission['screenCaptureAllowed'] and not permission['locked'],'Native screen permission unavailable'
    home=root/'home';home.mkdir();tmp=root/'tmp';tmp.mkdir()
    daily=pwd.getpwuid(os.getuid()).pw_dir;guard=root/'guard.sb'
    guard.write_text('(version 1)\n(allow default)\n(deny network-outbound)\n(deny file-read* file-write* (subpath '+json.dumps(daily)+'))\n(deny mach-lookup (global-name "com.apple.securityd") (global-name "com.apple.secd"))\n')
    env={'PATH':'/usr/bin:/bin:/usr/sbin:/sbin','LANG':'en_US.UTF-8','USER':pwd.getpwuid(os.getuid()).pw_name,'HOME':str(home),'CFFIXED_USER_HOME':str(home),'TMPDIR':str(tmp)+'/','TMP':str(tmp),'TEMP':str(tmp),'CODEX_HOME':str(home/'.codex')}
    denied=run(['/usr/bin/sandbox-exec','-f',guard,'/usr/bin/nc','-v','-G','1','-z','192.0.2.1','443'],env=env,cwd=root);assert denied.returncode!=0 and 'Operation not permitted' in denied.stderr
    (a.out/'network-guard.log').write_text(denied.stdout+denied.stderr)
    with (a.out/'process.log').open('wb') as log:proc=subprocess.Popen(['/usr/bin/sandbox-exec','-f',str(guard),str(exe)],env=env,cwd=root,stdout=log,stderr=subprocess.STDOUT,start_new_session=True)
    report['pid']=proc.pid;data=home/'.codex-desktop/desktop';runtime_home=home/'.codex-desktop/opencodex/home'
    deadline=time.monotonic()+240
    while time.monotonic()<deadline:
        assert proc.poll() is None,'Signed manager exited during supply'
        logpath=data/'logs/app.log';logtext=logpath.read_text(errors='replace') if logpath.exists() else ''
        if 'integrated runtime provisioning failed' in logtext:raise RuntimeError('Offline provision failed')
        if 'integrated runtime provisioned and verified' in logtext:break
        time.sleep(.5)
    else:raise RuntimeError('No complete supply receipt in original 240s window')
    assert 'commit='+a.source in logtext
    (a.out/'app.log').write_text(logtext)
    pointer=json.loads((data/'manager-location.json').read_text());assert pointer['activeRoot']==str(home/'.codex-desktop')
    assert str(json.loads((data/'manager-state/data-root.json').read_text())['structure_version'])=='2'
    source=json.loads((data/'manager-state/runtime.json').read_text());assert source['source']=='managed' and source['resolved_version']=='2.97.0-desktop023.lifecycle.3'
    package=json.loads((install/'runtime/opencodex/node_modules/@bitkyc08/opencodex/package.json').read_text());assert package['version']==source['resolved_version']
    assert runtime_home.is_dir() and not (home/'.opencodex').exists() and not (data/'opencodex-home').exists()
    report['checks'].append('Actual signed native normal startup supplied complete lifecycle.3 under independent installation; new U/D/H only')
    def snapshot(label):
        result=run([ax,'snapshot',proc.pid,exe]);assert result.returncode==0,result.stderr
        value=json.loads(result.stdout);(a.out/(label+'.json')).write_text(json.dumps(value,ensure_ascii=False,indent=2))
        assert not value['locked'];return value
    deadline=time.monotonic()+30
    while True:
        snap=snapshot('overview')
        if any(n.get('role')=='AXWebArea' for n in snap['nodes']):break
        if time.monotonic()>deadline:raise RuntimeError('Actual WebView unavailable')
        time.sleep(.3)
    run([ax,'focus',proc.pid,exe]).check_returncode()
    windows=[w for w in snap['windows'] if w.get('onscreen') and w.get('layer')==0 and w.get('bounds',{}).get('Width',0)>500]
    assert windows;run([cap,windows[0]['id'],a.out/'overview.png']).check_returncode()
    def press(label):
        s=snapshot('before-'+label);items=[n for n in s['nodes'] if label in [n.get('title'),n.get('description')] and 'AXPress' in n.get('actions',[])]
        assert len(items)==1,(label,len(items));result=run([ax,'press',proc.pid,exe,items[0]['path'],label]);assert result.returncode==0,result.stderr;time.sleep(.4)
    press('模型');snap=snapshot('models');assert any('新增渠道' in str(n) for n in snap['nodes'])
    press('设置');snapshot('settings')
    assert not (runtime_home/'runtime-port.json').exists(),'Unexpected service start'
    assert inventory(app)==original and inventory(a.app)==original
    report['checks'].append('PID-scoped native overview/models/settings observed; original signed app unchanged; no proxy start')
    report['result']='pass'
except Exception as error:
    import traceback
    report['error']=str(error);(a.out/'failure.log').write_text(traceback.format_exc())
finally:
    if proc and proc.poll() is None:
        assert proc.pid>1 and os.getpgid(proc.pid)==proc.pid
        os.killpg(proc.pid,signal.SIGTERM)
        try:proc.wait(timeout=10)
        except subprocess.TimeoutExpired:report['cleanupError']='Owned manager did not exit; retained test root'
    if proc:report['managerExit']=proc.returncode
    if not proc or proc.poll() is not None:
        shutil.rmtree(root);report['ownedRootRemoved']=True
    else:report['ownedRootRemoved']=False;report['result']='fail'
    (a.out/'report.json').write_text(json.dumps(report,ensure_ascii=False,indent=2)+chr(10))
print(json.dumps(report,ensure_ascii=False))
raise SystemExit(0 if report['result']=='pass' else 1)
