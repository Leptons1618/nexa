"""Application settings loaded from environment variables / .env file."""

from __future__ import annotations

import os
from pathlib import Path

from pydantic import Field
from pydantic_settings import BaseSettings, SettingsConfigDict


class AppSettings(BaseSettings):
    model_config = SettingsConfigDict(
        env_file=".env",
        env_file_encoding="utf-8",
        case_sensitive=False,
        extra="ignore",
    )

    # ── Server ──────────────────────────────────────────
    app_name: str = Field("Nexa Support", description="Service display name")
    host: str = Field("0.0.0.0", description="API bind host")
    port: int = Field(8000, description="API bind port")

    # ── LLM provider ───────────────────────────────────
    llm_provider: str = Field("ollama", description="'ollama' or 'cloud'")

    # Ollama
    ollama_base_url: str = Field("http://localhost:11434", description="Ollama server URL")
    ollama_model: str = Field("mistral", description="Ollama model name")
    ollama_use_chat_api: bool = Field(True, description="Use /api/chat instead of /api/generate")

    # Cloud (OpenAI-compatible)
    cloud_api_key: str = Field("", description="API key for cloud LLM")
    cloud_base_url: str = Field("https://api.openai.com/v1", description="Cloud LLM base URL")
    cloud_model: str = Field("gpt-4", description="Cloud model identifier")

    # Common LLM
    temperature: float = Field(0.2)
    top_p: float = Field(0.9)
    max_tokens: int = Field(512)

    # ── Embeddings ──────────────────────────────────────
    embedding_model_name: str = Field("sentence-transformers/all-MiniLM-L6-v2")
    embedding_batch_size: int = Field(32)

    # ── Vector store ────────────────────────────────────
    vector_store: str = Field("faiss", description="'faiss' or 'qdrant'")
    index_path: str = Field("data/index.faiss")
    metadata_path: str = Field("data/index_meta.json")
    qdrant_url: str | None = Field(None)
    qdrant_api_key: str | None = Field(None)
    qdrant_collection: str = Field("nexa_support")

    # ── RAG ─────────────────────────────────────────────
    chunk_size: int = Field(400)
    chunk_overlap: int = Field(80)
    top_k: int = Field(4)
    similarity_threshold: float = Field(0.35)

    # ── Prompts ─────────────────────────────────────────
    system_prompt_path: str = Field("app/config/prompts/system.txt")
    rag_prompt_path: str = Field("app/config/prompts/rag_addon.txt")

    # ── Misc ────────────────────────────────────────────
    log_level: str = Field("INFO")
    data_dir: str = Field("data")

    def ensure_dirs(self) -> None:
        Path(self.data_dir).mkdir(parents=True, exist_ok=True)
        Path(os.path.dirname(self.index_path) or ".").mkdir(parents=True, exist_ok=True)
        Path(os.path.dirname(self.metadata_path) or ".").mkdir(parents=True, exist_ok=True)


def get_settings() -> AppSettings:
    """Convenience alias used by the dependency container."""
    from app.dependencies import get_settings as _gs

    return _gs()
