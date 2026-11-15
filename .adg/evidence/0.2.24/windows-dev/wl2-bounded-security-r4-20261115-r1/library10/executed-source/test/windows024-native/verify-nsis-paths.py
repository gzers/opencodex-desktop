"""Compile/execute only the exact NSIS root-selection functions, never Install.

The generated installer itself is compiled separately with actual024 resources.
This probe extracts .onInit/RestorePreviousInstallLocation and SetContext verbatim,
substitutes only the test registry key, and writes a resolved I into evidence.
"""
import argparse
import hashlib
import json
import os
from pathlib import Path
import subprocess
import sys
import traceback
import winreg


def extract(text, begin, end):
    start = text.index(begin)
    return text[start:text.index(end, start) + len(end)]


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument('--project', type=Path, required=True)
    parser.add_argument('--makensis', type=Path, required=True)
    parser.add_argument('--evidence', type=Path, required=True)
    args = parser.parse_args()
    evidence = args.evidence.resolve()
    evidence.mkdir(parents=True, exist_ok=False)
    report = {'passed': False, 'scope': 'actual NSIS root functions only; no installation',
              'installed': False, 'cells': []}
    key_path = f'Software\\OpenCodeX.Native024.InstallerPath.{os.getpid()}'
    owned_key = False
    try:
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path, 0, winreg.KEY_READ | winreg.KEY_WOW64_64KEY):
                raise ValueError('fixture registry key must be fresh')
        except FileNotFoundError:
            pass
        nsis = args.project.resolve() / 'target/release/nsis/x64'
        source = (nsis / 'installer.nsi').read_text(encoding='utf-8')
        init = extract(source, 'Function .onInit', 'FunctionEnd')
        restore = extract(source, 'Function RestorePreviousInstallLocation', 'FunctionEnd')
        context = extract((nsis / 'utils.nsh').read_text(encoding='utf-8'), '!macro SetContext', '!macroend')
        combined = context + '\n' + init + '\n' + restore
        report['extractedSourceSha256'] = hashlib.sha256(combined.encode()).hexdigest()
        # These routines contain no file deletion, installation or registry write.
        for command in ('Delete ', 'ExecWait ', 'WriteReg', 'RmDir ', 'File /'):
            if command in combined:
                raise ValueError('unexpected side effect in path-only functions')
        output = evidence / 'resolved.txt'
        exe = evidence / 'path-probe.exe'
        header = r'''Unicode true
RequestExecutionLevel user
SilentInstall silent
!include LogicLib.nsh
!include FileFunc.nsh
!include x64.nsh
!define PRODUCTNAME "OpenCodeX Desktop"
!define INSTALLMODE "currentUser"
!define ARCH "x64"
!define DISPLAYLANGUAGESELECTOR "false"
!define PLACEHOLDER_INSTALL_DIR "placeholder\@{PRODUCTNAME}"
Var PassiveMode
Var NoShortcutMode
Var UpdateMode
InstallDir "@{PLACEHOLDER_INSTALL_DIR}"
'''.replace('@{', '$' + '{')
        probe = header + f'OutFile "{exe}"\n!define MANUPRODUCTKEY "{key_path}"\n' + combined
        probe += f'\nSection\nFileOpen $0 "{output}" w\nFileWriteUTF16LE $0 "$INSTDIR"\nFileClose $0\nSetErrorLevel 0\nSectionEnd\n'
        script = evidence / 'path-probe.nsi'
        script.write_text(probe, encoding='utf-8')
        with (evidence / 'compile.stdout.log').open('wb') as out, (evidence / 'compile.stderr.log').open('wb') as err:
            code = subprocess.run([str(args.makensis.resolve()), '-INPUTCHARSET', 'UTF8', '-V4', str(script)], stdout=out, stderr=err).returncode
        report['compileExit'] = code
        if code:
            raise RuntimeError('native NSIS path probe compilation failed')
        default = Path(os.environ['LOCALAPPDATA']) / 'Programs/OpenCodeX Desktop'
        custom = evidence / 'custom 空格/Desktop'
        remembered = evidence / 'remembered 空格/Desktop'
        for mode, arguments, expected in (
            ('default', [], default),
            ('custom', ['/D=' + str(custom)], custom),
            ('remembered', [], remembered),
            ('explicit-over-remembered', ['/D=' + str(custom)], custom),
        ):
            if mode == 'remembered':
                with winreg.CreateKeyEx(winreg.HKEY_CURRENT_USER, key_path, 0, winreg.KEY_WRITE | winreg.KEY_WOW64_64KEY) as key:
                    owned_key = True
                    winreg.SetValueEx(key, '', 0, winreg.REG_SZ, str(remembered))
            # NSIS requires /D= last and unquoted, even for paths with spaces.
            # Python's automatic list2cmdline quotes the entire switch and the
            # native parser consequently ignores it. These are fixed test paths.
            if any('"' in text or '\n' in text for text in [str(exe), *arguments]):
                raise ValueError('ambiguous test command path')
            command = '"' + str(exe) + '" /S ' + ' '.join(arguments)
            result = subprocess.run(command, timeout=10)
            actual = output.read_text(encoding='utf-16-le')
            (evidence / (mode + '.txt')).write_text(actual, encoding='utf-8')
            output.unlink()
            if result.returncode or actual.rstrip('\\').casefold() != str(expected).rstrip('\\').casefold():
                raise RuntimeError(f'{mode} native resolved I differs: {actual!r}')
            report['cells'].append({'mode': mode, 'exitCode': result.returncode,
                                    'resolved': actual, 'expected': str(expected),
                                    'directoryCreated': Path(actual).exists()})
        report['passed'] = True
    except Exception as error:
        report['firstError'] = {'type': type(error).__name__, 'message': str(error)}
        traceback.print_exc()
    finally:
        if owned_key:
            winreg.DeleteKeyEx(winreg.HKEY_CURRENT_USER, key_path, winreg.KEY_WOW64_64KEY, 0)
        try:
            with winreg.OpenKey(winreg.HKEY_CURRENT_USER, key_path, 0, winreg.KEY_READ | winreg.KEY_WOW64_64KEY):
                report['cleanupKeyMissing'] = False
        except FileNotFoundError:
            report['cleanupKeyMissing'] = True
        (evidence / 'result.json').write_text(json.dumps(report, ensure_ascii=False, indent=2) + '\n', encoding='utf-8')
    return 0 if report['passed'] else 1


if __name__ == '__main__':
    sys.exit(main())
