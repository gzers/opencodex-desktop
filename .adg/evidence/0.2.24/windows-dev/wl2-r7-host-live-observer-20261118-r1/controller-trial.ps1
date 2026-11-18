$ErrorActionPreference = 'Stop'
$code = 'C:/Users/15119/.codex/worktrees/r6-windows-native-resume/code'
$native = Join-Path $code 'test/windows024-native/archive-r6-process-facts.cs'
$receipt = Get-Content -LiteralPath '.adg/evidence/0.2.24/windows-dev/wl2-r7-nine-r2-host-20261118-r1/raw/Host-creation.json' -Raw | ConvertFrom-Json
Add-Type -Path $native
$own = [R6ArchiveProcessFacts]::Observe([uint32]$PID)
$expected = [Diagnostics.Process]::GetCurrentProcess()
try {
  if ($own.pid -ne $PID -or $own.exited -or $own.session -ne $expected.SessionId -or
      $own.filetime -cne $expected.StartTime.ToUniversalTime().ToFileTimeUtc().ToString() -or
      $own.image -ine $expected.MainModule.FileName) { throw 'Native positive control differed' }
} finally { $expected.Dispose() }
$result = [ordered]@{ nativeSha256 = (Get-FileHash -LiteralPath $native).Hash.ToLowerInvariant(); observedUtc = [DateTime]::UtcNow.ToString('o'); controller = $own; targetReceipt = $receipt; queriedRaw = $null; nativeResult = $null; liveProcessVerified = $false; error = $null }
try {
  $facts = [R6ArchiveProcessFacts]::Observe([uint32]$receipt.pid)
  $result.queriedRaw = 0
  $result.nativeResult = $facts
  $result.liveProcessVerified = !$facts.exited -and $facts.pid -eq $receipt.pid -and $facts.session -eq 7 -and $facts.image -ieq $receipt.image -and $facts.filetime -ceq $receipt.creationFiletime
  if (!$result.liveProcessVerified) { $result.error = 'Native process generation, liveness, image or session differed.' }
} catch {
  $err = $_.Exception.GetBaseException()
  if ($err -is [ComponentModel.Win32Exception]) { $result.queriedRaw = $err.NativeErrorCode }
  $result.error = $err.Message
}
$result | ConvertTo-Json -Depth 8
