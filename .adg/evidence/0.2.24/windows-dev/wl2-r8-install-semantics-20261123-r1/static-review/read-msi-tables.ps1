param([Parameter(Mandatory)][string]$Msi, [Parameter(Mandatory)][string]$Out)
$ErrorActionPreference = "Stop"

$installer = New-Object -ComObject WindowsInstaller.Installer
$db = $installer.GetType().InvokeMember("OpenDatabase", "InvokeMethod", $null, $installer, @($Msi, 0))

function Get-Field($rec, [int]$index) {
    try { return [string]$rec.GetType().InvokeMember("StringData", "GetProperty", $null, $rec, @($index)) }
    catch { return "" }
}

function Query([string]$sql) {
    $view = $db.GetType().InvokeMember("OpenView", "InvokeMethod", $null, $db, @($sql))
    $view.GetType().InvokeMember("Execute", "InvokeMethod", $null, $view, $null)
    $rows = New-Object System.Collections.ArrayList
    while ($true) {
        $rec = $view.GetType().InvokeMember("Fetch", "InvokeMethod", $null, $view, $null)
        if ($null -eq $rec) { break }
        $count = [int]$rec.GetType().InvokeMember("FieldCount", "GetProperty", $null, $rec, $null)
        $cells = New-Object System.Collections.ArrayList
        for ($i = 1; $i -le $count; $i++) { [void]$cells.Add((Get-Field $rec $i)) }
        [void]$rows.Add((@($cells) -join " | "))
    }
    $view.GetType().InvokeMember("Close", "InvokeMethod", $null, $view, $null)
    return $rows
}

$sections = [ordered]@{
    "Property" = "SELECT Property, Value FROM Property"
    "Directory" = "SELECT Directory, Directory_Parent, DefaultDir FROM Directory"
    "Component" = "SELECT Component, ComponentId, Directory_ FROM Component"
    "File" = "SELECT File, Component_, FileName, FileSize FROM File"
    "CustomAction" = "SELECT Action, Type, Source, Target FROM CustomAction"
    "InstallExecuteSequence" = "SELECT Action, Condition, Sequence FROM InstallExecuteSequence"
}

$lines = New-Object System.Collections.ArrayList
$counts = New-Object System.Collections.ArrayList
foreach ($name in $sections.Keys) {
    $rows = Query $sections[$name]
    [void]$lines.Add("=== " + $name + " (" + $rows.Count + ") ===")
    foreach ($row in $rows) { [void]$lines.Add($row) }
    [void]$counts.Add($name + "=" + $rows.Count)
}
[IO.File]::WriteAllLines($Out, $lines, (New-Object Text.UTF8Encoding($false)))
Write-Output (($counts) -join ", ")

