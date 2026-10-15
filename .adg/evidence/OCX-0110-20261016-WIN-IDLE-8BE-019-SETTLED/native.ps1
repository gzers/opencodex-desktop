param(
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-z][a-z0-9-]+$')][string]$Launch,
    [Parameter(Mandatory=$true)][ValidatePattern('^[a-z][a-z0-9-]+$')][string]$Phase,
    [ValidateSet('inspect','click','keys','close','invoke','value','folderpath')][string]$Action='inspect',
    [int]$X=0,[int]$Y=0,[string]$Keys='',[string]$Name='',[string]$Value=''
)
$ErrorActionPreference='Stop'
$taskLaunch=Get-Content -LiteralPath (Join-Path $PSScriptRoot ($Launch+'-launch.json')) -Raw | ConvertFrom-Json
$taskProcess=Get-Process -Id $taskLaunch.pid
if($taskProcess.StartTime.ToUniversalTime().Ticks -ne ([datetimeoffset]$taskLaunch.creationTimeUtc).UtcTicks){throw 'PID generation changed'}
$taskOutput=Join-Path $PSScriptRoot ($Phase+'.json')
if(Test-Path -LiteralPath $taskOutput){throw 'Phase already recorded'}
$taskCode='C:\Users\15119\.codex\worktrees\0110-windows-gates\opencodex-desktop'
. (Join-Path $taskCode 'test/smoke/windows-process-token.ps1')
. (Join-Path (Split-Path -Parent $PSScriptRoot) 'OCX-0110-20261015-WIN-ANCHOR-PERSISTENCE/methods/process-user-sid.ps1')
$taskUser=Get-AcceptanceProcessSidEvidence -Process $taskProcess -ExpectedExecutable $taskLaunch.executable
Assert-WindowsProcessMode -Observation $taskUser -Mode $taskLaunch.mode
$taskHarness=Get-WindowsHarnessToken
if($Action -ne 'inspect'){Assert-WindowsProcessMode -Observation $taskHarness -Mode $taskLaunch.mode}
Add-Type -AssemblyName System.Windows.Forms,System.Drawing,UIAutomationClient,UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Runtime.InteropServices;
public static class RootNative {
 [StructLayout(LayoutKind.Sequential)] public struct Rect {public int Left,Top,Right,Bottom;}
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr w,out Rect r);
 [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr w,out Rect r);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr w);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint first,uint second,bool attach);
 [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr w);
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr w,out uint p);
 public static bool Activate(IntPtr w){
  uint ignored,current=GetCurrentThreadId(),foreground=GetWindowThreadProcessId(GetForegroundWindow(),out ignored),target=GetWindowThreadProcessId(w,out ignored);
  bool a=foreground!=0&&foreground!=current&&AttachThreadInput(current,foreground,true);
  bool b=target!=0&&target!=current&&target!=foreground&&AttachThreadInput(current,target,true);
  try{BringWindowToTop(w);return SetForegroundWindow(w);}
  finally{if(b)AttachThreadInput(current,target,false);if(a)AttachThreadInput(current,foreground,false);}
 }
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern void mouse_event(uint f,uint x,uint y,uint d,UIntPtr e);
 [DllImport("user32.dll")] public static extern void keybd_event(byte v,byte s,uint f,UIntPtr e);
 [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr w,uint m,IntPtr a,IntPtr b);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr w);
 [DllImport("user32.dll")] public static extern IntPtr GetMenu(IntPtr w);
 public delegate bool Callback(IntPtr w,IntPtr p);
 [DllImport("user32.dll")] public static extern bool EnumChildWindows(IntPtr w,Callback c,IntPtr p);
 [DllImport("user32.dll")] public static extern int GetDlgCtrlID(IntPtr w);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetClassNameW(IntPtr w,System.Text.StringBuilder s,int n);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern IntPtr SendMessageW(IntPtr w,uint m,IntPtr a,string b);
 public static IntPtr FolderEdit(IntPtr w){
  IntPtr found=IntPtr.Zero; int count=0;
  EnumChildWindows(w,(h,p)=>{var s=new System.Text.StringBuilder(256);GetClassNameW(h,s,256);
   if(GetDlgCtrlID(h)==1152&&s.ToString()=="Edit"){found=h;count++;}return true;},IntPtr.Zero);
  if(count!=1)throw new InvalidOperationException("Unique native folder edit required");return found;
 }
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr c);
 [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr w,uint a,out int n,int s);
}
'@
[void][RootNative]::SetThreadDpiAwarenessContext([IntPtr](-4))
$taskProcess.Refresh()
$taskWindow=$taskProcess.MainWindowHandle
$taskOwner=[uint32]0
[void][RootNative]::GetWindowThreadProcessId($taskWindow,[ref]$taskOwner)
if($taskWindow -eq [IntPtr]::Zero -or $taskOwner -ne $taskProcess.Id){throw 'Bound main HWND missing'}
$taskRect=[RootNative+Rect]::new()
[void][RootNative]::GetWindowRect($taskWindow,[ref]$taskRect)
$taskShell=New-Object -ComObject WScript.Shell
try{
    for($taskFocusAttempt=0;$taskFocusAttempt -lt 4;$taskFocusAttempt++){
        [void]$taskShell.AppActivate($taskProcess.Id)
        [void][RootNative]::Activate($taskWindow)
        Start-Sleep -Milliseconds 200
        if([RootNative]::GetForegroundWindow() -eq $taskWindow){break}
    }
    if([RootNative]::GetForegroundWindow() -ne $taskWindow){throw 'Bound window did not become foreground'}
    if($Action -eq 'click'){
        if($X -lt 0 -or $Y -lt 0 -or $X -ge ($taskRect.Right-$taskRect.Left) -or $Y -ge ($taskRect.Bottom-$taskRect.Top)){throw 'Click outside bound window'}
        [void][RootNative]::SetCursorPos($taskRect.Left+$X,$taskRect.Top+$Y)
        [RootNative]::mouse_event(2,0,0,0,[UIntPtr]::Zero)
        [RootNative]::mouse_event(4,0,0,0,[UIntPtr]::Zero)
    }elseif($Action -eq 'keys'){
        if(!$Keys){throw 'Keys required'}
        $taskShell.SendKeys($Keys)
    }elseif($Action -eq 'folderpath'){
        $taskEdit=[RootNative]::FolderEdit($taskWindow)
        $taskEditOwner=[uint32]0
        [void][RootNative]::GetWindowThreadProcessId($taskEdit,[ref]$taskEditOwner)
        if($taskEditOwner -ne $taskProcess.Id){throw 'Folder edit belongs to another process'}
        if([RootNative]::SendMessageW($taskEdit,0x0c,[IntPtr]::Zero,$Value) -eq [IntPtr]::Zero){throw 'Native folder edit rejected path'}
    }elseif($Action -in @('invoke','value')){
        $taskElement=[Windows.Automation.AutomationElement]::FromHandle($taskWindow)
        $taskControlType=if($Action -eq 'invoke'){[Windows.Automation.ControlType]::Button}else{[Windows.Automation.ControlType]::Edit}
        $taskConditions=[Windows.Automation.AndCondition]::new(
            [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::NameProperty,$Name),
            [Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,$taskControlType))
        $taskMatches=$taskElement.FindAll([Windows.Automation.TreeScope]::Descendants,$taskConditions)
        if($taskMatches.Count -ne 1 -or !$taskMatches[0].Current.IsEnabled){throw 'Unique enabled UI button required'}
        $taskMatches[0].SetFocus()
        if($Action -eq 'invoke'){
            $taskPattern=$taskMatches[0].GetCurrentPattern([Windows.Automation.InvokePattern]::Pattern)
            $taskPattern.Invoke()
        }else{
            $taskPattern=$taskMatches[0].GetCurrentPattern([Windows.Automation.ValuePattern]::Pattern)
            # SetValue alone does not establish a Vue input event. Delete a
            # sentinel through the focused native edit and verify the result.
            $taskPattern.SetValue($Value+'x')
            Start-Sleep -Milliseconds 150
            foreach($taskKey in @(0x23,0x08)){
                [RootNative]::keybd_event($taskKey,0,0,[UIntPtr]::Zero)
                Start-Sleep -Milliseconds 40
                [RootNative]::keybd_event($taskKey,0,2,[UIntPtr]::Zero)
                Start-Sleep -Milliseconds 60
            }
            Start-Sleep -Milliseconds 150
            if($taskPattern.Current.Value -cne $Value){throw 'Keyboard path entry did not retain the full value'}
        }
    }elseif($Action -eq 'close'){
        [void][RootNative]::PostMessageW($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
    }
    Start-Sleep -Milliseconds 450
}finally{[void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskShell)}
$taskBackdrop=0
$taskBackdropHr=[RootNative]::DwmGetWindowAttribute($taskWindow,38,[ref]$taskBackdrop,4)
$taskAx=@()
try{
    $taskElement=[Windows.Automation.AutomationElement]::FromHandle($taskWindow)
    $taskAll=$taskElement.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.Condition]::TrueCondition)
    for($taskIndex=0;$taskIndex -lt [Math]::Min(250,$taskAll.Count);$taskIndex++){
        $taskCurrent=$taskAll[$taskIndex].Current
        $taskAx+=@{name=$taskCurrent.Name;type=$taskCurrent.ControlType.ProgrammaticName;enabled=$taskCurrent.IsEnabled;bounds=$taskCurrent.BoundingRectangle.ToString()}
    }
}catch{$taskAx=@(@{error=$_.Exception.Message})}
$taskVisible=[RootNative]::IsWindowVisible($taskWindow)
$taskScreenSaved=$false
if($taskVisible -and [RootNative]::GetForegroundWindow() -eq $taskWindow){
    [void][RootNative]::GetWindowRect($taskWindow,[ref]$taskRect)
    $taskClip=[Drawing.Rectangle]::Intersect([Windows.Forms.SystemInformation]::VirtualScreen,[Drawing.Rectangle]::FromLTRB($taskRect.Left,$taskRect.Top,$taskRect.Right,$taskRect.Bottom))
    if($taskClip.Width -gt 0 -and $taskClip.Height -gt 0){
        $taskBitmap=[Drawing.Bitmap]::new($taskClip.Width,$taskClip.Height)
        $taskGraphics=[Drawing.Graphics]::FromImage($taskBitmap)
        try{$taskGraphics.CopyFromScreen($taskClip.Location,[Drawing.Point]::Empty,$taskClip.Size);$taskBitmap.Save((Join-Path $PSScriptRoot ($Phase+'.png')),[Drawing.Imaging.ImageFormat]::Png);$taskScreenSaved=$true}
        finally{$taskGraphics.Dispose();$taskBitmap.Dispose()}
    }
}
$taskPointer=$null
if(Test-Path -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop'){
    $taskPointer=(Get-ItemProperty -LiteralPath 'HKCU:\Software\com.gzers.opencodex.desktop').StartupDataRootV1
}
[ordered]@{
    atUtc=[datetimeoffset]::UtcNow.ToString('o');launch=$Launch;phase=$Phase;action=$Action;x=$X;y=$Y;keys=$Keys;name=$Name;value=$Value
    applicationUser=$taskUser;hwnd=$taskWindow.ToInt64();bounds=$taskRect
    visible=$taskVisible;foreground=([RootNative]::GetForegroundWindow() -eq $taskWindow)
    backdropHresult=$taskBackdropHr;backdropRaw=$taskBackdrop
    applicationMenuAttached=([RootNative]::GetMenu($taskWindow) -ne [IntPtr]::Zero)
    screenshotSaved=$taskScreenSaved;ax=$taskAx;startupPointer=$taskPointer
    physicalDpiGateClaimed=$false;cdpUsed=$false
} | ConvertTo-Json -Depth 9 | Set-Content -LiteralPath $taskOutput -Encoding utf8
Get-Content -LiteralPath $taskOutput
