<# Nexa Support — Start Frontend (Dioxus dev server) #>

$ErrorActionPreference = "Stop"
Push-Location "$PSScriptRoot\..\frontend"

Write-Host "Starting Nexa frontend (Dioxus) ..." -ForegroundColor Cyan
dx serve --platform web
Pop-Location
