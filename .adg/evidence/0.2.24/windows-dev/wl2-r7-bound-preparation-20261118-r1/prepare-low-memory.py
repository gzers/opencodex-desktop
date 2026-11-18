from pathlib import Path
ROOT=Path(__file__).resolve().parent
text=(ROOT/'run-product.ps1').read_text(encoding='utf-8');assert 'profile.test.package.opencodex-desktop.codegen-units=16' in text
text=text.replace('profile.test.package.opencodex-desktop.codegen-units=16','profile.test.package.opencodex-desktop.codegen-units=4')
(ROOT/'run-product-low-memory.ps1').write_text(text,encoding='utf-8',newline=chr(10))
print('command-only package codegen16->4, j1/offline/locked/test assertions and native action budgets unchanged; no system memory policy change')
