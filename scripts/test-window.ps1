param([int]$ProcessId, [string]$ExpectedExecutable, [long]$WindowHandle, [switch]$Activate, [switch]$Minimize, [string]$Screenshot)
$ErrorActionPreference = 'Stop'
$process = Get-Process -Id $ProcessId
if ([IO.Path]::GetFullPath($process.Path) -ine [IO.Path]::GetFullPath($ExpectedExecutable)) { throw 'Unexpected process path' }
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class MpvKitWindowProbe {
 [DllImport("kernel32.dll")] public static extern uint GetCurrentThreadId();
 [DllImport("user32.dll")] public static extern uint GetWindowThreadProcessId(IntPtr h,IntPtr pid);
 [DllImport("user32.dll",EntryPoint="GetWindowThreadProcessId")] public static extern uint WindowOwner(IntPtr h,out uint pid);
 [DllImport("user32.dll")] public static extern bool AttachThreadInput(uint a,uint b,bool attach);
 [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr h,int command);
 [DllImport("user32.dll")] public static extern bool IsIconic(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr context);
 [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr h);
 [DllImport("user32.dll")] public static extern IntPtr GetForegroundWindow();
 [DllImport("user32.dll")] public static extern uint GetDpiForWindow(IntPtr h);
 [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr h,int attr,out int value,int size);
 [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr h,out Rect rect);
 public struct Rect { public int Left,Top,Right,Bottom; }
}
"@
$null=[MpvKitWindowProbe]::SetThreadDpiAwarenessContext([IntPtr](-4))
$handle=if ($WindowHandle) { [IntPtr]$WindowHandle } else { $process.MainWindowHandle }
$owner=0
$null=[MpvKitWindowProbe]::WindowOwner($handle,[ref]$owner)
if ($owner -ne $ProcessId) { throw "Window does not belong to expected process" }
if ($handle -eq 0) { throw 'No application window' }
if ($Minimize) {
 [MpvKitWindowProbe]::ShowWindow($handle,6) | Out-Null
 Start-Sleep -Milliseconds 200
}
if ($Activate) {
 $thread=[MpvKitWindowProbe]::GetCurrentThreadId()
 $foregroundThread=[MpvKitWindowProbe]::GetWindowThreadProcessId([MpvKitWindowProbe]::GetForegroundWindow(),[IntPtr]::Zero)
 $attached=[MpvKitWindowProbe]::AttachThreadInput($thread,$foregroundThread,$true)
 try { [MpvKitWindowProbe]::ShowWindow($handle,9) | Out-Null; [MpvKitWindowProbe]::SetForegroundWindow($handle) | Out-Null }
 finally { if ($attached) { [MpvKitWindowProbe]::AttachThreadInput($thread,$foregroundThread,$false) | Out-Null } }
 Start-Sleep -Milliseconds 350
}
$backdrop=0
$result=[MpvKitWindowProbe]::DwmGetWindowAttribute($handle,38,[ref]$backdrop,4)
$rect=New-Object MpvKitWindowProbe+Rect
[MpvKitWindowProbe]::GetWindowRect($handle,[ref]$rect) | Out-Null
if ($Screenshot -and [MpvKitWindowProbe]::GetForegroundWindow() -eq $handle) {
 Add-Type -AssemblyName System.Drawing
 $bitmap=New-Object Drawing.Bitmap ($rect.Right-$rect.Left),($rect.Bottom-$rect.Top)
 $graphics=[Drawing.Graphics]::FromImage($bitmap)
 $graphics.CopyFromScreen($rect.Left,$rect.Top,0,0,$bitmap.Size)
 $bitmap.Save($Screenshot,[Drawing.Imaging.ImageFormat]::Png)
 $graphics.Dispose();$bitmap.Dispose()
}
@{handle=$handle.ToInt64();minimized=[MpvKitWindowProbe]::IsIconic($handle);foreground=([MpvKitWindowProbe]::GetForegroundWindow() -eq $handle);dpi=[MpvKitWindowProbe]::GetDpiForWindow($handle);backdrop=$backdrop;dwmResult=$result;width=($rect.Right-$rect.Left);height=($rect.Bottom-$rect.Top);os=[Environment]::OSVersion.VersionString} | ConvertTo-Json -Compress
