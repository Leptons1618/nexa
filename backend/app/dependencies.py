"""Dependency container — lazy-initialised singletons for services."""

from __future__ import annotations

import logging
from functools import lru_cache
from typing import Any, Dict

from app.config.settings import AppSettings

logger = logging.getLogger(__name__)

_cache: Dict[str, Any] = {}


@lru_cache(maxsize=1)
def get_settings() -> AppSettings:
    settings = AppSettings()
    settings.ensure_dirs()
    return settings


def get_llm_client():
    """Return the configured LLM client (Ollama or Cloud)."""
    if "llm" not in _cache:
        settings = get_settings()
        if settings.llm_provider == "ollama":
            from app.services.llm.ollama_client import OllamaClient

            _cache["llm"] = OllamaClient(
                base_url=settings.ollama_base_url,
                model=settings.ollama_model,
                temperature=settings.temperature,
                top_p=settings.top_p,
                max_tokens=settings.max_tokens,
                use_chat_api=settings.ollama_use_chat_api,
            )
        elif settings.llm_provider == "cloud":
            from app.services.llm.cloud_client import CloudLLMClient

            _cache["llm"] = CloudLLMClient(
                api_key=settings.cloud_api_key,
                base_url=settings.cloud_base_url,
                model=settings.cloud_model,
                temperature=settings.temperature,
                top_p=settings.top_p,
                max_tokens=settings.max_tokens,
            )
        else:
            raise ValueError(f"Unknown LLM provider: {settings.llm_provider}")
        logger.info("LLM client created: provider=%s", settings.llm_provider)
    return _cache["llm"]


def get_embedder():
    """Return the embedding service (sentence-transformers)."""
    if "embedder" not in _cache:
        settings = get_settings()
        from app.services.rag.embeddings import EmbeddingService

        _cache["embedder"] = EmbeddingService(
            model_name=settings.embedding_model_name,
            batch_size=settings.embedding_batch_size,
        )
        logger.info("Embedding service created: model=%s", settings.embedding_model_name)
    return _cache["embedder"]


def get_vector_store():
    """Return the vector store (FAISS or Qdrant)."""
    if "store" not in _cache:
        settings = get_settings()
        embedder = get_embedder()
        from app.services.rag.vector_store import create_vector_store

        _cache["store"] = create_vector_store(
            kind=settings.vector_store,
            dim=embedder.dimension,
            index_path=settings.index_path,
            metadata_path=settings.metadata_path,
            qdrant_url=settings.qdrant_url,
            qdrant_api_key=settings.qdrant_api_key,
            qdrant_collection=settings.qdrant_collection,
        )
        logger.info("Vector store created: kind=%s", settings.vector_store)
    return _cache["store"]


def get_rag_pipeline():
    """Return the RAG pipeline (embedder + store + LLM)."""
    if "pipeline" not in _cache:
        settings = get_settings()
        from app.services.rag.pipeline import RAGPipeline

        _cache["pipeline"] = RAGPipeline(
            embedder=get_embedder(),
            store=get_vector_store(),
            llm=get_llm_client(),
            system_prompt_path=settings.system_prompt_path,
            rag_prompt_path=settings.rag_prompt_path,
            top_k=settings.top_k,
            similarity_threshold=settings.similarity_threshold,
        )
        logger.info("RAG pipeline created")
    return _cache["pipeline"]


def get_ingestion_service():
    """Return the ingestion service."""
    if "ingestion" not in _cache:
        settings = get_settings()
        from app.services.ingestion.ingest_service import IngestionService

        _cache["ingestion"] = IngestionService(
            embedder=get_embedder(),
            store=get_vector_store(),
            chunk_size=settings.chunk_size,
            chunk_overlap=settings.chunk_overlap,
        )
        logger.info("Ingestion service created")
    return _cache["ingestion"]


def reset_cache() -> None:
    """Clear all cached singletons. Used by tests."""
    _cache.clear()
    get_settings.cache_clear()
