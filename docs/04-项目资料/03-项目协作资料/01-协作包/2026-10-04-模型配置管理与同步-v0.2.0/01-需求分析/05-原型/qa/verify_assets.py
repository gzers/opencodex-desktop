"""只读核对唯一原型目录、历史来源指纹和静态依赖；仅显式 --report 写新报告。"""
from pathlib import Path
from html.parser import HTMLParser
from urllib.parse import urlsplit, unquote
from datetime import date
import argparse
import hashlib
import json
import os
import re
import shutil
import subprocess
import tempfile

parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--date', default=date.today().isoformat(), help='本次核对日期')
parser.add_argument('--report', type=Path, help='可选新报告路径；拒绝覆盖已有文件')
args = parser.parse_args()
if args.report and args.report.exists():
    parser.error('已有报告拒绝覆盖，请指定新的报告路径')
here = Path(__file__).resolve().parent
root = next(p for p in here.parents if (p / '.git').exists())
product = root / 'docs/04-项目资料/01-项目支撑资料/01-产品原型'
retired = root / 'docs/04-项目资料/03-项目协作资料/03-共享原型'
manifest = json.loads((product / '迁移清单.json').read_text())
results = []


def check(name, good, detail=None):
    results.append(dict(name=name, status='PASS' if good else 'FAIL', detail=detail))
    print(results[-1]['status'], name)


class Assets(HTMLParser):
    def __init__(self):
        super().__init__()
        self.refs = []

    def handle_starttag(self, tag, attrs):
        attrs = dict(attrs)
        keys = ['src', 'data-proxy'] if tag in ['script', 'img', 'iframe'] else ['href'] if tag == 'link' else []
        for key in keys:
            value = attrs.get(key, '')
            if value and not urlsplit(value).scheme and not value.startswith(('#', '//', 'data:')):
                self.refs.append(unquote(urlsplit(value).path))


check('原型唯一目录：产品入口存在且旧共享目录不存在', (product / '原型/index.html').is_file() and not retired.exists())
commit = manifest['sourceCommit']
with tempfile.TemporaryDirectory(prefix='opencodex-fixed-baseline-') as tmp:
    target = Path(tmp)
    names = subprocess.check_output(['git', 'ls-tree', '-r', '--name-only', commit, '--', manifest['source']], cwd=root, text=True).splitlines()
    for name in names:
        file = target / name
        file.parent.mkdir(parents=True, exist_ok=True)
        file.write_bytes(subprocess.check_output(['git', 'show', f'{commit}:{name}'], cwd=root))
    for item in manifest['moved']:
        data = (target / item['path']).read_bytes()
        check('原件指纹 ' + Path(item['path']).name, len(data) == item['bytes'] and hashlib.sha256(data).hexdigest() == item['sha256'])
    old = target / manifest['source'] / '原型/index.html'
    assets = Assets()
    assets.feed(old.read_text())
    missing = [ref for ref in assets.refs if not (old.parent / ref).exists()]
    ignored = [ref for ref in missing if '官方页面快照' in ref]
    check('固定旧基线已跟踪静态资源可恢复', not (set(missing) - set(ignored)), dict(restoredFiles=len(names), references=len(assets.refs), excludedLocalOnly=ignored))

source_root = root / manifest['source']
entries = [*product.rglob('*.html'), source_root / '原型/index.html', *source_root.glob('候选/**/*.html')]
for entry in entries:
    assets = Assets()
    assets.feed(entry.read_text())
    missing = [ref for ref in assets.refs if not (entry.parent / ref).resolve().exists() and '官方页面快照' not in ref]
    stale = [ref for ref in assets.refs if (entry.parent / ref).resolve().is_relative_to(retired)]
    check('当前依赖 ' + str(entry.relative_to(root)), not missing and not stale, dict(missing=missing, retiredReferences=stale))

