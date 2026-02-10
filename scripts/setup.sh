#!/usr/bin/env bash
# ─────────────────────────────────────────────────────────
#  Nexa Support — Interactive Setup Script (Linux / macOS)
# ─────────────────────────────────────────────────────────
set -e

# ── Colors ──────────────────────────────────────────────
RESET="\033[0m"
BOLD="\033[1m"
CYAN="\033[36m"
GREEN="\033[32m"
YELLOW="\033[33m"
RED="\033[31m"
DIM="\033[2m"

banner() {
    echo ""
    echo -e "${CYAN}${BOLD}╔═══════════════════════════════════════════════╗${RESET}"
    echo -e "${CYAN}${BOLD}║          Nexa Support — Setup                 ║${RESET}"
    echo -e "${CYAN}${BOLD}╚═══════════════════════════════════════════════╝${RESET}"
    echo ""
}

info()    { echo -e "  ${CYAN}[INFO]${RESET}  $1"; }
success() { echo -e "  ${GREEN}[  OK]${RESET}  $1"; }
warn()    { echo -e "  ${YELLOW}[WARN]${RESET}  $1"; }
fail()    { echo -e "  ${RED}[FAIL]${RESET}  $1"; }

check_cmd() {
    if command -v "$1" &>/dev/null; then
        success "$1 found: $(command -v "$1")"
        return 0
    else
        fail "$1 not found"
        return 1
    fi
}

# ── Prerequisite Check ──────────────────────────────────
check_prerequisites() {
    echo -e "\n${BOLD}Checking prerequisites...${RESET}\n"
    local ok=true

    check_cmd python3 || ok=false
    check_cmd pip3    || check_cmd pip || ok=false
    check_cmd cargo   || ok=false
    check_cmd rustup  || ok=false

    if command -v ollama &>/dev/null; then
        success "ollama found"
    else
        warn "ollama not found — needed for local LLM (install from https://ollama.ai)"
    fi

    if command -v dx &>/dev/null; then
        success "dioxus-cli (dx) found"
    else
        warn "dioxus-cli (dx) not found — will install if you choose frontend setup"
    fi

    echo ""
    if [ "$ok" = false ]; then
        fail "Some required tools are missing. Install them first."
        return 1
    fi
    success "All required tools present."
    return 0
}

# ── Backend Setup ───────────────────────────────────────
setup_backend() {
    echo -e "\n${BOLD}Setting up Python backend...${RESET}\n"
    cd "$(dirname "$0")/../backend" || exit 1

    if [ ! -f ".env" ] && [ -f ".env.example" ]; then
        cp .env.example .env
        info "Created .env from .env.example"
    fi

    if [ ! -d "venv" ]; then
        python3 -m venv venv
        info "Virtual environment created"
    fi

    source venv/bin/activate 2>/dev/null || . venv/bin/activate
    pip install --upgrade pip -q
    pip install -r requirements.txt -q
    success "Python dependencies installed"
    cd - &>/dev/null
}

# ── Frontend Setup ──────────────────────────────────────
setup_frontend() {
    echo -e "\n${BOLD}Setting up Rust/Dioxus frontend...${RESET}\n"

    info "Adding wasm32 target..."
    rustup target add wasm32-unknown-unknown 2>/dev/null
    success "wasm32-unknown-unknown target ready"

    if ! command -v dx &>/dev/null; then
        info "Installing Dioxus CLI (this may take a few minutes)..."
        cargo install dioxus-cli
    fi
    success "Dioxus CLI (dx) ready"
}

# ── Configure Environment ───────────────────────────────
configure_env() {
    echo -e "\n${BOLD}Configure environment...${RESET}\n"
    local env_file="$(dirname "$0")/../backend/.env"

    echo -e "  Select LLM provider:"
    echo -e "    ${BOLD}1)${RESET} Ollama (local, default)"
    echo -e "    ${BOLD}2)${RESET} Cloud API (OpenAI-compatible)"
    read -rp "  Choice [1]: " provider_choice
    provider_choice=${provider_choice:-1}

    if [ "$provider_choice" = "2" ]; then
        read -rp "  Cloud API key: " api_key
        read -rp "  Base URL [https://api.openai.com/v1]: " base_url
        base_url=${base_url:-https://api.openai.com/v1}
        read -rp "  Model name [gpt-4]: " model_name
        model_name=${model_name:-gpt-4}

        if [ -f "$env_file" ]; then
            sed -i.bak "s|^LLM_PROVIDER=.*|LLM_PROVIDER=cloud|" "$env_file" 2>/dev/null || true
            sed -i.bak "s|^CLOUD_API_KEY=.*|CLOUD_API_KEY=$api_key|" "$env_file" 2>/dev/null || true
            sed -i.bak "s|^CLOUD_BASE_URL=.*|CLOUD_BASE_URL=$base_url|" "$env_file" 2>/dev/null || true
            sed -i.bak "s|^CLOUD_MODEL=.*|CLOUD_MODEL=$model_name|" "$env_file" 2>/dev/null || true
            rm -f "${env_file}.bak"
        fi
        success "Cloud API configured"
    else
        read -rp "  Ollama model [mistral]: " ollama_model
        ollama_model=${ollama_model:-mistral}
        if [ -f "$env_file" ]; then
            sed -i.bak "s|^LLM_PROVIDER=.*|LLM_PROVIDER=ollama|" "$env_file" 2>/dev/null || true
            sed -i.bak "s|^OLLAMA_MODEL=.*|OLLAMA_MODEL=$ollama_model|" "$env_file" 2>/dev/null || true
            rm -f "${env_file}.bak"
        fi
        success "Ollama configured with model: $ollama_model"
    fi
}

# ── Interactive Menu ────────────────────────────────────
menu() {
    banner
    echo -e "  ${BOLD}Select an option:${RESET}"
    echo ""
    echo -e "    ${BOLD}1)${RESET}  Check prerequisites"
    echo -e "    ${BOLD}2)${RESET}  Install backend (Python)"
    echo -e "    ${BOLD}3)${RESET}  Install frontend (Rust/Dioxus)"
    echo -e "    ${BOLD}4)${RESET}  Install everything"
    echo -e "    ${BOLD}5)${RESET}  Configure environment (.env)"
    echo -e "    ${BOLD}q)${RESET}  Quit"
    echo ""
    read -rp "  Choice: " choice

    case "$choice" in
        1) check_prerequisites; echo ""; read -rp "  Press Enter to continue..." _; menu ;;
        2) setup_backend;      echo ""; read -rp "  Press Enter to continue..." _; menu ;;
        3) setup_frontend;     echo ""; read -rp "  Press Enter to continue..." _; menu ;;
        4)
            check_prerequisites && {
                setup_backend
                setup_frontend
                echo ""
                success "All components installed!"
                echo -e "\n  Run ${BOLD}./scripts/start_backend.sh${RESET} and ${BOLD}./scripts/start_frontend.sh${RESET} to start."
            }
            echo ""; read -rp "  Press Enter to continue..." _; menu
            ;;
        5) configure_env; echo ""; read -rp "  Press Enter to continue..." _; menu ;;
        q|Q) echo -e "\n  ${DIM}Goodbye!${RESET}\n"; exit 0 ;;
        *) warn "Invalid choice"; menu ;;
    esac
}

menu
