from pathlib import Path
ROOT=Path(__file__).resolve().parent
prior=ROOT.parent/'wl2-r6-recovery-native-chain-20261116-r1'/'check-native-traces.py'
raw=prior.read_bytes()
with (ROOT/'check-recovery-native-traces.py').open('xb') as out:out.write(raw)
print('Copied exact existing recovery-prefix verifier for affected unchanged native counterexamples')
