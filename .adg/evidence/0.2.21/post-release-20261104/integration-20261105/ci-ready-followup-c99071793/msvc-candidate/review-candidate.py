import argparse,hashlib,json,struct,subprocess,zipfile
from pathlib import Path
p=argparse.ArgumentParser();p.add_argument('--zip',required=True);p.add_argument('--artifact',required=True);p.add_argument('--run',required=True);p.add_argument('--repo',required=True);p.add_argument('--expected-head',required=True);p.add_argument('--expected-run',required=True,type=int);p.add_argument('--out',required=True);a=p.parse_args()
archive=Path(a.zip);artifact=json.loads(Path(a.artifact).read_text());run=json.loads(Path(a.run).read_text())
assert artifact['name']=='windows-msvc-candidate' and not artifact['expired']
assert run['id']==a.expected_run and run['head_sha']==a.expected_head
assert artifact['workflow_run']['id']==run['id'] and artifact['workflow_run']['head_sha']==a.expected_head
zip_hash=hashlib.sha256(archive.read_bytes()).hexdigest();assert artifact['digest']=='sha256:'+zip_hash
assert archive.stat().st_size==artifact['size_in_bytes']
with zipfile.ZipFile(archive) as z:
    assert sorted(z.namelist())==['candidate.json','opencodex-desktop.exe']
    for item in z.infolist():
        assert item.file_size < 512*1024*1024 and not item.flag_bits&1
        assert item.external_attr>>16&0o170000 != 0o120000
    m=json.loads(z.read('candidate.json').decode('utf-8-sig'));exe=z.read('opencodex-desktop.exe')
assert m['schemaVersion']==1 and int(m['runId'])==run['id'] and int(m['runAttempt'])==run['run_attempt']
assert m['requestedHead']==a.expected_head and m['target']=='x86_64-pc-windows-msvc'
assert m['profile']=='release; custom-protocol'
assert m['executable']=='opencodex-desktop.exe' and m['bytes']==len(exe)
exe_hash=hashlib.sha256(exe).hexdigest();assert exe_hash==m['sha256']
checked=m['checkedCommit'];assert len(checked)==40 and all(c in '0123456789abcdef' for c in checked)
def git(*args):return subprocess.check_output(['git','-C',a.repo,*args],text=True).strip()
assert git('rev-parse',checked+':apps')==m['appsTree']==git('rev-parse',a.expected_head+':apps')
parents=git('show','-s','--format=%P',checked).split();assert checked==a.expected_head or a.expected_head in parents
assert exe[:2]==b'MZ';offset=struct.unpack_from('<I',exe,0x3c)[0]
assert exe[offset:offset+4]==b'PE'+bytes([0,0])
assert struct.unpack_from('<H',exe,offset+4)[0]==0x8664
assert struct.unpack_from('<H',exe,offset+24)[0]==0x20b
assert checked.encode() in exe
result={'result':'candidate identity verified','run':run['id'],'attempt':run['run_attempt'],'associatedHead':a.expected_head,'checkedCommit':checked,'parents':parents,'appsTree':m['appsTree'],'target':m['target'],'zipSha256':zip_hash,'artifactId':artifact['id'],'executableSha256':exe_hash,'executableBytes':len(exe),'pe':'PE32+ AMD64','embeddedCheckedCommitFound':True,'candidateManifest':m,'executed':False,'scope':'Build/copy/provenance integrity only; does not prove native smoke, RDP performance, packaged Codex restart, installer/upgrade/updater or release acceptance.'}
Path(a.out).write_text(json.dumps(result,indent=2)+chr(10));print(json.dumps({k:result[k] for k in ['result','run','artifactId','executableSha256','checkedCommit']}))
