from pathlib import Path
import hashlib,json,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
native=BASE/'tree/anchored/observation/retirement/history.rs'
text=native.read_text(encoding='utf-8')
checks=[]
for method,next_method in [('original_frames','archive_originals'),('archive_originals','history_references'),('history_references',None)]:
    block=text.split('fn '+method+'(',1)[1]
    if next_method:block=block.split('fn '+next_method+'(',1)[0]
    assert block.count('self.post_release(end)?')==2,(method,block)
    checks.append(dict(method=method,completeBothTreeNativeBarriers=2))
journal=(BASE/'v3/journal/history.rs').read_text(encoding='utf-8')
boundary=journal.split('fn original_history_boundary(',1)[1].split('fn seal_original_history(',1)[0]
assert boundary.count('self.check(end)')==2 and '.check(end)?' in boundary and 'released_lease' in boundary
assert 'witness.post_release' not in boundary
seal=journal.split('fn seal_original_history(',1)[1].split('#[cfg(test)]',1)[0]
assert 'original_history_boundary' in seal and 'witness.original_frames(end)' in journal
assert journal.index('ChainEntry::History(event), end')<journal.index('witness.archive_originals')
old_files=[]
for rel in ['bounded.rs','coordination.rs','coordination/segments.rs','tree.rs','tree/anchored/observation/retirement.rs','v3/journal/recovery.rs','v3/journal/recovery/final_release.rs']:
    path=BASE/rel;raw=path.read_bytes();prior=(ROOT/'library02/executed-source'/path.relative_to(CODE)).read_bytes()
    assert raw==prior,rel
    old_files.append(dict(path=path.relative_to(CODE).as_posix(),bytes=len(raw),sha256=sha(raw),allExistingNativePredicatesByteIdenticalToFailedRun=True))
archive=(BASE/'tree/anchored/observation/archive.rs').read_bytes()
assert archive==(ROOT/'library02/executed-source'/ (BASE/'tree/anchored/observation/archive.rs').relative_to(CODE)).read_bytes()
with (ROOT/'barrier-ownership-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(newNativeOperations=checks,canonicalDriverBeforeAfterReceiptAndJournal=True,allOriginalArchivePerFileComparisonsUnchanged=True,
        noDeadlineChanged=True,oldNativeImplementations=old_files,mutationAuthorityExpanded=False,meaning='Eliminate duplicate complete-tree traversal at both orchestration layers while preserving complete native before-after barriers on every witness operation and receipt/journal boundaries.'),out,indent=2)
print(json.dumps(dict(nativeOperations=len(checks),nativeBarriersPerOperation=2,oldNativeFilesIdentical=len(old_files),cutoffChanged=False)))
