from pathlib import Path
ROOT=Path(__file__).resolve().parent
previous=ROOT.parent/'wl2-r6-retirement-native-20261116-r1'
for name in ['verify_evidence.py','verify_git_payload.py','check-execution-boundary.py']:
    with (ROOT/name).open('xb') as out:out.write((previous/name).read_bytes())
print('Copied unchanged offline payload and private execution boundary verifiers')
