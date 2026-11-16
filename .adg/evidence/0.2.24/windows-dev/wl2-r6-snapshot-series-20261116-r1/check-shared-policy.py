from pathlib import Path
import subprocess,hashlib,json
ROOT=Path(__file__).resolve().parent
CODE=Path('C:/Users/15119/.codex/worktrees/024-windows-native/code')
path='apps/desktop/tauri/src/modules/runtime/client_artifact_policy.rs'
old=subprocess.check_output(['git','-C',str(CODE),'show','eee0ad1a49442f6369373d34bf22ea610f1b1af4:'+path])
raw=(CODE/path).read_bytes()
assert old.replace(b'\r\n',b'\n')==raw.replace(b'\r\n',b'\n')
with (ROOT/'shared-policy-source-equality.json').open('x',encoding='utf-8') as out:
    json.dump(dict(path=path,baseline='eee0ad1a49442f6369373d34bf22ea610f1b1af4',workBytes=len(raw),workSha256=hashlib.sha256(raw).hexdigest(),gitLineEndingEquivalent=True,algorithmChanged=False,calledDirectly='client_artifact_policy::assess',authority='content data only; full native/source/phase predicates separate'),out,indent=2)
print(json.dumps(dict(policyUnchanged=True,workSha256=hashlib.sha256(raw).hexdigest())))
