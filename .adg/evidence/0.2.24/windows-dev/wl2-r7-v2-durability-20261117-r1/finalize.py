from pathlib import Path
import hashlib,json,shutil,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code')
PREV=ROOT.parent/'wl2-r7-v2-reference-20261117-r1'
def read(p):return json.loads(p.read_text(encoding='utf-8-sig'))
def write(n,v):(ROOT/n).write_text(json.dumps(v,ensure_ascii=False,indent=2)+'\n',encoding='utf-8')
head=subprocess.check_output(['git','rev-parse','HEAD'],cwd=CODE).decode().strip()
assert head.startswith('c3b99c0fa')
assert subprocess.check_output(['git','rev-parse','origin/codex/024-windows-native'],cwd=CODE).decode().strip()==head
sources=read(ROOT/'source-manifest.json')
checks=[]
for row in sources['files']:
    raw=(CODE/row['path']).read_bytes()
    blob=subprocess.check_output(['git','show',f"{head}:{row['path']}"],cwd=CODE)
    assert len(raw)==row['bytes'] and hashlib.sha256(raw).hexdigest()==row['sha256']
    assert raw.replace(b'\r\n',b'\n')==blob.replace(b'\r\n',b'\n')
    checks.append(dict(path=row['path'],normalizedGitBytesEqual=True,gitSha256=hashlib.sha256(blob).hexdigest()))
write('source-git-verification.json',dict(codeSha=head,files=checks))
origin=read(ROOT/'implementation-origin.json');origin['templateHashBasis']='UTF8 text with LF normalized by Path.read_text; not raw CRLF hash';write('implementation-origin.json',origin)
old=read(PREV/'status.json');matrix=old['matrix']
for row in matrix:
    if row['item']=='V2 remaining native scope':row['status']='native V2 DATA durability now6actual tests/26scenes passed; original typed producer, full originalset proof and partial restoration still incomplete; journal mapping remains aggregate membership'
matrix.extend([
 dict(item='TEAM28/IMP160 docs intake',status='docs857fbd672 reception-only revision consumed; same Windows/r7 write set; repaired mergec17b031f1 preserves localhandoff and authoritativeIMP, prior sealed evidence89blobs verified'),
 dict(item='independent native V2 DATA durability',status='actual full product MSVC lib test raw0/145459ms,6tests/26scenes passed; only strict V2 schema/chain, separatev2-record names, actual original native files/ID/all8/ADS/bytes/complete members before+after replay'),
 dict(item='V2 every native write failure stage',status='5actual stages precreate/empty/halfwrite/fullwrite/published keep original filehandles+raw prefix; no reappend/read-success afterfailure; original deadline unchanged'),
 dict(item='V2 beforewrite reference/version rejection',status='7actual storage scenes replay/scope/unknown/gap/v1/reference/preparation-switch reject before second create; exact first native file bytes retained'),
 dict(item='V2 native late drift/cutoff/membership',status='6actual member/fileSD/dirSD/ADS/latefileSD/latemember and4cutoff/nonempty scenes reject; no namespace adoption/rewindow'),
 dict(item='V2 preparation DATA native storage',status='3actual persistence scenes complete-declared/Pending/unknownterminal-refuse; synthetic declared IDs remainDATA, no native originalOwner/restore permit'),
 dict(item='V2 durability cleanup and generated files',status='26new own roots explicitly absent, schemas rawarchived/trackedrestored/untrackedmoved; oldrejected/driverfailedroots retained; no old deletion retry'),
])
write('status.json',dict(codeSha=head,codeBranch='codex/024-windows-native',mainInput=old['mainInput'],docsInput='857fbd672 / merged docs c17b031f1',authority='TEAM28/IMP160/r7 unchanged',scope='strict independent V2 native DATA persistence; no native originalOwner proof generation or restoration acceptance',runs=dict(product01=read(ROOT/'product01/result.json')),uniqueNewNativeTests=6,newNativeScenes=26,artifact=read(ROOT/'artifact.json'),firstUnexpectedProductError=None,operationalErrors='operational-errors.json; corrected without product assertion changes',sourceFilesVerified=4,publicStartOpen=False,productionAccepted=False,allAccepted=False,currentR2Touched=False,currentR2ExecutionAuthorized=False,goal='active',cleanup=read(ROOT/'cleanup-readback.json'),matrix=matrix))
for n in ['verify_evidence.py','verify_git_payload.py']:shutil.copyfile(PREV/n,ROOT/n)
(ROOT/'.gitattributes').write_text('* -text whitespace=cr-at-eol,-blank-at-eof\n',encoding='utf-8')
print(json.dumps(dict(codeSha=head,matrix=len(matrix),sourceCount=4)))
