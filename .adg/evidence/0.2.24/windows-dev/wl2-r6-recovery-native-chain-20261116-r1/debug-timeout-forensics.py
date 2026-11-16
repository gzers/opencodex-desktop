from pathlib import Path
from ctypes import wintypes as w
import ctypes as c,hashlib,json,re
ROOT=Path(__file__).resolve().parent
K=c.WinDLL('kernel32',use_last_error=True)
class Info(c.Structure):
    _fields_=[('attributes',w.DWORD),('created',w.FILETIME),('accessed',w.FILETIME),('written',w.FILETIME),
        ('volume',w.DWORD),('sizeHigh',w.DWORD),('sizeLow',w.DWORD),('links',w.DWORD),('indexHigh',w.DWORD),('indexLow',w.DWORD)]
K.CreateFileW.argtypes=[w.LPCWSTR,w.DWORD,w.DWORD,c.c_void_p,w.DWORD,w.DWORD,w.HANDLE];K.CreateFileW.restype=w.HANDLE
K.GetFileInformationByHandle.argtypes=[w.HANDLE,c.POINTER(Info)];K.GetFileInformationByHandle.restype=w.BOOL
K.CloseHandle.argtypes=[w.HANDLE];K.CloseHandle.restype=w.BOOL
def inspect(path,directory=False):
    h=K.CreateFileW(str(path),0x80,7,None,3,0x200000|(0x2000000 if directory else 0),None)
    if h==c.c_void_p(-1).value:return dict(path=str(path),openRawError=c.get_last_error())
    try:
        info=Info()
        if not K.GetFileInformationByHandle(h,c.byref(info)):return dict(path=str(path),queryRawError=c.get_last_error())
        row=dict(path=str(path),volume=info.volume,fileId=str(info.indexHigh*2**32+info.indexLow),links=info.links,attributes=info.attributes)
        if not directory:
            raw=path.read_bytes();row.update(bytes=len(raw),sha256=hashlib.sha256(raw).hexdigest())
        return row
    finally:assert K.CloseHandle(h)
text=(ROOT/'library03/stdout.log').read_text(encoding='utf-8-sig')
rows=[]
targets=['wl2_r6_recovery_chain_native_all_current_and_original_nodes_use_same_canonical_chain','wl2_r6_recovery_chain_native_interrupted_original_restore_preserves_exact_native_prefix']
for name,body in re.findall(r'test ([^\n]+?) \.\.\. (.*?)(?=\ntest |\nfailures:|\ntest result:|\Z)',text,re.S):
    if name.rsplit('::',1)[-1] not in targets:continue
    assert body.rstrip().endswith('FAILED')
    paths=re.findall(r'r4-retained-synthetic-fixture=([^\r\n]+)',body)
    data=[Path(p)/'session' for p in paths if p.startswith('D:')];assert len(data)==1
    directory=data[0];events=[json.loads(p.read_bytes()) for p in sorted(directory.glob('event-*.json'))]
    recovery=[e for e in events if e.get('documentKind')=='stopped-recovery-event'];assert recovery
    plan=recovery[0]['plan'];positions=[]
    for root in plan['roots']:
        original=root['originalTree'];live=Path(original['root']);custody=Path(original['custody'])
        for n in original['nodes']:
            locations=[inspect(live/n['relative'],n['kind']=='DirectoryAnchor')]
            if n['kind']=='CustodiedFile':locations.append(inspect(custody/f"original-{n['nodeId']}"))
            matches=[p for p in locations if p.get('fileId')==n['fileId'] and p.get('volume')==n['volume']]
            assert len(matches)==1,(n['relative'],locations)
            positions.append(dict(kind='original',relative=n['relative'],expectedId=n['fileId'],locations=locations))
        for n in root['retirements']:
            locations=[inspect(live/n['relative']),inspect(Path(n['target']))]
            matches=[p for p in locations if p.get('fileId')==n['currentFileId'] and p.get('volume')==n['currentVolume']]
            assert len(matches)==1 and matches[0]['bytes']==n['bytes'] and matches[0]['sha256']==n['sha256'],(n['relative'],locations)
            positions.append(dict(kind='current',relative=n['relative'],expectedId=n['currentFileId'],locations=locations))
    raw=(directory/'current.json').read_bytes();current=json.loads(raw)
    rows.append(dict(test=name,scene=str(directory),lastRecoveryEvent=recovery[-1],immutableRecoveryEvents=len(recovery),
        current=current,currentRawSha256=hashlib.sha256(raw).hexdigest(),positions=positions,
        timing=re.findall(r'r6 recovery native canonical component phase=([^\r\n]+)',body)))
assert len(rows)==2
with (ROOT/'debug-timeout-forensics.json').open('x',encoding='utf-8') as out:
    json.dump(dict(rawExit=101,passed=21,failed=2,originalCutoffUnchanged=True,observations=rows,
        readOnlyAfterOriginalDeadline=True,freshReadIsNotMutationOrRecoveryPermission=True,newSecurityQualification=False,cleanupAttempted=False),out,indent=2)
print(json.dumps(dict(failures=len(rows),actualNativePositions=sum(len(r['positions']) for r in rows),mutated=False)))
