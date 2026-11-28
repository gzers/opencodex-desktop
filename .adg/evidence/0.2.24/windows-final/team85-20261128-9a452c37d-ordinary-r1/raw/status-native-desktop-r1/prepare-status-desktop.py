from pathlib import Path
import hashlib
import sys


def replace_once(text, old, new):
    if text.count(old) != 1:
        raise ValueError(f'Unexpected source for {old!r}')
    return text.replace(old, new, 1)


fixture = Path('C:/Users/Public/ocx-team85-9a452c37d-r1')
previous = fixture / 'status-doctor-desktop-r2'
output = Path('../scratch/status-native-reasons-r2/desktop-input')
source = '1dac7c065413347ebc56b6123a8cf10c517449aa'
baseline = '9a452c37d3005119042d81ee1ae8f62c4920c4d0'
exe_hash = sys.argv[1].lower()
observer = (previous / 'observe-status-doctor.mjs').read_text(encoding='utf-8')
observer = observer.replace('closeSync, mkdirSync, openSync', 'closeSync, openSync')
observer = replace_once(observer, ", 'run_doctor'", '')
start = observer.index('async function callDoctor(')
end = observer.index('try {\n  const target =', start)
observer = observer[:start] + observer[end:]
start = observer.index('  if (report.preparation) {\n    await record(\'doctor\'')
end = observer.index('\n} catch (error) {', start)
observer = observer[:start].rstrip() + observer[end:]
observer = observer.replace(baseline, source).replace('__ocxTeam85StatusDoctor', '__ocxTeam85NativeStatus')
assert 'run_doctor' not in observer and 'callDoctor' not in observer
observer_bytes = observer.encode('utf-8')
observer_hash = hashlib.sha256(observer_bytes).hexdigest()

launcher = (previous / 'run-status-doctor-desktop.ps1').read_text(encoding='utf-8')
launcher = replace_once(launcher, "'status-doctor-desktop-r2-'", "'status-native-desktop-r1-'")
launcher = replace_once(launcher, f"source = '{baseline}'; artifactId", f"source = '{source}'; frozenPayloadSource = '{baseline}'; artifactId")
launcher = replace_once(launcher, "launchRequestedUtc = $null", f"managerSha256 = '{exe_hash}'; diagnosticOnly = $true; launchRequestedUtc = $null")
launcher = replace_once(launcher, "        Copy-Item -LiteralPath (Join-Path (Join-Path $fixture 'program') $member) -Destination $target", """        if ($member -ceq 'opencodex-desktop.exe') {
            $diagnostic = Join-Path $PSScriptRoot 'opencodex-desktop.exe'
            Assert-Digest $diagnostic $report.managerSha256
            Copy-Item -LiteralPath $diagnostic -Destination $target
            Assert-Digest $target $report.managerSha256
            continue
        }
        Copy-Item -LiteralPath (Join-Path (Join-Path $fixture 'program') $member) -Destination $target""")
launcher = replace_once(launcher, "    $report.observerSha256 = (Get-FileHash -LiteralPath $observerScript -Algorithm SHA256).Hash", f"    $report.observerSha256 = '{observer_hash}'\n    Assert-Digest $observerScript $report.observerSha256")
launcher = replace_once(launcher, "    $env:OPENCODEX_SANDBOX = '1'", "    $env:OPENCODEX_SANDBOX = '1'\n    $env:OCXD024_STATUS_TRACE = '1'")
start = launcher.index('    $observer = Start-Process')
end = launcher.index('        $report.launchRequestedUtc', start)
launcher = launcher[:start] + """    $observerInfo = [Diagnostics.ProcessStartInfo]::new($node, $arguments)
    $observerInfo.WorkingDirectory = $program
    $observerInfo.UseShellExecute = $false
    $observerInfo.CreateNoWindow = $true
    $observerInfo.RedirectStandardOutput = $true
    $observerInfo.RedirectStandardError = $true
    $observer = [Diagnostics.Process]::Start($observerInfo)
    $report.observerPid = $observer.Id
    try {
        $stdout = $observer.StandardOutput.ReadToEndAsync()
        $stderr = $observer.StandardError.ReadToEndAsync()
""" + launcher[end:]
launcher = replace_once(launcher, '        $report.observerExitCode = $observer.ExitCode', """        $report.observerExitCode = $observer.ExitCode
        if ($stdout.Wait(2000)) { [IO.File]::WriteAllText((Join-Path $sandbox 'observer-stdout.log'), $stdout.Result) }
        if ($stderr.Wait(2000)) { [IO.File]::WriteAllText((Join-Path $sandbox 'observer-stderr.log'), $stderr.Result) }""")
launcher = replace_once(launcher, '    if ($report.trace.firstError)', """    $nativeLog = Read-Log (Join-Path $sandbox 'manager-stderr.log')
    $report.nativeStages = @($nativeLog -split '\\r?\\n' | Where-Object { $_ -match '^STATUS024_WINDOWS stage=' })
    if ($report.trace.firstError)""")
for name, contents in [('observe-status-doctor.mjs', observer_bytes), ('run-status-native-desktop.ps1', launcher.encode('utf-8'))]:
    with (output / name).open('xb') as file:
        file.write(contents)
