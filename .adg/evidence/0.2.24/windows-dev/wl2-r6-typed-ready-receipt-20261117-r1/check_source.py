from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent;CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code');BASE='df0c8c24fdb1e5394290cecc28a4bf68f815dbc4'
git=lambda *a:subprocess.check_output(['git','-C',str(CODE),*a]);sha=lambda b:hashlib.sha256(b).hexdigest()
paths=git('diff','--name-only',BASE).decode().splitlines();assert set(paths)=={'apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024.rs','apps/desktop/tauri/src/infrastructure/windows_official_lifecycle024/actions.rs','test/windows024-native/build-wl2.ps1'}
preserved=[];prefix='apps/desktop/tauri/src/infrastructure/'
for p in [prefix+'windows_client_originals024/v3/capture.rs',prefix+'windows_client_originals024/v3/journal/closed_stop.rs',prefix+'windows_client_originals024/v3/journal/completed.rs',prefix+'windows_client_originals024/v3/journal/handles.rs',prefix+'windows_client_originals024/coordination.rs',prefix+'windows_client_originals024/content.rs',prefix+'windows_diagnostic_roles024.rs',prefix+'official_runtime024_transport.rs','apps/desktop/tauri/src/modules/runtime/official_lifecycle024.rs']:
    old=git('show',f'{BASE}:{p}').replace(b'\r\n',b'\n');new=(CODE/p).read_bytes().replace(b'\r\n',b'\n');assert old==new,p;preserved.append(dict(path=p,normalizedSha256=sha(new)))
for p,start,stop in [(prefix+'windows_official_lifecycle024/actions.rs','    pub(super) fn admit(','/// Fresh private nonserde receipt'),
    (prefix+'windows_official_lifecycle024/actions.rs','    pub(crate) fn check(&self, owner: &mut Owner, end: Instant)',None),
    (prefix+'windows_official_lifecycle024.rs','    async fn stop(',None)]:
    old=git('show',f'{BASE}:{p}').decode().replace('\r\n','\n');new=(CODE/p).read_text(encoding='utf-8-sig')
    def section(s):
        i=s.index(start);return s[i:s.index(stop,i)] if stop else s[i:]
    if p.endswith('/actions.rs') and stop is None:
        def section(s):
            i=s.index(start);return s[i:s.index('#[cfg(test)]\nmod tests {',i)]
    if p.endswith('lifecycle024.rs'):
        # New constructor initializes only this new memory field; all original
        # Stop/Status/recovery/native startup code remains byte-identical.
        new=new.replace('        typed_ready: None,\n','')
    assert section(old)==section(new),p
    preserved.append(dict(path=p,section=start,normalizedSha256=sha(section(new).encode())))
gate=git('grep','-n','-A','10','fn require_durable_capture','--','apps/desktop/tauri/src/infrastructure').decode();assert 'Err(' in gate
with (ROOT/'source-scope.json').open('x',encoding='utf-8') as f:json.dump(dict(base=BASE,changedFiles=[dict(path=p,bytes=(CODE/p).stat().st_size,sha256=sha((CODE/p).read_bytes())) for p in paths],preserved=preserved,publicGateExcerpt=gate,actualTypedObservationFactoryExecuted=False),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(changed=3,preserved=len(preserved),sharedTypedServiceAndOldMutationChecksUnchanged=True)))
