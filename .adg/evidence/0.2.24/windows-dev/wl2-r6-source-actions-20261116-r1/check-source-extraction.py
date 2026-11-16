from pathlib import Path
import hashlib,json,re,subprocess
ROOT=Path(__file__).resolve().parent
CODE=Path(r"C:/Users/15119/.codex/worktrees/024-windows-native/code")
old=subprocess.check_output(["git","-C",str(CODE),"show","e7c18e97ce5e7f65ed989ea2f15ac6a5d2cc62bc:apps/desktop/tauri/src/infrastructure/windows_client_originals024/v2/adapter.rs"]).decode()
new=(CODE/"apps/desktop/tauri/src/infrastructure/windows_client_originals024/source.rs").read_text(encoding="utf-8")
token=re.compile(r'"(?:\\.|[^"\\])*"|//[^\n]*|[A-Za-z_][A-Za-z_0-9]*|\d+|[^\s]')
def body(text,impl,name):
    start=text.index("impl "+impl+" {");start=text.index("fn "+name+"(",start)
    words=[w for w in token.findall(text[start:]) if not w.startswith('//')]
    start=words.index('{');depth=0;out=[]
    for w in words[start:]:
        if w=='{':depth+=1
        if w=='}':depth-=1
        out.append(w)
        if depth==0:break
    assert depth==0
    return out
rows=[]
for kind,name in [("CurrentPin","bytes"),("CurrentPin","capture"),("CurrentPin","check"),("Source","capture"),("Source","check")]:
    before=body(old,kind,name);after=body(new,kind,name);assert before==after,(kind,name)
    rows.append(dict(method=f"{kind}::{name}",rustBodyTokensEqual=True,tokens=len(before),sha256=hashlib.sha256(json.dumps(before).encode()).hexdigest()))
with (ROOT/"source-extraction-verification.json").open("x",encoding="utf-8") as out:json.dump(dict(baseline="e7c18e97ce5e7f65ed989ea2f15ac6a5d2cc62bc",checks=rows,meaning="all five original strict native method bodies token-identical after module move; context factory is additional and separately scoped",nativeAcceptance=False),out,indent=2)
print(json.dumps(dict(methodsVerified=len(rows),tokensEqual=True)))
