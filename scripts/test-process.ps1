param(
    [Parameter(Mandatory=$true)][int]$ProcessId,
    [Parameter(Mandatory=$true)][string]$ExpectedExecutable,
    [Parameter(Mandatory=$true)][ValidateSet('alive','close','stop','cleanup-image','window','resize')][string]$Action,
    [int]$Width=0,
    [int]$Height=0
)
$ErrorActionPreference = 'Stop'
if (-not [IO.Path]::IsPathRooted($ExpectedExecutable)) { throw 'An exact absolute test image is required' }
$taskProcesses = if ($Action -eq 'cleanup-image') {
    if ($ProcessId -ne 0) { throw 'Image cleanup uses ProcessId 0' }
    [Diagnostics.Process]::GetProcessesByName([IO.Path]::GetFileNameWithoutExtension($ExpectedExecutable))
} else {
    [Diagnostics.Process]::GetProcessById($ProcessId)
}
foreach ($taskProcess in $taskProcesses) {
    try {
        try {
            $null = $taskProcess.Handle
            $taskImage = $taskProcess.MainModule.FileName
        } catch {
            if ($Action -eq 'cleanup-image') { continue } # Unknown or exited images are never terminated.
            throw
        }
        if (-not [String]::Equals($taskImage, $ExpectedExecutable, [StringComparison]::OrdinalIgnoreCase)) {
            if ($Action -eq 'cleanup-image') { continue }
            throw 'Test process image mismatch; operation refused'
        }
        switch ($Action) {
            { $_ -in @('window','resize') } {
                if (-not ('MpvKitTestWindow' -as [type])) {
                    Add-Type @'
using System;
using System.Runtime.InteropServices;
public static class MpvKitTestWindow {
    [StructLayout(LayoutKind.Sequential)] public struct Rect { public int Left, Top, Right, Bottom; }
    [StructLayout(LayoutKind.Sequential)] public struct MonitorInfo { public int Size; public Rect Monitor, Work; public uint Flags; }
    [DllImport("user32.dll")] public static extern IntPtr SetThreadDpiAwarenessContext(IntPtr value);
    [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] public static extern bool GetClientRect(IntPtr window, out Rect rect);
    [DllImport("user32.dll")] public static extern IntPtr MonitorFromWindow(IntPtr window, uint flags);
    [DllImport("user32.dll")] public static extern bool GetMonitorInfo(IntPtr monitor, ref MonitorInfo info);
    [DllImport("user32.dll")] public static extern bool SetWindowPos(IntPtr window, IntPtr after, int x, int y, int width, int height, uint flags);
    [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr window, int attribute, out Rect rect, int size);
}
'@
                }
                $null = [MpvKitTestWindow]::SetThreadDpiAwarenessContext([IntPtr]::new(-4))
                $taskWindow = $taskProcess.MainWindowHandle
                if ($taskWindow -eq [IntPtr]::Zero) { throw 'Test player window is not ready' }
                if ($Action -eq 'resize') {
                    if ($Width -lt 200 -or $Width -gt 16000 -or $Height -lt 200 -or $Height -gt 16000) { throw 'Invalid test window size' }
                    if (-not [MpvKitTestWindow]::SetWindowPos($taskWindow, [IntPtr]::Zero, 0, 0, $Width, $Height, 22)) { throw 'Test resize failed' }
                    Start-Sleep -Milliseconds 200
                }
                $taskOuter = [MpvKitTestWindow+Rect]::new()
                $taskClient = [MpvKitTestWindow+Rect]::new()
                $taskFrame = [MpvKitTestWindow+Rect]::new()
                $taskMonitor = [MpvKitTestWindow+MonitorInfo]::new()
                $taskMonitor.Size = [Runtime.InteropServices.Marshal]::SizeOf($taskMonitor)
                if (-not [MpvKitTestWindow]::GetWindowRect($taskWindow,[ref]$taskOuter) -or
                    -not [MpvKitTestWindow]::GetClientRect($taskWindow,[ref]$taskClient) -or
                    -not [MpvKitTestWindow]::GetMonitorInfo([MpvKitTestWindow]::MonitorFromWindow($taskWindow,2),[ref]$taskMonitor)) { throw 'Test window query failed' }
                if ([MpvKitTestWindow]::DwmGetWindowAttribute($taskWindow,9,[ref]$taskFrame,16) -ne 0) { $taskFrame = $taskOuter }
                @{outer=$taskOuter;frame=$taskFrame;client=$taskClient;monitor=$taskMonitor.Monitor;work=$taskMonitor.Work} | ConvertTo-Json -Compress
            }
            'alive' { if ($taskProcess.HasExited) { throw 'Test process has exited' } }
            'close' {
                if (-not $taskProcess.CloseMainWindow()) { throw 'Test window could not close normally' }
                if (-not $taskProcess.WaitForExit(10000)) { throw 'Test window close timed out' }
            }
            { $_ -in @('stop','cleanup-image') } {
                if (-not $taskProcess.HasExited) { $taskProcess.Kill() }
                if (-not $taskProcess.WaitForExit(5000)) { throw 'Test process cleanup timed out' }
            }
        }
    } finally {
        $taskProcess.Dispose()
    }
}
