<# Nexa Support — Interactive Setup Script (Windows)
   Installs all dependencies for backend and frontend
   with an interactive terminal menu. #>

$ErrorActionPreference = "Stop"

# ── Colours & Helpers ────────────────────────────────────
function Write-Banner {
    Write-Host ""
    Write-Host "  ╔═══════════════════════════════════════╗" -ForegroundColor Cyan
    Write-Host "  ║         Nexa Support — Setup          ║" -ForegroundColor Cyan
    Write-Host "  ╚═══════════════════════════════════════╝" -ForegroundColor Cyan
    Write-Host ""
}

function Write-Step($num, $total, $msg) {
    Write-Host "  [$num/$total] " -ForegroundColor Yellow -NoNewline
    Write-Host $msg
}

function Write-Ok($msg)   { Write-Host "    ✓ $msg" -ForegroundColor Green }
function Write-Warn($msg) { Write-Host "    ! $msg" -ForegroundColor Yellow }
function Write-Err($msg)  { Write-Host "    ✗ $msg" -ForegroundColor Red }
function Write-Info($msg) { Write-Host "    $msg" -ForegroundColor DarkGray }

function Show-Menu {
    Write-Host ""
    Write-Host "  What would you like to do?" -ForegroundColor White
    Write-Host ""
    Write-Host "    [1] Full Setup       — Install everything (recommended)" -ForegroundColor Cyan
    Write-Host "    [2] Backend Only     — Python venv + dependencies" -ForegroundColor Cyan
    Write-Host "    [3] Frontend Only    — Rust wasm target + Dioxus CLI" -ForegroundColor Cyan
    Write-Host "    [4] Check Prereqs    — Verify installed tools" -ForegroundColor Cyan
    Write-Host "    [5] Configure LLM    — Set provider & API keys" -ForegroundColor Cyan
    Write-Host "    [0] Exit" -ForegroundColor DarkGray
    Write-Host ""
    $choice = Read-Host "  Enter choice"
    return $choice
}

# ── Prerequisite Check ───────────────────────────────────
function Test-Prerequisites {
    Write-Host ""
    Write-Host "  Checking prerequisites..." -ForegroundColor White
    $ok = $true

    # Python
    if (Get-Command python -ErrorAction SilentlyContinue) {
        $pyVer = python --version 2>&1
        Write-Ok "Python: $pyVer"
    } else {
        Write-Err "Python not found — install from https://python.org"
        $ok = $false
    }

    # Rust / Cargo
    if (Get-Command cargo -ErrorAction SilentlyContinue) {
        $cargoVer = cargo --version 2>&1
        Write-Ok "Cargo: $cargoVer"
    } else {
        Write-Err "Rust/Cargo not found — install from https://rustup.rs"
        $ok = $false
    }

    # Ollama (optional)
    if (Get-Command ollama -ErrorAction SilentlyContinue) {
        Write-Ok "Ollama: installed"
    } else {
        Write-Warn "Ollama not found — needed for local LLM (optional if using cloud)"
    }

    # dx
    if (Get-Command dx -ErrorAction SilentlyContinue) {
        Write-Ok "Dioxus CLI (dx): installed"
    } else {
        Write-Info "Dioxus CLI not yet installed (will be installed during setup)"
    }

    Write-Host ""
    return $ok
}

# ── Backend Setup ────────────────────────────────────────
function Install-Backend {
    param([int]$StepStart = 1, [int]$Total = 2)

    Write-Step $StepStart $Total "Setting up Python backend..."
    Push-Location backend

    if (-not (Test-Path ".env") -and (Test-Path ".env.example")) {
        Copy-Item ".env.example" ".env"
        Write-Ok "Created .env from .env.example"
    }

    if (-not (Test-Path "venv")) {
        python -m venv venv
        Write-Ok "Virtual environment created"
    } else {
        Write-Info "Virtual environment already exists"
    }

    & .\venv\Scripts\Activate.ps1
    pip install --upgrade pip --quiet 2>$null
    pip install -r requirements.txt --quiet 2>$null
    Write-Ok "Python dependencies installed"
    Pop-Location
}

