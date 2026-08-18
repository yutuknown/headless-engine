# Headless Engine Windows Installer
$ErrorActionPreference = "Stop"

$repo = "yutuknown/headless-engine"
$asset = "headless-engine-windows-x86_64.zip"
$installDir = "$env:LOCALAPPDATA\Programs\headless-engine"

Write-Host ">>> Downloading latest Headless Engine Windows release..." -ForegroundColor Cyan
$downloadUrl = "https://github.com/$repo/releases/latest/download/$asset"
$tempZip = [System.IO.Path]::GetTempFileName() + ".zip"
$tempExtract = [System.IO.Path]::Combine([System.IO.Path]::GetTempPath(), [System.Guid]::NewGuid().ToString())

Invoke-WebRequest -Uri $downloadUrl -OutFile $tempZip

Write-Host ">>> Extracting to $installDir..." -ForegroundColor Cyan
Expand-Archive -Path $tempZip -DestinationPath $tempExtract -Force

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir -Force | Out-Null
}

Move-Item -Path "$tempExtract\headless-engine.exe" -Destination "$installDir\headless-engine.exe" -Force
Remove-Item -Path $tempZip -Force
Remove-Item -Path $tempExtract -Recurse -Force

# Add to user PATH if not present
$userPath = [Environment]::GetEnvironmentVariable("Path", "User")
if ($userPath -notlike "*$installDir*") {
    [Environment]::SetEnvironmentVariable("Path", "$userPath;$installDir", "User")
    $env:Path += ";$installDir"
    Write-Host ">>> Added $installDir to User PATH." -ForegroundColor Green
}

Write-Host ">>> Headless Engine installed successfully! Run 'headless-engine --help'" -ForegroundColor Green
& "$installDir\headless-engine.exe" --help
