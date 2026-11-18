$ErrorActionPreference='Stop'
$tokens=$null
$errors=$null
$ast=[System.Management.Automation.Language.Parser]::ParseFile($args[0],[ref]$tokens,[ref]$errors)
if($errors.Count){throw ($errors | Out-String)}
foreach($name in @('Check','SameData','CheckOriginalRangeProof')){
    $fn=$ast.Find({param($node) $node -is [System.Management.Automation.Language.FunctionDefinitionAst] -and $node.Name -ceq $name},$false)
    if(!$fn){throw "Missing $name"}
    . ([ScriptBlock]::Create($fn.Extent.Text))
}
$id=[guid]::NewGuid().ToString()
$reply=[pscustomobject]@{requestId=$id}
$injection=[pscustomobject]@{ownerSessionId=[guid]::NewGuid().ToString(); failedActionId=[guid]::NewGuid().ToString(); hostInstanceId=[guid]::NewGuid().ToString(); scopeDigest='a'*64;workerThread='ThreadId(17)';originalClaimAndNlkVerified=[ordered]@{claimKey='b'*64;epoch=[guid]::NewGuid().ToString();objectsDigest='c'*64}}
$proof=[pscustomobject]@{requestId=$id;transportActionId=[guid]::NewGuid().ToString();sessionId=$injection.ownerSessionId;failedActionId=$injection.failedActionId;hostInstanceId=$injection.hostInstanceId;scopeDigest=$injection.scopeDigest;workerThread=$injection.workerThread;startRawError=31;creatorResumed=$false;originalRanges=$injection.originalClaimAndNlkVerified}
$injection=$injection|ConvertTo-Json -Depth 32|ConvertFrom-Json
$proof=$proof|ConvertTo-Json -Depth 32|ConvertFrom-Json
CheckOriginalRangeProof $proof $reply $injection
$replacements=[ordered]@{requestId=[guid]::NewGuid().ToString();transportActionId=$injection.failedActionId;sessionId=[guid]::NewGuid().ToString();failedActionId=[guid]::NewGuid().ToString();hostInstanceId=[guid]::NewGuid().ToString();scopeDigest='e'*64;workerThread='ThreadId(18)';startRawError=32;creatorResumed=$true;originalRanges=[pscustomobject]@{claimKey='e'*64;epoch=$proof.originalRanges.epoch;objectsDigest=$proof.originalRanges.objectsDigest}}
foreach($field in $replacements.Keys){
    $saved=$proof.$field
    $proof.$field=$replacements[$field]
    $refused=$false
    try { CheckOriginalRangeProof $proof $reply $injection } catch { $refused=$true }
    if(!$refused){throw "Accepted replacement for $field"}
    $proof.$field=$saved
}
@{version=$PSVersionTable.PSVersion.ToString();parsed=$true;accepted=1;refused=$replacements.Count}|ConvertTo-Json -Compress