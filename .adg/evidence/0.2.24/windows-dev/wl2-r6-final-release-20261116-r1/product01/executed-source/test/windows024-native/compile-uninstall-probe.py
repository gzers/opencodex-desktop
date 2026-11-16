"""Compile the actual NSIS preuninstall macro into a no-delete native probe.

The probe invokes the real installed ocxd helper at /D=I, then writes a marker
only after the hook permits continuation. It never performs Install/RemoveFiles.
"""
import argparse
import hashlib
import json
from pathlib import Path
import subprocess

ROOT = Path(__file__).resolve().parents[2]
parser = argparse.ArgumentParser(description=__doc__)
parser.add_argument('--makensis', type=Path, required=True)
parser.add_argument('--evidence', type=Path, required=True)
args = parser.parse_args()
out = args.evidence.resolve()
out.mkdir(parents=True, exist_ok=False)
hook = ROOT / 'apps/desktop/tauri/windows/installer-hooks.nsh'
source = r'''Unicode true
RequestExecutionLevel user
SilentInstall silent
Name "OpenCodeX W1-C preuninstall-only probe"
InstallDir "placeholder"
!include LogicLib.nsh
!include FileFunc.nsh
Var UpdateMode
Var DeleteAppDataCheckboxState
Function .onInit
  StrCpy $UpdateMode 0
  StrCpy $DeleteAppDataCheckboxState 0
  @{GetOptions} $CMDLINE "/UPDATE" $0
  @{IfNot} @{Errors}
    StrCpy $UpdateMode 1
  @{EndIf}
FunctionEnd
Section
  !insertmacro NSIS_HOOK_PREUNINSTALL
  FileOpen $0 "$INSTDIR\hook-passed.txt" w
  FileWrite $0 "hook permitted continuation; no deletion performed"
  FileClose $0
  SetErrorLevel 0
SectionEnd
'''.replace('@{', '$' + '{')
source = f'OutFile "{out / "hook-probe.exe"}"\n!include "{hook}"\n' + source
script = out / 'hook-probe.nsi'
script.write_text(source, encoding='utf-8')
command = [str(args.makensis.resolve()), '-INPUTCHARSET', 'UTF8', '-V4', str(script)]
with (out / 'compile.stdout.log').open('wb') as stdout, (out / 'compile.stderr.log').open('wb') as stderr:
    result = subprocess.run(command, stdout=stdout, stderr=stderr)
report = {'command': command, 'exitCode': result.returncode, 'scope': 'exact real hook plus no-delete marker only',
          'hookSha256': hashlib.sha256(hook.read_bytes()).hexdigest()}
if result.returncode == 0:
    report['probeSha256'] = hashlib.sha256((out / 'hook-probe.exe').read_bytes()).hexdigest()
(out / 'result.json').write_text(json.dumps(report, indent=2) + '\n')
raise SystemExit(result.returncode)
