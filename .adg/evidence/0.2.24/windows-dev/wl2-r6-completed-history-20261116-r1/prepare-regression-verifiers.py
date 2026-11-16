from pathlib import Path
ROOT=Path(__file__).resolve().parent
previous=ROOT.parent/'wl2-r6-handle-handoff-20261116-r1'
raw=(previous/'check-native-traces.py').read_bytes()
assert b"('native-traces-'" in raw
raw=raw.replace(b"('native-traces-'",b"('handoff-native-traces-'")
with (ROOT/'check-handoff-native-traces.py').open('xb') as out:out.write(raw)
raw=(previous/'check-recovery-native-traces.py').read_bytes();assert b'recovery-native-traces-' in raw
with (ROOT/'check-recovery-native-traces.py').open('xb') as out:out.write(raw)
print('Prepared unchanged physical-prefix assertions with distinct per-scope output files')
