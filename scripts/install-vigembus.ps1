$ErrorActionPreference = "Stop"

$release = Invoke-RestMethod `
  -Headers @{ "User-Agent" = "skate-kbm" } `
  -Uri "https://api.github.com/repos/nefarius/ViGEmBus/releases/latest"

$asset = $release.assets |
  Where-Object { $_.name -match "ViGEmBus.*\.exe$|ViGEmBus.*\.msi$" } |
  Select-Object -First 1

if (-not $asset) {
  throw "Could not find a ViGEmBus installer asset in the latest GitHub release."
}

$downloads = Join-Path $env:USERPROFILE "Downloads"
$out = Join-Path $downloads $asset.name

Write-Host "Downloading $($asset.name)..."
Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $out

Write-Host "Installer saved to $out"
Write-Host "Run it and accept the driver install prompts, then start skate-kbm again."
Start-Process $out
