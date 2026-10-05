"""只读核对迁移来源、临时恢复旧基线及当前静态依赖。不会写运行配置。"""
from pathlib import Path
from html.parser import HTMLParser
from urllib.parse import urlsplit, unquote
import hashlib, json, os, subprocess, tempfile
here = Path(__file__).resolve().parent
root = next(p for p in here.parents if (p / '.git').exists())
shared = root / 'docs/04-项目资料/03-项目协作资料/03-共享原型'
manifest = json.loads((shared / '迁移清单.json').read_text())
results = []
def check(name, good, detail=None):
    results.append(dict(name=name, status='PASS' if good else 'FAIL', detail=detail))
    print(results[-1]['status'], name)
class Assets(HTMLParser):
    def __init__(self):
        super().__init__(); self.refs=[]
    def handle_starttag(self,tag,attrs):
        a=dict(attrs)
        for key in ['src','data-proxy'] if tag in ['script','img','iframe'] else ['href'] if tag=='link' else []:
            value=a.get(key,'')
            if value and not urlsplit(value).scheme and not value.startswith(('#','//','data:')):
                self.refs.append(unquote(urlsplit(value).path))
commit=manifest['sourceCommit']
with tempfile.TemporaryDirectory(prefix='opencodex-fixed-baseline-') as tmp:
    target=Path(tmp)
    names=subprocess.check_output(['git','ls-tree','-r','--name-only',commit,'--',manifest['source']],cwd=root,text=True).splitlines()
    for name in names:
        file=target/name;file.parent.mkdir(parents=True,exist_ok=True)
        file.write_bytes(subprocess.check_output(['git','show',f'{commit}:{name}'],cwd=root))
    for item in manifest['moved']:
        data=(target/item['path']).read_bytes()
        check('原件指纹 '+Path(item['path']).name,len(data)==item['bytes'] and hashlib.sha256(data).hexdigest()==item['sha256'])
    old=target/manifest['source']/'原型/index.html'
    a=Assets();a.feed(old.read_text())
    missing=[ref for ref in a.refs if not (old.parent/ref).exists()]
    ignored=[r for r in missing if '官方页面快照' in r]
    check('固定旧基线已跟踪静态资源可恢复',not(set(missing)-set(ignored)),dict(restoredFiles=len(names),references=len(a.refs),excludedLocalOnly=ignored))
oldroot=root/manifest['source']
entries=[shared/'原型/index.html',oldroot/'原型/index.html']
entries += [p for p in oldroot.glob('候选/**/*.html') if '03-共享原型' in p.read_text()]
for entry in entries:
    a=Assets();a.feed(entry.read_text())
    missing=[r for r in a.refs if not(entry.parent/r).resolve().exists() and '官方页面快照' not in r]
    check('当前依赖 '+str(entry.relative_to(root)),not missing,missing)
node=Path(os.environ.get('NODE_EXECUTABLE',Path.home()/'.cache/codex-runtimes/codex-primary-runtime/dependencies/node/bin/node'))
if node.exists():
    scripts=[*shared.rglob('*.js'),*here.glob('*.mjs'),*oldroot.glob('原型/qa/*.mjs')]
    for p in scripts:
        r=subprocess.run([str(node),'--check',str(p)],capture_output=True,text=True)
        check('JS 语法 '+p.name,r.returncode==0,r.stderr or None)
    html=(shared/'原型/index.html').read_text()
    import re
    inline=re.findall(r'<script(?![^>]*src=)[^>]*>(.*?)</script>',html,re.S)
    for i,code in enumerate(inline):
        f=Path(tempfile.gettempdir())/f'opencodex-inline-check-{i}.js';f.write_text(code)
        r=subprocess.run([str(node),'--check',str(f)],capture_output=True,text=True);f.unlink()
        check('主入口 inline JS '+str(i),r.returncode==0,r.stderr or None)
else:
    check('Node 语法核对环境',False,'既有 Node 不可用')
report=dict(date='2026-10-04',sourceCommit=commit,boundary='只读资料校验，不含生产 API',results=results)
(here/'../资源与迁移核对结果.json').resolve().write_text(json.dumps(report,ensure_ascii=False,indent=2)+'\n')
raise SystemExit(any(r['status']=='FAIL' for r in results))
