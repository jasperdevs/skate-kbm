$ErrorActionPreference = "Stop"

$root = Resolve-Path (Join-Path $PSScriptRoot "..")
$appDir = Join-Path $root "dist\app"
$zip = Join-Path $root "dist\skate-kbm-windows-x64.zip"

if (-not (Test-Path (Join-Path $appDir "skate-kbm.exe"))) {
  throw "Missing dist\app\skate-kbm.exe. Run npm run build first."
}

if (Test-Path $zip) {
  Remove-Item $zip
}

Compress-Archive -Path (Join-Path $appDir "*") -DestinationPath $zip
Write-Host "Created $zip"