# ── Frontend Setup ───────────────────────────────────────
function Install-Frontend {
    param([int]$StepStart = 1, [int]$Total = 2)

    Write-Step $StepStart $Total "Setting up Rust frontend..."

    if (-not (Get-Command cargo -ErrorAction SilentlyContinue)) {
        Write-Err "Rust/Cargo not found. Install from https://rustup.rs"
        return
    }
    Write-Ok "Rust toolchain found"

    rustup target add wasm32-unknown-unknown 2>$null
    Write-Ok "wasm32-unknown-unknown target ready"

    if (-not (Get-Command dx -ErrorAction SilentlyContinue)) {
        Write-Info "Installing Dioxus CLI (this may take a few minutes)..."
        cargo install dioxus-cli 2>$null
    }
    Write-Ok "Dioxus CLI (dx) ready"
}

# ── LLM Configuration ───────────────────────────────────
function Set-LLMConfig {
    Write-Host ""
    Write-Host "  Configure LLM Provider" -ForegroundColor White
    Write-Host ""
    Write-Host "    [1] Ollama (local)    — Free, runs on your machine" -ForegroundColor Cyan
    Write-Host "    [2] Cloud API         — OpenAI, Groq, Together, etc." -ForegroundColor Cyan
    Write-Host ""
    $provider = Read-Host "  Enter choice (1 or 2)"

    $envPath = "backend/.env"
    if (-not (Test-Path $envPath)) {
        Write-Warn "No .env file found — run Backend Setup first"
        return
    }

    $envContent = Get-Content $envPath -Raw

    if ($provider -eq "2") {
        $envContent = $envContent -replace 'LLM_PROVIDER=.*', 'LLM_PROVIDER=cloud'
        Write-Ok "Set provider to: cloud"

        $apiKey = Read-Host "  Enter your API key (or press Enter to skip)"
        if ($apiKey) {
            $envContent = $envContent -replace 'CLOUD_API_KEY=.*', "CLOUD_API_KEY=$apiKey"
            Write-Ok "API key saved to .env"
        }

        $baseUrl = Read-Host "  Base URL [https://api.openai.com/v1]"
        if (-not $baseUrl) { $baseUrl = "https://api.openai.com/v1" }
        $envContent = $envContent -replace 'CLOUD_BASE_URL=.*', "CLOUD_BASE_URL=$baseUrl"

        $cloudModel = Read-Host "  Model name [gpt-4]"
        if (-not $cloudModel) { $cloudModel = "gpt-4" }
        $envContent = $envContent -replace 'CLOUD_MODEL=.*', "CLOUD_MODEL=$cloudModel"
        Write-Ok "Cloud config saved"
    } else {
        $envContent = $envContent -replace 'LLM_PROVIDER=.*', 'LLM_PROVIDER=ollama'
        Write-Ok "Set provider to: ollama"

        $model = Read-Host "  Ollama model [mistral]"
        if (-not $model) { $model = "mistral" }
        $envContent = $envContent -replace 'OLLAMA_MODEL=.*', "OLLAMA_MODEL=$model"
        Write-Ok "Ollama model set to: $model"
    }

    $envContent | Set-Content $envPath -NoNewline
    Write-Ok "Configuration saved to backend/.env"
}

# ── Main Loop ────────────────────────────────────────────
Write-Banner

while ($true) {
    $choice = Show-Menu

    switch ($choice) {
        "1" {
            Write-Host ""
            $prereqs = Test-Prerequisites
            if (-not $prereqs) {
                Write-Warn "Some prerequisites are missing. Continue anyway? (y/n)"
                $cont = Read-Host "  "
                if ($cont -ne "y") { continue }
            }
            Install-Backend -StepStart 1 -Total 4
            Install-Frontend -StepStart 3 -Total 4
            Write-Host ""
            Write-Host "  ✓ Full setup complete!" -ForegroundColor Green
            Write-Host "    Run scripts\start_backend.ps1 and scripts\start_frontend.ps1 to start." -ForegroundColor DarkGray
        }
        "2" {
            Install-Backend -StepStart 1 -Total 1
            Write-Host ""
            Write-Ok "Backend setup complete!"
        }
        "3" {
            Install-Frontend -StepStart 1 -Total 1
            Write-Host ""
            Write-Ok "Frontend setup complete!"
        }
        "4" {
            Test-Prerequisites | Out-Null
        }
        "5" {
            Set-LLMConfig
        }
        "0" {
            Write-Host ""
            Write-Host "  Goodbye!" -ForegroundColor Cyan
            Write-Host ""
            break
        }
        default {
            Write-Warn "Invalid choice, try again."
        }
    }
}
