$ErrorActionPreference = 'Stop'

$root = Split-Path -Parent $MyInvocation.MyCommand.Path
$mapper = Join-Path $root 'dist\app\mapper\skate-kbm-mapper.exe'
$stateDir = Join-Path $root '.skate-kbm'
$pidFile = Join-Path $stateDir 'mapper.pid'
$outLog = Join-Path $stateDir 'mapper.out.log'
$errLog = Join-Path $stateDir 'mapper.err.log'

if (!(Test-Path -LiteralPath $mapper)) {
  throw "Mapper binary missing: $mapper. Run npm run build first."
}

New-Item -ItemType Directory -Force -Path $stateDir | Out-Null

if (Test-Path -LiteralPath $pidFile) {
  $existingPid = Get-Content -LiteralPath $pidFile -ErrorAction SilentlyContinue | Select-Object -First 1
  if ($existingPid -and (Get-Process -Id $existingPid -ErrorAction SilentlyContinue)) {
    Write-Output "skate-kbm is already running as PID $existingPid"
    exit 0
  }
}

'' | Set-Content -LiteralPath $outLog
'' | Set-Content -LiteralPath $errLog

$process = Start-Process `
  -FilePath $mapper `
  -ArgumentList @('--cursor-lock', 'off') `
  -WorkingDirectory $root `
  -WindowStyle Hidden `
  -RedirectStandardOutput $outLog `
  -RedirectStandardError $errLog `
  -PassThru

$process.Id | Set-Content -LiteralPath $pidFile
Write-Output "Started skate-kbm safely as PID $($process.Id)"
Write-Output "Cursor lock is off. Stop with Ctrl+Alt+Backspace or .\stop-skate.ps1"
Write-Output "Logs: $outLog"
