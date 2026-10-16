# Prepare a diagnostic profile from this host's WPR export. This helper never
# starts/stops tracing, configures heap instrumentation, or launches a product.
function ConvertTo-WindowsGpuAllocationProfile {
    [CmdletBinding()]
    param([Parameter(Mandatory)][string]$ExportedProfileXml)
    $ErrorActionPreference='Stop'
    if($ExportedProfileXml.Length -gt 262144){throw 'Bounded WPR export required'}
    $taskSettings=[Xml.XmlReaderSettings]::new()
    $taskSettings.DtdProcessing=[Xml.DtdProcessing]::Prohibit
    $taskSettings.XmlResolver=$null
    $taskReader=[Xml.XmlReader]::Create([IO.StringReader]::new($ExportedProfileXml),$taskSettings)
    $taskSource=[Xml.XmlDocument]::new()
    $taskSource.XmlResolver=$null
    try{$taskSource.Load($taskReader)}finally{$taskReader.Dispose()}
    if($taskSource.DocumentElement.Name -cne 'WindowsPerformanceRecorder' -or
       $taskSource.DocumentElement.GetAttribute('Version') -cne '1.0' -or
       $taskSource.DocumentElement.NamespaceURI){throw 'Recognized WPR export schema required'}
    $taskSources=@($taskSource.SelectNodes('/WindowsPerformanceRecorder/Profiles/SystemProvider') |
        Where-Object { $_.SelectSingleNode("Keywords/Keyword[@Value='VirtualAllocation']") })
    if($taskSources.Count -ne 1){throw 'One exported virtual-allocation system provider required'}
    $taskKeywords=@('ProcessThread','Loader','VirtualAllocation','MemoryInfo','MemoryInfoWS')
    $taskStacks=@('VirtualAllocation','ProcessCreate')
    foreach($taskKeyword in $taskKeywords){
        if(!$taskSources[0].SelectSingleNode("Keywords/Keyword[@Value='$taskKeyword']")){throw "Export lacks required keyword: $taskKeyword"}
    }
    foreach($taskStack in $taskStacks){
        if(!$taskSources[0].SelectSingleNode("Stacks/Stack[@Value='$taskStack']")){throw "Export lacks required stack: $taskStack"}
    }
    $taskGraphics=@(
        @{Id='OcxD3D11';Guid='ca11c036-0102-4a2d-a6ad-f03cfed5d3c9';Level='6';Keyword='0x000000000000000f'}
        @{Id='OcxDXGI';Guid='db6f6ddb-ac77-4e88-8253-819df9bbf140';Level='6';Keyword='0x000000000000000f'}
        @{Id='OcxDxgKrnl';Guid='802ec45a-1e99-4b83-9920-87c98277ba9d';Level='5';Keyword='0x0000000000000277'}
    )
    $taskProviderCopies=@()
    foreach($taskDefinition in $taskGraphics){
        $taskProvider=@($taskSource.SelectNodes('/WindowsPerformanceRecorder/Profiles/EventProvider') |
            Where-Object { $_.GetAttribute('Name') -ceq $taskDefinition.Guid })
        if($taskProvider.Count -ne 1 -or $taskProvider[0].GetAttribute('Stack') -cne 'true' -or
           $taskProvider[0].GetAttribute('Level') -cne $taskDefinition.Level){throw 'Expected exported graphics provider and stack configuration required'}
        $taskProviderKeywords=@($taskProvider[0].SelectNodes('Keywords/Keyword') | ForEach-Object GetAttribute 'Value')
        if($taskProviderKeywords.Count -ne 1 -or $taskProviderKeywords[0] -cne $taskDefinition.Keyword){throw 'Exported graphics keywords differ'}
        $taskCapture=$taskProvider[0].SelectSingleNode('CaptureStateOnStart')
        if(!$taskCapture -or $taskCapture.GetAttribute('Timeout') -cne '30' -or
           $taskCapture.SelectNodes('Keyword').Count -ne 1 -or
           $taskCapture.FirstChild.GetAttribute('Value') -cne $taskDefinition.Keyword){throw 'Exported graphics capture-state configuration differs'}
        $taskProviderCopies+=@{Definition=$taskDefinition;Node=$taskProvider[0]}
    }
    $taskDocument=[Xml.XmlDocument]::new()
    $taskTop=$taskDocument.CreateElement('WindowsPerformanceRecorder')
    $taskTop.SetAttribute('Version','1.0')
    [void]$taskDocument.AppendChild($taskTop)
    $taskProfiles=$taskDocument.CreateElement('Profiles')
    [void]$taskTop.AppendChild($taskProfiles)
    foreach($taskCollector in @(@{Kind='SystemCollector';Id='OcxAllocationSystem'},@{Kind='EventCollector';Id='OcxAllocationGraphics'})){
        $taskNode=$taskDocument.CreateElement($taskCollector.Kind)
        $taskNode.SetAttribute('Id',$taskCollector.Id)
        $taskNode.SetAttribute('Name',$taskCollector.Id)
        foreach($taskSize in @(@{Name='BufferSize';Value='256'},@{Name='Buffers';Value='64'})){
            $taskChild=$taskDocument.CreateElement($taskSize.Name)
            $taskChild.SetAttribute('Value',$taskSize.Value)
            [void]$taskNode.AppendChild($taskChild)
        }
        [void]$taskProfiles.AppendChild($taskNode)
    }
    $taskSystem=$taskDocument.CreateElement('SystemProvider')
    $taskSystem.SetAttribute('Id','OcxAllocationKernel')
    foreach($taskGroup in @(@{Name='Keywords';Item='Keyword';Values=$taskKeywords},@{Name='Stacks';Item='Stack';Values=$taskStacks})){
        $taskNode=$taskDocument.CreateElement($taskGroup.Name)
        foreach($taskValue in $taskGroup.Values){
            $taskChild=$taskDocument.CreateElement($taskGroup.Item)
            $taskChild.SetAttribute('Value',$taskValue)
            [void]$taskNode.AppendChild($taskChild)
        }
        [void]$taskSystem.AppendChild($taskNode)
    }
    [void]$taskProfiles.AppendChild($taskSystem)
    foreach($taskCopy in $taskProviderCopies){
        $taskNode=$taskDocument.ImportNode($taskCopy.Node,$true)
        $taskNode.SetAttribute('Id',$taskCopy.Definition.Id)
        [void]$taskProfiles.AppendChild($taskNode)
    }
    $taskProfile=$taskDocument.CreateElement('Profile')
    foreach($taskAttribute in @{Id='OcxGpuAllocation.Verbose.File';Name='OcxGpuAllocation';Description='Diagnostic GPU and virtual allocation stacks; never a performance acceptance run';LoggingMode='File';DetailLevel='Verbose'}.GetEnumerator()){
        $taskProfile.SetAttribute($taskAttribute.Key,$taskAttribute.Value)
    }
    $taskCollectors=$taskDocument.CreateElement('Collectors')
    $taskSystemReference=$taskDocument.CreateElement('SystemCollectorId')
    $taskSystemReference.SetAttribute('Value','OcxAllocationSystem')
    $taskProviderReference=$taskDocument.CreateElement('SystemProviderId')
    $taskProviderReference.SetAttribute('Value','OcxAllocationKernel')
    [void]$taskSystemReference.AppendChild($taskProviderReference)
    [void]$taskCollectors.AppendChild($taskSystemReference)
    $taskEventReference=$taskDocument.CreateElement('EventCollectorId')
    $taskEventReference.SetAttribute('Value','OcxAllocationGraphics')
    $taskEventProviders=$taskDocument.CreateElement('EventProviders')
    foreach($taskDefinition in $taskGraphics){
        $taskChild=$taskDocument.CreateElement('EventProviderId')
        $taskChild.SetAttribute('Value',$taskDefinition.Id)
        [void]$taskEventProviders.AppendChild($taskChild)
    }
    [void]$taskEventReference.AppendChild($taskEventProviders)
    [void]$taskCollectors.AppendChild($taskEventReference)
    [void]$taskProfile.AppendChild($taskCollectors)
    [void]$taskProfiles.AppendChild($taskProfile)
    $taskText=[Text.StringBuilder]::new()
    $taskWriterSettings=[Xml.XmlWriterSettings]::new()
    $taskWriterSettings.Indent=$true
    $taskWriterSettings.OmitXmlDeclaration=$true
    $taskWriter=[Xml.XmlWriter]::Create($taskText,$taskWriterSettings)
    try{$taskDocument.Save($taskWriter)}finally{$taskWriter.Dispose()}
    [pscustomobject][ordered]@{
        profileXml=$taskText.ToString()
        profileName='OcxGpuAllocation.Verbose'
        configuredBufferBytes=2L*256*1024*64
        systemKeywords=$taskKeywords;systemStacks=$taskStacks
        graphicsProviders=@($taskGraphics | ForEach-Object { $_.Guid })
        exportedGraphicsSettingsPreserved=$true
        scope='diagnostic preparation only'
        traceStarted=$false;productStarted=$false
        fileGrowthBoundEnforced=$false;eventLossVerified=$false
        executionAdmitted=$false;formalRssMetricReplaced=$false
    }
}
