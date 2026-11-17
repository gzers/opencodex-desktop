from pathlib import Path
import hashlib,json
ROOT=Path(__file__).resolve().parent
prior=ROOT.parent/'wl2-r6-native-stop-history-20261117-r1'
source=prior/'commit_source.py'
text=source.read_text(encoding='utf-8')
replacements={
"18a079fa4cf1c628bab1ad9c3e67513e0a227402":"a653a51acea51dbba5b4e6b4a2488f8b53f09ce4",
"results['library02']==48 and results['library03']==4":"results['library02']==51 and results['library03']==8",
"source-scope-final.json":"source-scope-verified.json",
"assert len(paths)==14":"assert len(paths)==6",
"feat(windows): 保留原Stop完成历史与最终同链回执":"feat(windows): 区分同窗状态与原动作历史读取",
}
for old,new in replacements.items():assert old in text; text=text.replace(old,new)
with (ROOT/'commit_source.py').open('x',encoding='utf-8') as f:f.write(text)
with (ROOT/'recorder-provenance.json').open('x',encoding='utf-8') as f:json.dump(dict(source=str(source),sourceSha256=hashlib.sha256(source.read_bytes()).hexdigest(),replacements=replacements),f,ensure_ascii=False,indent=2)
print(json.dumps(dict(prepared='commit_source.py',expectedFiles=6,finalData=51,release=8)))
