# Nexa Support

A production-ready, RAG-powered support chatbot with a **Python FastAPI backend** and a **Dioxus (Rust/WASM) frontend**. Supports both local LLMs via **Ollama** and cloud providers (OpenAI-compatible APIs).

## Features

- **Monorepo** — backend, frontend, and scripts in one repository
- **MVC architecture** — clean separation of Models, Views, and Controllers
- **Dual LLM support** — Ollama (local) and any OpenAI-compatible cloud API
- **RAG pipeline** — FAISS or Qdrant vector store with sentence-transformers embeddings
- **Dioxus frontend** — clean, responsive UI built with Rust/WASM and Lucide icons
- **Fully tested** — pytest suite covering all API endpoints
- **Refusal policy** — returns a safe response when no relevant context is found

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
│   │   ├── main.py          # App factory
│   │   └── dependencies.py  # Dependency container
│   └── tests/               # pytest suite
├── frontend/                # Dioxus (Rust) web UI
│   ├── src/
│   │   ├── components/      # Sidebar, header, chat bubbles, icons
│   │   └── pages/           # Chat, Documents, Settings
│   └── assets/              # CSS
└── scripts/                 # PowerShell setup & run scripts
```

## Quickstart

### Prerequisites

- Python 3.10+
- Rust toolchain (via [rustup](https://rustup.rs))
- Ollama running locally (or a cloud API key)

### Setup (one-time)

```powershell
.\scripts\setup.ps1
```

This creates a Python venv, installs dependencies, adds the wasm32 target, and installs the Dioxus CLI.

### Run

**Backend** (terminal 1):
```powershell
.\scripts\start_backend.ps1
```

**Frontend** (terminal 2):
```powershell
.\scripts\start_frontend.ps1
```

The backend serves at `http://localhost:8000`, the frontend at `http://localhost:8080`.

### Test

```powershell
.\scripts\test.ps1
```

## API Endpoints

All API routes are prefixed with `/api/`.

| Method | Path          | Description                  |
|--------|---------------|------------------------------|
| GET    | `/`           | Root health message          |
| GET    | `/api/health` | Service & LLM health check   |
| POST   | `/api/chat`   | Send a question, get answer  |
| POST   | `/api/ingest` | Ingest documents into store  |
| GET    | `/api/config` | View current configuration   |

### Examples

```bash
# Health
curl http://localhost:8000/api/health

# Chat
curl -X POST http://localhost:8000/api/chat \
  -H "Content-Type: application/json" \
  -d '{"message": "How do I deploy Nexa?"}'

# Ingest
curl -X POST http://localhost:8000/api/ingest \
  -H "Content-Type: application/json" \
  -d '{"paths": ["docs/"], "version": "2.0.0"}'

# Config
curl http://localhost:8000/api/config
```

## Configuration

Copy `backend/.env.example` to `backend/.env` and adjust:

| Variable         | Default                               | Description              |
|------------------|---------------------------------------|--------------------------|
| `LLM_PROVIDER`   | `ollama`                              | `ollama` or `cloud`      |
| `OLLAMA_BASE_URL` | `http://localhost:11434`             | Ollama server address    |
| `OLLAMA_MODEL`   | `mistral`                             | Ollama model name        |
| `CLOUD_API_KEY`  | (empty)                               | API key for cloud LLM    |
| `CLOUD_BASE_URL` | `https://api.openai.com/v1`           | OpenAI-compatible URL    |
| `CLOUD_MODEL`    | `gpt-4`                              | Cloud model identifier   |
| `VECTOR_STORE`   | `faiss`                               | `faiss` or `qdrant`      |

See `.env.example` for the full list.

## License

[MIT](LICENSE)