# 历史正文、报告及固定 Git 引用保留原路径；可变的本地导航必须使用当前唯一入口。
invalid_links = []
product_links = 0
for doc in (root / 'docs').rglob('*.md'):
    for target in re.findall(r'\]\(([^)]+)\)', doc.read_text()):
        url = urlsplit(target)
        if url.scheme or target.startswith(('#', '//')):
            continue
        resolved = (doc.parent / unquote(url.path)).resolve()
        if resolved.is_relative_to(retired):
            invalid_links.append(dict(file=str(doc.relative_to(root)), target=target, reason='旧目录导航'))
        elif resolved.is_relative_to(product) or resolved.name == '20261009-产品原型唯一目录纠错.md':
            product_links += 1
            if not resolved.exists():
                invalid_links.append(dict(file=str(doc.relative_to(root)), target=target, reason='目标不存在'))
check('当前原型 Markdown 导航有效且不指向旧目录', not invalid_links, dict(checked=product_links, invalid=invalid_links))

helper = source_root / '原型/qa/prototype-location.mjs'
qa_scripts = [*here.glob('*.mjs'), *source_root.glob('原型/qa/*.mjs')]
windows_qa = source_root.parents[1] / '05-运行维护/20261008-Windows窗口外观与菜单适配/原型QA/windows-window.browser.mjs'
qa_scripts.append(windows_qa)
consumers = [p for p in qa_scripts if p.name not in ['source-diagnostics.mjs', 'prototype-location.mjs']]
not_using_helper = [str(p.relative_to(root)) for p in consumers if 'prototype-location.mjs' not in p.read_text()]
check('所有当前原型 QA 复用唯一目录定位模块', not not_using_helper, dict(consumers=len(consumers), invalid=not_using_helper))

node = os.environ.get('NODE_EXECUTABLE') or shutil.which('node')
if node:
    location_code = 'const p=await import(process.argv[1]);console.log(JSON.stringify({root:p.prototypeRoot,entry:p.prototypeEntry}));'
    location = subprocess.run([node, '--input-type=module', '-e', location_code, helper.as_uri()], capture_output=True, text=True)
    expected = dict(root=str(product), entry=str(product / '原型/index.html'))
    check('Node QA 默认根目录与资产核对一致', location.returncode == 0 and json.loads(location.stdout or '{}') == expected, location.stderr or None)
    for script in [*product.rglob('*.js'), *qa_scripts]:
        result = subprocess.run([node, '--check', str(script)], capture_output=True, text=True)
        check('JS 语法 ' + str(script.relative_to(root)), result.returncode == 0, result.stderr or None)
    with tempfile.TemporaryDirectory(prefix='opencodex-inline-check-') as tmp:
        for entry in [product / '原型/index.html', source_root / '原型/index.html']:
            inline = re.findall(r'<script(?![^>]*src=)[^>]*>(.*?)</script>', entry.read_text(), re.S)
            for i, code in enumerate(inline):
                file = Path(tmp) / f'inline-{i}.js'
                file.write_text(code)
                result = subprocess.run([node, '--check', str(file)], capture_output=True, text=True)
                check('入口 inline JS ' + str(entry.relative_to(root)) + f' [{i}]', result.returncode == 0, result.stderr or None)
else:
    check('Node 语法核对环境', False, '既有 Node 不可用')

report = dict(date=args.date, prototypeRoot=str(product.relative_to(root)), retiredRoot=str(retired.relative_to(root)), sourceCommit=commit, boundary='只读原型资料核对；不写运行配置，不覆盖历史证据，不含生产 API', results=results)
if args.report:
    args.report.parent.mkdir(parents=True, exist_ok=True)
    with args.report.open('x') as output:
        output.write(json.dumps(report, ensure_ascii=False, indent=2) + '\n')
print('TOTAL', len(results), '/ FAIL', sum(result['status'] == 'FAIL' for result in results))
raise SystemExit(any(result['status'] == 'FAIL' for result in results))
