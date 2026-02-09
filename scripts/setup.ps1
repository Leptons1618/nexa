<# Nexa Support — Setup Script
   Installs all dependencies for backend and frontend. #>

$ErrorActionPreference = "Stop"

Write-Host "`n=== Nexa Support — Setup ===" -ForegroundColor Cyan

# ── Backend ─────────────────────────────────────────────
Write-Host "`n[1/4] Setting up Python backend..." -ForegroundColor Yellow
Push-Location backend

if (-not (Test-Path ".env")) {
    Copy-Item ".env.example" ".env"
    Write-Host "       Created .env from .env.example"
}

if (-not (Test-Path "venv")) {
    python -m venv venv
    Write-Host "       Virtual environment created"
}

& .\venv\Scripts\Activate.ps1
pip install --upgrade pip --quiet
pip install -r requirements.txt --quiet
Write-Host "       Python dependencies installed"
Pop-Location

# ── Frontend ────────────────────────────────────────────
Write-Host "`n[2/4] Checking Rust toolchain..." -ForegroundColor Yellow
if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
    Write-Host "       ERROR: Rust/Cargo not found. Install from https://rustup.rs" -ForegroundColor Red
    exit 1
}
Write-Host "       Rust toolchain OK"

Write-Host "`n[3/4] Adding wasm32 target..." -ForegroundColor Yellow
rustup target add wasm32-unknown-unknown 2>$null
Write-Host "       wasm32-unknown-unknown target ready"

Write-Host "`n[4/4] Installing Dioxus CLI..." -ForegroundColor Yellow
if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
    cargo install dioxus-cli
}
Write-Host "       Dioxus CLI (dx) ready"

Write-Host "`n=== Setup complete ===" -ForegroundColor Green
Write-Host "Run scripts/start_backend.ps1 and scripts/start_frontend.ps1 to start."
