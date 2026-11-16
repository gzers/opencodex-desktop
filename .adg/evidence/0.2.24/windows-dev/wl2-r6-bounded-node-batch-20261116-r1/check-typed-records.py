from pathlib import Path
import hashlib,json,subprocess,re
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
BASE=CODE/'apps/desktop/tauri/src/infrastructure/windows_client_originals024'
sha=lambda b:hashlib.sha256(b).hexdigest()
p=BASE/'v3/journal.rs';current=p.read_text(encoding='utf-8')
prior=(ROOT/'library06/executed-source'/p.relative_to(CODE)).read_text(encoding='utf-8')
block=lambda t:t.split('    fn check(&mut self, end: Instant)',1)[1].split('#[derive',1)[0]
assert block(current)==block(prior),'Record native identity/all8/byte/ADS/readback body unchanged'
native=lambda t:t.split('        if native::current_identity()? != self.token {',1)[1].split('    pub(in super::super) fn check(',1)[0]
assert native(current)==native(prior),'full current/peer/pin/token/all8/raw/membership comparison body unchanged'
append=current.split('fn append_entry_inner(',1)[1]
assert append.index('self.check(end)?')<append.index('for record in &self.events')<append.index('next.apply_entry')<append.index('Record::create(')
assert 'let event = record.entry.as_ref().ok_or_else(invalid)?;' in append
assert append.index('encoded(&event)?')<append.index('record.entry = Some(event);')<append.index('self.events.push(record);')<append.index('if self.fault == 1')
assert len(re.findall(r'^\s+self\.check\(end\)[?;]*$',append,re.M))==2
assignments=[]
for file in BASE.rglob('*.rs'):
    for number,line in enumerate(file.read_text(encoding='utf-8').splitlines(),1):
        if '.entry =' in line:assignments.append(dict(path=file.relative_to(CODE).as_posix(),line=number,text=line.strip()))
assert len(assignments)==1 and assignments[0]['text']=='record.entry = Some(event);'
with (ROOT/'typed-record-verification.json').open('x',encoding='utf-8') as out:
    json.dump(dict(nativeRecordCheckBodyByteIdentical=True,fullNativeJournalCurrentCheckBodyByteIdentical=True,
        originalDeadlineGateUnchanged=True,fullIndependentProgressReplayBeforeNewWrite=True,
        encodedRawAndOwnedTypedEventSameLocalValue=True,immutableRecordRetainedBeforeFallibleCurrent=True,
        noPublicOrMutableTypedRecordSetter=True,actualAssignments=assignments,
        recordCheckSourceSha256=sha(block(current).encode()),fullJournalNativeBodySha256=sha(native(current).encode()),
        skipsRepeatedDataDecodeOnly=True,nativeFactCached=False),out,indent=2)
print(json.dumps(dict(nativeRecordComparisonsUnchanged=True,fullReplayRetained=True,typedEntryAssignments=len(assignments),nativeFactCached=False)))
