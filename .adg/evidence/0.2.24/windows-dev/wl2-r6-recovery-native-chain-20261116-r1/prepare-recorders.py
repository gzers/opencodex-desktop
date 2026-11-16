from pathlib import Path
ROOT=Path(__file__).resolve().parent
previous=ROOT.parent/'wl2-r6-recovery-chain-20261116-r1'
for name in ['archive_run.py','verify_evidence.py','verify_git_payload.py']:
    with (ROOT/name).open('xb') as out:out.write((previous/name).read_bytes())
print('Copied original native artifact/archive and raw payload recorders')
