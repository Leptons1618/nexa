# Nexa Support

A production-ready, RAG-powered support chatbot with a **Python FastAPI backend** and a **Dioxus (Rust/WASM) frontend**. Supports both local LLMs via **Ollama** and cloud providers (OpenAI-compatible APIs).

---

## Features

- **Monorepo** — backend, frontend, and scripts in one repository
- **MVC architecture** — clean separation of Models, Views, and Controllers
- **Dual LLM support** — Ollama (local) and any OpenAI-compatible cloud API
- **RAG pipeline** — FAISS or Qdrant vector store with sentence-transformers embeddings
- **Dioxus frontend** — clean, responsive UI built with Rust/WASM and Lucide icons
- **Global state management** — models and config cached globally, no redundant API refetches on navigation
- **API keys management** — configure cloud provider credentials from the Settings UI
- **Confirmation modals** — all destructive actions (delete session, clear history, clear index, delete file) require explicit confirmation
- **Rich text rendering** — Markdown, fenced code with syntax highlighting, LaTeX (KaTeX), tables, task lists, `==highlights==`, auto-linked URLs, images, blockquotes
- **Loading animations** — skeleton placeholders, spinners, staggered section fade-in
- **Fully tested** — pytest suite covering all API endpoints
- **Refusal policy** — returns a safe response when no relevant context is found
- **GZip compression** — backend responses compressed automatically for faster transfers

## Project Layout

```
nexa/
├── backend/                 # Python FastAPI server
│   ├── app/
│   │   ├── config/          # Settings, prompts
│   │   ├── models/          # Pydantic schemas (M)
│   │   ├── views/           # API routes (V)
│   │   ├── controllers/     # Business logic (C)
│   │   ├── services/
│   │   │   ├── llm/         # Ollama + Cloud clients
│   │   │   ├── rag/         # Embeddings, vector store, pipeline
│   │   │   └── ingestion/   # Loader, chunker, ingest service
│   │   ├── main.py          # App factory + GZip middleware
│   │   └── dependencies.py  # Dependency container
│   └── tests/               # pytest suite
├── frontend/                # Dioxus (Rust) web UI
│   ├── src/
│   │   ├── state.rs         # Global AppState (models, selection, loaded)
│   │   ├── api.rs           # HTTP client functions
│   │   ├── models.rs        # Shared data models
│   │   ├── components/      # Sidebar, header, chat bubbles, icons,
│   │   │                    # model selector, confirm modal, rich content
│   │   └── pages/           # Chat, Documents, Settings
│   └── assets/              # CSS
└── scripts/                 # Setup & run scripts (Windows + Linux/macOS)
```

## Quickstart

### Prerequisites

