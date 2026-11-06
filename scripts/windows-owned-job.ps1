# Browser descendants inherit a private kill-on-close job, not a PID inventory.
$ErrorActionPreference = 'Stop'
try {
  $plan = $env:XTERM_OWNED_JOB_COMMAND | ConvertFrom-Json
  if (-not $plan -or -not $plan.executable -or -not $plan.cwd) { throw 'Missing owned command plan' }
  Remove-Item Env:XTERM_OWNED_JOB_COMMAND
  Add-Type -Path (Join-Path $PSScriptRoot 'windows-owned-job.cs')
  # Native Process.Start avoids Windows PowerShell's legacy argument rewriting.
  $arguments = @($plan.arguments | ForEach-Object { [string]$_ })
  [XtermOwnedJob]::Run([string]$plan.executable, [string[]]$arguments, [string]$plan.cwd)
} catch {
  [Console]::Error.WriteLine($_.Exception.Message)
  [Environment]::Exit(1)
}
