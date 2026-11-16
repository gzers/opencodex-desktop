from pathlib import Path
ROOT=Path(__file__).resolve().parent
p=ROOT/'finalize.py';text=p.read_text(encoding='utf-8')
old='明确Vec<String>修复后native3/5/1定向及准确同EXE22回归过。'
new='明确Vec<String>修复后native3/5/1定向过；准确同debugEXE回归21过1败，既有artifact用例在原10秒bounded安全读回超时保留；仅失败用例采用同Rust/断言/原截止与原release参数定向，结果独立列出，不加预算。'
assert text.count(old)==1
p.write_text(text.replace(old,new),encoding='utf-8')
print('Corrected handoff generator to preserve observed debug timeout; release result still verified independently.')
