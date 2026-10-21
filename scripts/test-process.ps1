param(
    [Parameter(Mandatory=$true)][int]$ProcessId,
    [Parameter(Mandatory=$true)][string]$ExpectedExecutable,
    [Parameter(Mandatory=$true)][ValidateSet('alive','close','stop','cleanup-image')][string]$Action
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
