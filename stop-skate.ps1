$ErrorActionPreference = 'Continue'

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$stateDir = Join-Path $root '.skate-kbm'
$pidFile = Join-Path $stateDir 'mapper.pid'

if (!(Test-Path -LiteralPath $pidFile)) {
  Write-Output 'No skate-kbm PID file found.'
  exit 0
}

$pidText = Get-Content -LiteralPath $pidFile -ErrorAction SilentlyContinue | Select-Object -First 1
if (!$pidText) {
  Remove-Item -LiteralPath $pidFile -Force -ErrorAction SilentlyContinue
  Write-Output 'No skate-kbm PID recorded.'
  exit 0
}

$process = Get-Process -Id $pidText -ErrorAction SilentlyContinue
if ($process) {
  Stop-Process -Id $pidText -Force
  Write-Output "Stopped skate-kbm PID $pidText"
} else {
  Write-Output "skate-kbm PID $pidText is not running."
}

Remove-Item -LiteralPath $pidFile -Force -ErrorAction SilentlyContinue
