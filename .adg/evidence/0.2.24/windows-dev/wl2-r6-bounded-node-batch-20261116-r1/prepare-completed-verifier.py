from pathlib import Path
ROOT=Path(__file__).resolve().parent
old=ROOT.parent/'wl2-r6-completed-history-20261116-r1'
raw=(old/'check-native-traces.py').read_bytes()
assert b"('native-traces-'" in raw
raw=raw.replace(b"('native-traces-'",b"('completed-native-traces-'")
with (ROOT/'check-completed-native-traces.py').open('xb') as out:out.write(raw)
print('Prepared unchanged completed-window/canonical-physical verifier with distinct output prefix')