| Tool | Version | Notes |
|------|---------|-------|
| Python | 3.10+ | Backend runtime |
| Rust | latest stable | Via [rustup](https://rustup.rs) |
| Ollama | latest | Local LLM server — *or* a cloud API key |

### Setup (one-time)

**Windows (PowerShell):**
```powershell
.\scripts\setup.ps1
```

**Linux / macOS (Bash):**
```bash
chmod +x scripts/setup.sh
./scripts/setup.sh
```

Both scripts create a Python venv, install dependencies, add the `wasm32-unknown-unknown` target, install the Dioxus CLI, and optionally configure your LLM provider.

### Run

**Backend** (terminal 1):
```powershell
.\scripts\start_backend.ps1        # Windows
# or
cd backend && source venv/bin/activate && python run.py   # Linux/macOS
```

**Frontend** (terminal 2):
```powershell
.\scripts\start_frontend.ps1       # Windows
# or
cd frontend && dx serve             # Linux/macOS
```

The backend serves at `http://localhost:8000`, the frontend at `http://localhost:8080`.

### Test

```powershell
.\scripts\test.ps1
# or
cd backend && pytest -q
```

## API Endpoints

All API routes are prefixed with `/api/`.

### Core

| Method | Path | Description |
|--------|------|-------------|
| GET | `/` | Root health message |
| GET | `/api/health` | Service & LLM health check |
| GET | `/api/config` | View current configuration |
| POST | `/api/chat` | Send a question, get an answer |
| POST | `/api/ingest` | Ingest documents into store |

### Chat & History

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/sessions` | List all chat sessions |
| GET | `/api/sessions/{id}` | Get a specific session |
| DELETE | `/api/sessions/{id}` | Delete a session |
| DELETE | `/api/sessions` | Clear all sessions |

### Ollama

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/ollama/status` | Ollama server status + current model |
| GET | `/api/ollama/models` | List available models |
| POST | `/api/ollama/switch` | Switch active Ollama model |

### Settings

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/settings/prompts` | Get system & RAG prompts |
| PUT | `/api/settings/prompts` | Update prompts |
| GET | `/api/settings/llm` | Get LLM generation parameters |
| PUT | `/api/settings/llm` | Update LLM parameters |
| GET | `/api/settings/api-keys` | Get provider config (key never exposed) |
| PUT | `/api/settings/api-keys` | Update provider, API key, base URL, model |

### Documents & Index

| Method | Path | Description |
|--------|------|-------------|
| GET | `/api/uploads` | List uploaded files |
| POST | `/api/uploads` | Upload files |
| DELETE | `/api/uploads/{name}` | Delete an uploaded file |
| GET | `/api/index/stats` | Vector index statistics |
| POST | `/api/index/rebuild` | Rebuild (reset) the vector index |
| POST | `/api/index/clear` | Clear all vectors and metadata |

### Examples

```bash
# Health
curl http://localhost:8000/api/health

# Chat
curl -X POST http://localhost:8000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "How do I deploy Nexa?"}'

# Update API keys (switch to cloud)
curl -X PUT http://localhost:8000/api/settings/api-keys \
  -H "Content-Type: application/json" \
  -d '{"llm_provider": "cloud", "cloud_api_key": "sk-...", "cloud_base_url": "https://api.openai.com/v1", "cloud_model": "gpt-4"}'

# Clear vector index
curl -X POST http://localhost:8000/api/index/clear
```

## Cloud Provider Configuration

You can configure the LLM provider in two ways:

### 1. Via the Settings UI

Navigate to **Settings > API Keys & Provider**, select "Cloud API", enter your API key, base URL, and model name, then save.

### 2. Via Environment Variables

Copy `backend/.env.example` to `backend/.env` and adjust:

| Variable | Default | Description |
|----------|---------|-------------|
| `LLM_PROVIDER` | `ollama` | `ollama` or `cloud` |
| `OLLAMA_BASE_URL` | `http://localhost:11434` | Ollama server address |
| `OLLAMA_MODEL` | `mistral` | Ollama model name |
| `CLOUD_API_KEY` | (empty) | API key for cloud LLM |
| `CLOUD_BASE_URL` | `https://api.openai.com/v1` | OpenAI-compatible URL |
| `CLOUD_MODEL` | `gpt-4` | Cloud model identifier |
| `VECTOR_STORE` | `faiss` | `faiss` or `qdrant` |

**Compatible cloud providers:** OpenAI, Groq, Together AI, Fireworks, Mistral, or any OpenAI-compatible endpoint.

## Android Testing

Dioxus supports mobile targets. To test on Android:

1. **Install Android prerequisites:**
   - Android Studio with SDK 33+
   - Android NDK (side-by-side)
   - Set `ANDROID_HOME` and `NDK_HOME` environment variables

2. **Add Rust Android targets:**
   ```bash
   rustup target add aarch64-linux-android armv7-linux-androideabi
   ```

3. **Configure Dioxus for mobile:**
   Update `Dioxus.toml` to include an `[application.android]` section (refer to the [Dioxus mobile guide](https://dioxuslabs.com/learn/0.6/guides/mobile)).

4. **Build & run:**
   ```bash
   dx serve --platform android
   ```

5. **Backend connectivity:** Ensure the Android device/emulator can reach the backend. Use your machine's LAN IP instead of `localhost` in API URLs.

> **Note:** WASM-based web builds work in any mobile browser without native Android compilation.

## License

[MIT](LICENSE)
