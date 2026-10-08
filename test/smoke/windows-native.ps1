param(
 [Parameter(Mandatory=$true)][string]$Executable,
 [Parameter(Mandatory=$true)][string]$Node,
 [Parameter(Mandatory=$true)][string]$OutputDirectory,
 [ValidateSet('ordinary','administrator')][string]$Mode='ordinary'
)
# Native-caption checks own one sandbox process and never install a package.
$ErrorActionPreference='Stop'
if(!$IsWindows -or $PSVersionTable.PSVersion.Major -ne 7){throw 'Windows PowerShell 7 required'}
$taskExe=(Resolve-Path -LiteralPath $Executable).Path
$taskNode=(Resolve-Path -LiteralPath $Node).Path
$taskOutput=[IO.Path]::GetFullPath($OutputDirectory)
New-Item -ItemType Directory -Path $taskOutput -Force|Out-Null
Add-Type -AssemblyName System.Windows.Forms,System.Drawing,UIAutomationClient,UIAutomationTypes
Add-Type -TypeDefinition @'
using System;
using System.Text;
using System.Runtime.InteropServices;
public static class NativeCaption {
 [StructLayout(LayoutKind.Sequential)] public struct Rect {public int Left,Top,Right,Bottom;}
 [StructLayout(LayoutKind.Sequential)] public struct Point {public int X,Y;}
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr w,out Rect r);
 [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr w,out Rect r);
 [DllImport("user32.dll")] public static extern bool GetCursorPos(out Point p);
 [DllImport("user32.dll")] public static extern bool SetCursorPos(int x,int y);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 [DllImport("user32.dll")] public static extern IntPtr SendMessageTimeoutW(IntPtr w,uint m,IntPtr wp,IntPtr lp,uint flags,uint timeout,out IntPtr result);
 [DllImport("user32.dll")] public static extern IntPtr GetMenu(IntPtr w);
 [DllImport("user32.dll")] public static extern IntPtr GetSystemMenu(IntPtr w,bool reset);
 [DllImport("user32.dll")] public static extern int GetMenuItemCount(IntPtr m);
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr w);
 [DllImport("user32.dll")] public static extern bool IsZoomed(IntPtr w);
 [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr w);
 [DllImport("user32.dll")] public static extern bool IsWindowVisible(IntPtr w);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr w,int n);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr w);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr w,out uint p);
 [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint first,uint second,bool attach);
 [DllImport("user32.dll")] public static extern bool BringWindowToTop(IntPtr w);
 public static bool Activate(IntPtr w){
  uint ignored,current=GetCurrentThreadId(),foreground=GetWindowThreadProcessId(GetForegroundWindow(),out ignored),target=GetWindowThreadProcessId(w,out ignored);
  bool a=foreground!=0&&foreground!=current&&AttachThreadInput(current,foreground,true);
  bool b=target!=0&&target!=current&&target!=foreground&&AttachThreadInput(current,target,true);
  try{BringWindowToTop(w);return SetForegroundWindow(w);}
  finally{if(b)AttachThreadInput(current,target,false);if(a)AttachThreadInput(current,foreground,false);}
 }
 [DllImport("user32.dll")] public static extern IntPtr SendMessageW(IntPtr w,uint m,IntPtr wp,IntPtr lp);
 [DllImport("user32.dll")] public static extern bool PostMessageW(IntPtr w,uint m,IntPtr wp,IntPtr lp);
 [DllImport("user32.dll")] public static extern void keybd_event(byte key,byte scan,uint flags,UIntPtr extra);
 [DllImport("user32.dll")] public static extern void mouse_event(uint flags,uint x,uint y,uint data,UIntPtr extra);
 [DllImport("user32.dll",CharSet=CharSet.Unicode)] public static extern int GetWindowTextW(IntPtr w,StringBuilder b,int length);
 [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr w,uint a,out Rect r,int size);
 [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr w,uint a,out int n,int size);
 public static Rect Bounds(IntPtr w){Rect r;GetWindowRect(w,out r);return r;}
 public static Rect Buttons(IntPtr w){Rect r;DwmGetWindowAttribute(w,5,out r,16);return r;}
 public static int Attribute(IntPtr w,uint a){int n;return DwmGetWindowAttribute(w,a,out n,4)>=0?n:-1;}
 public static string Title(IntPtr w){var b=new StringBuilder(256);GetWindowTextW(w,b,256);return b.ToString();}
 public static int Hit(IntPtr w,int x,int y){return (int)SendMessageW(w,0x84,IntPtr.Zero,new IntPtr((y<<16)|(x&65535)));}
 public static void Key(byte key,bool up=false){keybd_event(key,0,up?2u:0u,UIntPtr.Zero);}
 public static void Click(int x,int y){SetCursorPos(x,y);mouse_event(2,0,0,0,UIntPtr.Zero);mouse_event(4,0,0,0,UIntPtr.Zero);}
 public static void Drag(int x,int y,int dx,int dy){
  SetCursorPos(x,y);mouse_event(2,0,0,0,UIntPtr.Zero);
  for(int i=1;i<=10;i++){SetCursorPos(x+dx*i/10,y+dy*i/10);System.Threading.Thread.Sleep(15);}
  mouse_event(4,0,0,0,UIntPtr.Zero);
 }
 [StructLayout(LayoutKind.Sequential)] public struct TitleBarInfo {
  public uint Size; public Rect Bar;
  [MarshalAs(UnmanagedType.ByValArray,SizeConst=6)] public uint[] States;
  [MarshalAs(UnmanagedType.ByValArray,SizeConst=6)] public Rect[] Rectangles;
 }
 public static TitleBarInfo TitleInfo(IntPtr w){
  var t=new TitleBarInfo{Size=(uint)Marshal.SizeOf<TitleBarInfo>(),States=new uint[6],Rectangles=new Rect[6]};
  var p=Marshal.AllocHGlobal((int)t.Size);
  try{Marshal.StructureToPtr(t,p,false);SendMessageW(w,0x33f,IntPtr.Zero,p);return Marshal.PtrToStructure<TitleBarInfo>(p);}
  finally{Marshal.FreeHGlobal(p);}
 }
}
'@
# DWM and UI Automation return physical pixels. Disable caller virtualization so
# the caption/hit tests, mouse and CopyFromScreen share that coordinate space.
$taskDpiContext=[NativeCaption]::SetThreadDpiAwarenessContext([IntPtr](-4))
$taskIdentity=[Security.Principal.WindowsIdentity]::GetCurrent()
$taskElevated=[Security.Principal.WindowsPrincipal]::new($taskIdentity).IsInRole([Security.Principal.WindowsBuiltInRole]::Administrator)
$taskIdentity.Dispose()
if(($Mode -eq 'administrator') -ne $taskElevated){throw 'Native test permission mismatch'}
$taskPrevious=[NativeCaption]::GetForegroundWindow()
$taskMouse=[NativeCaption+Point]::new();[void][NativeCaption]::GetCursorPos([ref]$taskMouse)
$taskSummary=[ordered]@{mode=$Mode;elevated=$taskElevated;sourceCommit=(& git rev-parse HEAD);exeSha256=(Get-FileHash -LiteralPath $taskExe).Hash.ToLower();startedAt=(Get-Date).ToString('o');checks=@();result='fail'}
$taskProcess=$null
function Test-Native([string]$Name,[scriptblock]$Action){
 try{$taskDetail=& $Action;$taskSummary.checks+=@{name=$Name;result='pass';detail=$taskDetail}}
 catch{$taskSummary.checks+=@{name=$Name;result='fail';error=$_.Exception.Message}}
}
function Focus-Native {
 [void][NativeCaption]::ShowWindow($taskWindow,5)
 [void][NativeCaption]::ShowWindow($taskWindow,9)
 $taskShell=New-Object -ComObject WScript.Shell
 try{
  for($taskAttempt=0;$taskAttempt -lt 4;$taskAttempt++){
   [void]$taskShell.AppActivate($taskProcess.Id);[void][NativeCaption]::Activate($taskWindow)
   Start-Sleep -Milliseconds 200
   if([NativeCaption]::GetForegroundWindow() -eq $taskWindow){return}
  }
 }
 finally{[void][Runtime.InteropServices.Marshal]::ReleaseComObject($taskShell)}
 Start-Sleep -Milliseconds 300
 if([NativeCaption]::GetForegroundWindow() -ne $taskWindow){throw 'Native candidate is not foreground'}
}
function Save-NativeScreen([string]$Name){
 $taskRect=[NativeCaption]::Bounds($taskWindow)
 $taskDesktop=[Windows.Forms.SystemInformation]::VirtualScreen
 $taskClip=[Drawing.Rectangle]::Intersect($taskDesktop,[Drawing.Rectangle]::FromLTRB($taskRect.Left,$taskRect.Top,$taskRect.Right,$taskRect.Bottom))
 if($taskClip.Width -lt 1 -or $taskClip.Height -lt 1){throw 'Native window outside captureable desktop'}
 $taskBitmap=[Drawing.Bitmap]::new($taskClip.Width,$taskClip.Height)
 $taskGraphics=[Drawing.Graphics]::FromImage($taskBitmap)
 try{$taskGraphics.CopyFromScreen($taskClip.Location,[Drawing.Point]::Empty,$taskClip.Size);$taskBitmap.Save((Join-Path $taskOutput ($Name+'.png')),[Drawing.Imaging.ImageFormat]::Png)}
 finally{$taskGraphics.Dispose();$taskBitmap.Dispose()}
 @{window=$taskRect;capture=$taskClip}|ConvertTo-Json -Depth 3|Set-Content -LiteralPath (Join-Path $taskOutput ($Name+'-bounds.json')) -Encoding utf8
}
function Invoke-NativePage([string]$Phase){
 & $taskNode (Join-Path $PSScriptRoot 'windows-appearance.mjs') "http://127.0.0.1:$taskPort" $taskOutput $Phase 1 *> (Join-Path $taskOutput ($Phase+'.log'))
 if($LASTEXITCODE -ne 0){throw "Native page probe failed: $Phase"}
}
try{
 $taskSocket=[Net.Sockets.TcpListener]::new([Net.IPAddress]::Loopback,0);$taskSocket.Start();$taskPort=$taskSocket.LocalEndpoint.Port;$taskSocket.Stop()
 $taskSandbox=Join-Path $taskOutput ('sandbox-'+[guid]::NewGuid().ToString('N'))
 $taskInfo=[Diagnostics.ProcessStartInfo]::new();$taskInfo.FileName=$taskExe;$taskInfo.UseShellExecute=$false
 $taskInfo.WorkingDirectory=Split-Path -Parent $taskExe
 $taskInfo.Environment['OPENCODEX_SANDBOX']='1';$taskInfo.Environment['OPENCODEX_SANDBOX_ROOT']=$taskSandbox
 $taskInfo.Environment['OPENCODEX_WINDOWS_SMOKE_CDP_PORT']=[string]$taskPort
 # Native-caption acceptance uses the monitor's real scale. Device-scale forcing
 # belongs only to the separate WebView quality matrix, never native DPI claims.
 foreach($taskVariable in @('HOME','OPENCODEX_WINDOWS_SMOKE_SCALE','WEBVIEW2_ADDITIONAL_BROWSER_ARGUMENTS','WEBVIEW2_USER_DATA_FOLDER')){[void]$taskInfo.Environment.Remove($taskVariable)}
 $taskProcess=[Diagnostics.Process]::Start($taskInfo);$taskSummary.pid=$taskProcess.Id;$taskSummary.port=$taskPort
 Start-Sleep -Seconds 2;$taskProcess.Refresh();if($taskProcess.HasExited){throw 'Candidate exited before native checks'}
 $taskWindow=$taskProcess.MainWindowHandle;if(!$taskWindow){throw 'Native HWND missing'}
 Focus-Native
 Test-Native 'first-screen-source-identity' {
  $taskLog=Get-Content -LiteralPath (Join-Path $taskSandbox 'logs/app.log') -Raw
  if(!$taskLog.Contains("commit=$($taskSummary.sourceCommit)")){throw 'Native EXE provenance mismatch'}
  & $taskNode (Join-Path $PSScriptRoot 'windows-webview.mjs') "http://127.0.0.1:$taskPort" $taskOutput *> (Join-Path $taskOutput 'firstscreen.log')
  if($LASTEXITCODE -ne 0){throw 'First-screen bridge assertions failed'}
 }
 Test-Native 'single-system-caption-no-application-menu' {
  if([NativeCaption]::GetMenu($taskWindow) -ne [IntPtr]::Zero){throw 'Windows application menu row still attached'}
  if([NativeCaption]::Title($taskWindow) -ne 'OpenCodeX Desktop'){throw 'Caption title missing'}
  $taskSystemMenu=[NativeCaption]::GetSystemMenu($taskWindow,$false)
  if([NativeCaption]::GetMenuItemCount($taskSystemMenu) -lt 6){throw 'System menu missing'}
  @{title=[NativeCaption]::Title($taskWindow);nativeDpi=[NativeCaption]::GetDpiForWindow($taskWindow);systemMenuItems=[NativeCaption]::GetMenuItemCount($taskSystemMenu);screens=@([Windows.Forms.Screen]::AllScreens|ForEach-Object{@{bounds=$_.Bounds;primary=$_.Primary}})}
 }
 Test-Native 'native-theme-and-dwm-material' {
  Invoke-NativePage native-theme
  $taskTheme=Get-Content -LiteralPath (Join-Path $taskOutput 'native-theme.json') -Raw|ConvertFrom-Json
  foreach($taskCase in $taskTheme.cases){if($taskCase.native.reason -eq 'mica' -and $taskCase.native.material -ne 'mica'){throw 'DWM Mica was not applied'}}
  Invoke-NativePage native-light;Save-NativeScreen native-light
  Invoke-NativePage native-dark
  Save-NativeScreen native-dark
  @{darkMode=[NativeCaption]::Attribute($taskWindow,20);backdrop=[NativeCaption]::Attribute($taskWindow,38)}
 }
 Test-Native 'inactive-solid-and-foreground-mica' {
  [void][NativeCaption]::ShowWindow($taskWindow,6);Start-Sleep -Milliseconds 300
  Invoke-NativePage native-state
  $taskBackground=Get-Content -LiteralPath (Join-Path $taskOutput 'native-state.json') -Raw|ConvertFrom-Json
  Copy-Item -LiteralPath (Join-Path $taskOutput 'native-state.json') -Destination (Join-Path $taskOutput 'inactive-state.json')
  if($taskBackground.cases[0].native.material -ne 'solid' -or $taskBackground.cases[0].native.active){throw 'Inactive native material did not fall back'}
  [void][NativeCaption]::ShowWindow($taskWindow,9);Focus-Native;Invoke-NativePage native-state
  $taskForeground=Get-Content -LiteralPath (Join-Path $taskOutput 'native-state.json') -Raw|ConvertFrom-Json
  Copy-Item -LiteralPath (Join-Path $taskOutput 'native-state.json') -Destination (Join-Path $taskOutput 'foreground-state.json')
  if(!$taskForeground.cases[0].native.active){throw 'Native foreground was not restored'}
 }
 Test-Native 'system-transparency-solid-fallback-and-restoration' {
  $taskPersonalize='HKCU:\Software\Microsoft\Windows\CurrentVersion\Themes\Personalize'
  $taskInitial=(Get-ItemProperty -LiteralPath $taskPersonalize).EnableTransparency
  $taskBroadcastResult=[IntPtr]::Zero
  try{
   Set-ItemProperty -LiteralPath $taskPersonalize -Name EnableTransparency -Value 0
   [void][NativeCaption]::SendMessageTimeoutW($taskWindow,0x1a,[IntPtr]::Zero,[IntPtr]::Zero,2,1000,[ref]$taskBroadcastResult)
   Start-Sleep -Milliseconds 300;Invoke-NativePage native-state
   $taskSolid=Get-Content -LiteralPath (Join-Path $taskOutput 'native-state.json') -Raw|ConvertFrom-Json
   Copy-Item -LiteralPath (Join-Path $taskOutput 'native-state.json') -Destination (Join-Path $taskOutput 'transparency-off.json')
   if($taskSolid.cases[0].native.material -ne 'solid' -or $taskSolid.cases[0].native.transparency){throw 'System transparency-off native fallback failed'}
   if($taskSolid.cases[0].effects -ne 'high'){throw 'Native transparency changed user quality'}
   Save-NativeScreen transparency-off
  }finally{
   Set-ItemProperty -LiteralPath $taskPersonalize -Name EnableTransparency -Value $taskInitial
   [void][NativeCaption]::SendMessageTimeoutW($taskWindow,0x1a,[IntPtr]::Zero,[IntPtr]::Zero,2,1000,[ref]$taskBroadcastResult)
   Start-Sleep -Milliseconds 300
  }
  Invoke-NativePage native-state
  Copy-Item -LiteralPath (Join-Path $taskOutput 'native-state.json') -Destination (Join-Path $taskOutput 'transparency-restored.json')
  if((Get-ItemProperty -LiteralPath $taskPersonalize).EnableTransparency -ne $taskInitial){throw 'System transparency was not restored'}
 }
 Test-Native 'caption-hit-targets-and-accessibility' {
  Focus-Native
  $taskRect=[NativeCaption]::Bounds($taskWindow);$taskButtons=[NativeCaption]::Buttons($taskWindow)
  $taskTitle=[NativeCaption]::TitleInfo($taskWindow)
  $taskTitle|ConvertTo-Json -Depth 5|Set-Content -LiteralPath (Join-Path $taskOutput 'titlebar-accessibility.json') -Encoding utf8
  $taskHits=@();foreach($taskIndex in @(2,3,5)){$taskButton=$taskTitle.Rectangles[$taskIndex];$taskX=[int](($taskButton.Left+$taskButton.Right)/2);$taskY=[int](($taskButton.Top+$taskButton.Bottom)/2);$taskHits+=[NativeCaption]::Hit($taskWindow,$taskX,$taskY)}
  # DWM can report HTCAPTION for its minimize zone to a direct WM_NCHITTEST
  # probe; the separate real mouse-click/minimize assertion is authoritative.
  if($taskHits[0] -notin @(2,8) -or $taskHits[1] -ne 9 -or $taskHits[2] -ne 20){throw "Native caption hit targets differ: $taskHits"}
  $taskAutomation=[Windows.Automation.AutomationElement]::FromHandle($taskWindow)
  $taskControls=$taskAutomation.FindAll([Windows.Automation.TreeScope]::Descendants,[Windows.Automation.PropertyCondition]::new([Windows.Automation.AutomationElement]::ControlTypeProperty,[Windows.Automation.ControlType]::Button))
  $taskNames=@($taskControls|ForEach-Object{$_.Current.Name})
  # TITLEBARINFOEX is the native MSAA accessibility contract even when UIA's
  # WebView provider hides non-client descendants from this enumeration.
  if($taskTitle.States[5] -band 0x8000){throw 'Native Close is accessibility-invisible'}
  @{buttons=$taskButtons;hitTargets=$taskHits;buttonNames=$taskNames;titlebar=$taskTitle}
 }
 Test-Native 'double-click-maximize-restore' {
  Focus-Native
  $taskRect=[NativeCaption]::Bounds($taskWindow);$taskDpi=[NativeCaption]::GetDpiForWindow($taskWindow);$taskY=$taskRect.Top+[int](14*$taskDpi/96)
  [NativeCaption]::Click($taskRect.Left+($taskRect.Right-$taskRect.Left)/2,$taskY);Start-Sleep -Milliseconds 80
  [NativeCaption]::Click($taskRect.Left+($taskRect.Right-$taskRect.Left)/2,$taskY);Start-Sleep -Milliseconds 500
  if(![NativeCaption]::IsZoomed($taskWindow)){throw 'Double click did not maximize'}
  [void][NativeCaption]::ShowWindow($taskWindow,9);Start-Sleep -Milliseconds 300
  if([NativeCaption]::IsZoomed($taskWindow)){throw 'Native restore failed'}
 }
 Test-Native 'system-button-minimize-and-restore' {
  Focus-Native
  $taskButton=[NativeCaption]::TitleInfo($taskWindow).Rectangles[2]
  $taskX=[int](($taskButton.Left+$taskButton.Right)/2);$taskY=[int](($taskButton.Top+$taskButton.Bottom)/2)
  [NativeCaption]::Click($taskX,$taskY);Start-Sleep -Milliseconds 350
  if(![NativeCaption]::IsIconic($taskWindow)){throw 'System minimize button did not minimize'}
  [void][NativeCaption]::ShowWindow($taskWindow,9);Focus-Native
 }
 Test-Native 'native-caption-drag-and-edge-resize' {
  Focus-Native
  $taskRect=[NativeCaption]::Bounds($taskWindow);$taskDpi=[NativeCaption]::GetDpiForWindow($taskWindow)
  $taskY=$taskRect.Top+[int](14*$taskDpi/96);$taskX=$taskRect.Left+[int](($taskRect.Right-$taskRect.Left)/2)
  if([NativeCaption]::Hit($taskWindow,$taskX,$taskY) -ne 2){throw 'Blank caption is not HTCAPTION'}
  [NativeCaption]::Drag($taskX,$taskY,60,40);Start-Sleep -Milliseconds 300
  $taskMoved=[NativeCaption]::Bounds($taskWindow)
  if([math]::Abs($taskMoved.Left-$taskRect.Left) -lt 30){throw 'Caption did not move the native window'}
  [NativeCaption]::Drag($taskMoved.Left+($taskMoved.Right-$taskMoved.Left)/2,$taskMoved.Top+[int](14*$taskDpi/96),-60,-40)
  Start-Sleep -Milliseconds 250;$taskRect=[NativeCaption]::Bounds($taskWindow)
  $taskY=$taskRect.Top+[int](($taskRect.Bottom-$taskRect.Top)/2)
  $taskHit=[NativeCaption]::Hit($taskWindow,$taskRect.Right-2,$taskY)
  if($taskHit -ne 11){throw "Native right resize edge missing: $taskHit"}
  [NativeCaption]::Drag($taskRect.Right-2,$taskY,-60,0);Start-Sleep -Milliseconds 300
  $taskResized=[NativeCaption]::Bounds($taskWindow)
  if([math]::Abs(($taskResized.Right-$taskResized.Left)-($taskRect.Right-$taskRect.Left)) -lt 30){throw 'Native edge resize did not change width'}
  [NativeCaption]::Drag($taskResized.Right-2,$taskY,60,0);Start-Sleep -Milliseconds 200
  @{captionHit=2;resizeHit=$taskHit;original=$taskRect;moved=$taskMoved;resized=$taskResized}
 }
 Test-Native 'native-system-menu-alt-space' {
  Focus-Native
  [NativeCaption]::Key(0x12);[NativeCaption]::Key(0x20);[NativeCaption]::Key(0x20,$true);[NativeCaption]::Key(0x12,$true)
  Start-Sleep -Milliseconds 400;Save-NativeScreen system-menu
  $taskMenu=[NativeCaption]::GetSystemMenu($taskWindow,$false)
  if([NativeCaption]::GetMenuItemCount($taskMenu) -lt 6){throw 'Alt+Space system menu unavailable'}
  [NativeCaption]::Key(0x1b);[NativeCaption]::Key(0x1b,$true)
  [void][NativeCaption]::SendMessageW($taskWindow,0x1f,[IntPtr]::Zero,[IntPtr]::Zero)
  $taskBounds=[NativeCaption]::Bounds($taskWindow)
  [NativeCaption]::Click($taskBounds.Left+700,$taskBounds.Top+300)
  Start-Sleep -Milliseconds 200
  @{items=[NativeCaption]::GetMenuItemCount($taskMenu)}
 }
 Test-Native 'standard-editing-shortcuts-without-application-menu' {
  Focus-Native
  $taskClipboard=[Windows.Forms.Clipboard]::GetDataObject()
  try{Invoke-NativePage native-editing}
  finally{
   if($taskClipboard){[Windows.Forms.Clipboard]::SetDataObject($taskClipboard,$true)}
   else{[Windows.Forms.Clipboard]::Clear()}
  }
 }
 Test-Native 'snap-hover-and-win-z' {
  Focus-Native
  $taskButton=[NativeCaption]::TitleInfo($taskWindow).Rectangles[3]
  [void][NativeCaption]::SetCursorPos([int](($taskButton.Left+$taskButton.Right)/2),[int](($taskButton.Top+$taskButton.Bottom)/2))
  Start-Sleep -Milliseconds 1400;Save-NativeScreen snap-hover
  [NativeCaption]::Key(0x1b);[NativeCaption]::Key(0x1b,$true)
  [void][NativeCaption]::SetCursorPos($taskButton.Left-400,$taskButton.Bottom+300)
  Start-Sleep -Milliseconds 200
  [NativeCaption]::Key(0x5b);[NativeCaption]::Key(0x5a);[NativeCaption]::Key(0x5a,$true);[NativeCaption]::Key(0x5b,$true)
  Start-Sleep -Milliseconds 700;Save-NativeScreen snap-keyboard
  # Server/remote sessions may lack the shell feature. Preserve this gate as
  # unresolved rather than substituting DOM maximize or HTMAXBUTTON evidence.
  $taskRoot=[Windows.Automation.AutomationElement]::RootElement
  $taskWindows=$taskRoot.FindAll([Windows.Automation.TreeScope]::Children,[Windows.Automation.Condition]::TrueCondition)
  $taskShellWindows=@($taskWindows|Where-Object {$_.Current.ClassName -match 'Xaml|CoreWindow|Multitask|Shell|Popup'}|ForEach-Object{@{name=$_.Current.Name;class=$_.Current.ClassName;id=$_.Current.AutomationId;process=$_.Current.ProcessId;rectangle=$_.Current.BoundingRectangle}})
  $taskShellWindows|ConvertTo-Json -Depth 4|Set-Content -LiteralPath (Join-Path $taskOutput 'snap-shell-windows.json') -Encoding utf8
  $taskSnap=@($taskShellWindows|Where-Object {$_.name -match 'Snap|贴靠|贴齐|布局'})
  [NativeCaption]::Key(0x1b);[NativeCaption]::Key(0x1b,$true)
  if(!$taskSnap.Count){throw 'Snap layout popup not independently detected; native Win11 acceptance remains open'}
  @{popupNames=$taskSnap}
 }
 Test-Native 'close-preference-keeps-shell-and-tray' {
  Focus-Native;Invoke-NativePage close-tray
  [NativeCaption]::Key(0x12);[NativeCaption]::Key(0x73);[NativeCaption]::Key(0x73,$true);[NativeCaption]::Key(0x12,$true)
  Start-Sleep -Milliseconds 500;$taskProcess.Refresh()
  if($taskProcess.HasExited -or [NativeCaption]::IsWindowVisible($taskWindow)){throw 'Alt+F4 bypassed keep-on-close'}
  [void][NativeCaption]::ShowWindow($taskWindow,9);Focus-Native
  Invoke-NativePage native-state
  @{pidPreserved=$taskProcess.Id;restored=[NativeCaption]::IsWindowVisible($taskWindow)}
 }
 Test-Native 'close-preference-exits-through-close-request' {
  Focus-Native;Invoke-NativePage close-exit
  [void][NativeCaption]::PostMessageW($taskWindow,0x10,[IntPtr]::Zero,[IntPtr]::Zero)
  if(!$taskProcess.WaitForExit(8000)){throw 'Close with keep disabled did not exit'}
  @{exitCode=$taskProcess.ExitCode}
 }
 $taskSummary.result=if(@($taskSummary.checks|Where-Object {$_.result -eq 'fail'}).Count){'fail'}else{'pass'}
}catch{$taskSummary.error=$_.Exception.Message}
finally{
 if($taskProcess){if(!$taskProcess.HasExited){$taskProcess.Kill($true);[void]$taskProcess.WaitForExit(5000)};$taskProcess.Dispose()}
 if($taskSandbox -and (Test-Path -LiteralPath (Join-Path $taskSandbox 'logs/app.log'))){Copy-Item -LiteralPath (Join-Path $taskSandbox 'logs/app.log') -Destination (Join-Path $taskOutput 'app.log')}
 [void][NativeCaption]::SetCursorPos($taskMouse.X,$taskMouse.Y);[void][NativeCaption]::SetForegroundWindow($taskPrevious)
 if($taskDpiContext){[void][NativeCaption]::SetThreadDpiAwarenessContext($taskDpiContext)}
 $taskSummary.finishedAt=(Get-Date).ToString('o');$taskSummary|ConvertTo-Json -Depth 7|Set-Content -LiteralPath (Join-Path $taskOutput 'native.json') -Encoding utf8
}
if($taskSummary.result -ne 'pass'){throw 'Native window gates incomplete; see native.json'}
