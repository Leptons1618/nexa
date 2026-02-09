<# Nexa Support — Start Backend #>

$ErrorActionPreference = "Stop"
Push-Location "$PSScriptRoot\..\backend"

if (Test-Path "venv\Scripts\Activate.ps1") {
    & .\venv\Scripts\Activate.ps1
}

Write-Host "Starting Nexa backend on http://localhost:8000 ..." -ForegroundColor Cyan
python run.py
Pop-Location
