<# Nexa Support — Run All Tests #>

$ErrorActionPreference = "Stop"

Write-Host "`n=== Backend Tests ===" -ForegroundColor Cyan
Push-Location "$PSScriptRoot\..\backend"

if (Test-Path "venv\Scripts\Activate.ps1") {
    & .\venv\Scripts\Activate.ps1
}

python -m pytest tests/ -v --tb=short
$backendExit = $LASTEXITCODE
Pop-Location

Write-Host "`n=== Frontend Build Check ===" -ForegroundColor Cyan
Push-Location "$PSScriptRoot\..\frontend"
cargo check 2>&1
$frontendExit = $LASTEXITCODE
Pop-Location

if ($backendExit -eq 0 -and $frontendExit -eq 0) {
    Write-Host "`n=== All checks passed ===" -ForegroundColor Green
} else {
    Write-Host "`n=== Some checks failed ===" -ForegroundColor Red
    exit 1
}
