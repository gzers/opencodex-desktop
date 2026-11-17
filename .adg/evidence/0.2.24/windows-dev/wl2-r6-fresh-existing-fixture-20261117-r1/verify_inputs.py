from pathlib import Path
import hashlib,io,json,subprocess,sys
R=Path(__file__).resolve().parent
root=Path(sys.argv[1]).resolve();revision='357210891f14a1851881e6192bd09d7031471707'
def git(*a,**kw):return subprocess.check_output(['git','-C',str(root),*a],**kw)
def norm(b):return b.replace(bytes([13,10]),bytes([10]))
expected={
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/existing_qualification.rs',
 'apps/desktop/tauri/src/infrastructure/windows_client_originals024/existing_qualification/fresh_fixture.rs',
 'test/windows024-native/build-wl2.ps1','test/windows024-native/launch-r6-official-existing-r2.ps1',
 'test/windows024-native/launch-r6-official-existing-r2.cmd','test/windows024-native/export-r6-official-existing-r2.ps1'}
assert set(git('diff','--name-only','323c3fc3ea41eae35177b43f80f0f40736b6bd6d',revision).decode().splitlines())==expected
allowed_script_delta={'test/windows024-native/launch-r6-official-existing-r2.ps1','test/windows024-native/launch-r6-official-existing-r2.cmd'}
reports=[]
for name in ['library01','product01','product02']:
    rows=json.loads((R/name/'source-manifest.json').read_text(encoding='utf-8-sig'))
    query=''.join(revision+':'+r['path']+chr(10) for r in rows).encode()
    stream=io.BytesIO(git('cat-file','--batch',input=query))
    raw_eol=[];nonexecuted_script_deltas=[];rust=0
    for row in rows:
        header=stream.readline().split();assert header[1]==b'blob',header
        blob=stream.read(int(header[2]));assert stream.read(1)==bytes([10])
        snapshot=(R/name/'executed-source'/row['path']).read_bytes()
        assert len(snapshot)==row['bytes'] and hashlib.sha256(snapshot).hexdigest()==row['sha256']
        if norm(snapshot)!=norm(blob):
            assert name=='library01' and row['path'] in allowed_script_delta,(name,row['path'])
            nonexecuted_script_deltas.append(row['path'])
        elif snapshot!=blob:raw_eol.append(row['path'])
        if row['path'].endswith('.rs'):rust+=1;assert norm(snapshot)==norm(blob)
    reports.append(dict(run=name,inputs=len(rows),rustInputs=rust,snapshotRawBytesVerified=True,
        rustMatchesCommit=True,onlyLineEndingDifferences=raw_eol,
        nonexecutedLauncherChangesAfterSnapshot=nonexecuted_script_deltas))
source=json.loads((R/'product02/source.json').read_text(encoding='utf-8-sig'))
assert source['head']==revision and not source['diff']
for row in json.loads((R/'artifacts.json').read_text(encoding='utf-8-sig')):
    raw=Path(row['path']).read_bytes();assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
print(json.dumps(dict(source=revision,allowedFiles=sorted(expected),runs=reports,
    allOtherTrackedCodeIncludingFullLifecycleStatusLeaseSharedContractsUnchanged=True,
    cleanFinalBuild=True,productSHA256Verified=True,nativeAuthority=False),indent=2))
