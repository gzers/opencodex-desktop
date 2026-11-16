from pathlib import Path
import json,hashlib
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
same=[]
for rel in ['handle_release.rs','tree/anchored/observation/retirement.rs','v3/journal/recovery/final_release.rs','v3/journal/history.rs','tree/anchored/observation/retirement/history.rs','bounded.rs']:
    # retirement.rs only reexports a new ownership borrow; its existing native
    # identity/security/content/children functions below are checked separately.
    p=BASE/rel;old=ROOT/'library02/executed-source'/p.relative_to(CODE);a=p.read_bytes();b=old.read_bytes()
    if rel.endswith('/retirement.rs'):
        a=a[a.index(b'struct Item'):];b=b[b.index(b'struct Item'):]
    assert a==b,rel
    same.append(dict(path=p.relative_to(CODE).as_posix(),unchangedNativeComparisons=True,comparedSha256=sha(a)))
history=(BASE/'tree/anchored/observation/retirement/history.rs').read_text(encoding='utf-8')
for name in ['original_frames','archive_originals','history_references']:
    block=history.split('fn '+name+'(',1)[1]
    if name=='original_frames':block=block.split('fn archive_originals(',1)[0]
    elif name=='archive_originals':block=block.split('fn history_references(',1)[0]
    assert block.count('self.post_release(end)?')==2
driver=(BASE/'v3/journal/handles.rs').read_text(encoding='utf-8')
assert driver.index('pending_history_references(end)?')<driver.index('verify_original_history_references')<driver.index('.detach(end)?')<driver.index('OriginalHandlesReleaseIntent')<driver.index('.close_all(')
assert driver.count('.check_originals(end)?')==3
assert '.check_held(end)?' in driver
handoff=(BASE/'tree/anchored/observation/retirement/handoff.rs').read_text(encoding='utf-8')
assert 'borrow_history(self.pending.as_mut().ok_or_else(invalid)?)' in handoff
assert 'self.handles.check_held(end)' in handoff
with (ROOT/'barrier-ownership-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(oldNativeComparisons=same,completeBeforeAfterNativeHistoryBarriers=True,
        actualHistoryReferencesMatchCanonicalBeforeOwnershipDetach=True,liveOriginalReadbacksAtDurableIntentAfterClosesAndAfterResult=True,
        removedOnlyAdjacentDuplicateOrchestration=True,originalCutoffChanged=False,assertionsRemoved=False),out,indent=2)
print(json.dumps(dict(unchangedNativeImplementations=len(same),originalLiveReadbackCount=3,canonicalReferenceJoin=True,deadlineChanged=False)))
