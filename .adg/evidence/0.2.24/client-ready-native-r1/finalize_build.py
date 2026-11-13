from pathlib import Path
import hashlib,json,re,subprocess
root=Path(__file__).resolve().parent;code=Path('/Users/ezio/.codex/worktrees/ocxd-023-integration/ocx-0219-fixes')
assert json.loads((root/'release-build-final.json').read_text())['exitCode']==0
source=json.loads((root/'source-before-final-build.json').read_text())
assert subprocess.check_output(['git','rev-parse','HEAD'],cwd=code,text=True).strip()==source['baseCommit']
for item in source['files']:assert hashlib.sha256((code/item['path']).read_bytes()).hexdigest()==item['sha256'],item['path']
match=re.search(r'Executable unittests src/lib.rs \(([^)]+)\)',(root/'release-build-final.log').read_text());assert match
binary=code/'apps/desktop/tauri'/match.group(1)
source['binarySha256']=hashlib.sha256(binary.read_bytes()).hexdigest();source['binaryPath']=str(binary)
source['releaseBuild']=json.loads((root/'release-build-final.json').read_text())
source['scope']='Current final test code plus parent product commit compiled under existing release opt-level=s/LTO. No fixed app or source publication claim.'
(root/'compiled-source.json').write_text(json.dumps(source,indent=2))
print(json.dumps({'sourceFiles':len(source['files']),'binarySha256':source['binarySha256'],'binaryPath':source['binaryPath']},indent=2))
