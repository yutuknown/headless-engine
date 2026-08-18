#!/usr/bin/env pwsh
# Reads .env and uploads all secrets to GitHub Actions repository secrets

$envFile = "$PSScriptRoot\..\.env"
if (-not (Test-Path $envFile)) {
    Write-Error ".env file not found. Create one from the template."
    exit 1
}

Get-Content $envFile | ForEach-Object {
    $line = $_.Trim()
    if ($line -eq "" -or $line.StartsWith("#")) { return }
    $parts = $line -split "=", 2
    if ($parts.Count -ne 2) { return }
    $key   = $parts[0].Trim()
    $value = $parts[1].Trim()
    if ($value -eq "") {
        Write-Warning "Skipping $key (empty value)"
        return
    }
    Write-Host "Setting secret: $key" -ForegroundColor Cyan
    gh secret set $key --body $value
}

Write-Host "`nAll secrets uploaded. Triggering publish workflow..." -ForegroundColor Green
gh workflow run publish.yml
