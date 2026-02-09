"""Shared fixtures for backend tests.

All heavy services (LLM, embeddings, vector-store, pipeline, ingestion)
are replaced with lightweight mocks so tests run instantly without GPUs,
models, or network access.
"""

from __future__ import annotations

from typing import Any, Dict, List, Optional, Tuple

import pytest
from fastapi.testclient import TestClient


# ── Mock implementations ────────────────────────────────


class MockLLMClient:
    """Fake LLM that always returns a fixed answer."""

    def generate(self, prompt: str, system_prompt: str = "") -> str:
        return "Mock LLM answer based on provided context."

    def health_check(self) -> bool:
        return True


class MockLLMClientUnhealthy:
    def generate(self, prompt: str, system_prompt: str = "") -> str:
        raise ConnectionError("LLM not reachable")

    def health_check(self) -> bool:
        return False


class MockEmbedder:
    dimension = 384

    def embed_documents(self, texts: List[str]) -> List[List[float]]:
        return [[0.1] * 384 for _ in texts]

    def embed_query(self, text: str) -> List[float]:
        return [0.1] * 384


class MockVectorStore:
    def __init__(self) -> None:
        self._texts: List[str] = []
        self._meta: List[Dict[str, Any]] = []

    def add_texts(
        self,
        texts: List[str],
        embeddings: List[List[float]],
        metadatas: List[Dict[str, Any]],
    ) -> None:
        self._texts.extend(texts)
        self._meta.extend(metadatas)

    def search(
        self,
        query_embedding: List[float],
        top_k: int,
        score_threshold: float,
    ) -> List[Tuple[str, float, Dict[str, Any]]]:
        return [
            (
                "Nexa supports multiple deployment modes.",
                0.92,
                {"document_name": "deploy.md", "text": "Nexa supports multiple deployment modes."},
            )
        ]

    def save(self) -> None:
        pass


class MockVectorStoreEmpty(MockVectorStore):
    """Returns no results — triggers refusal response."""

    def search(self, *_a, **_kw):  # type: ignore[override]
        return []


class MockPipeline:
    def generate(self, query: str) -> Tuple[str, List[str]]:
        return ("Mock answer from pipeline.", ["doc1.md"])


class MockPipelineNoContext:
    def generate(self, query: str) -> Tuple[str, List[str]]:
        return (
            "I can only help with questions related to Nexa. "
            "This information is not available in the documentation.",
            [],
        )


class MockIngestionService:
    def ingest_paths(
        self,
        paths: List[str],
        tags: Optional[List[str]] = None,
        version: Optional[str] = None,
    ) -> int:
        return 5


# ── Fixtures ────────────────────────────────────────────


@pytest.fixture()
def client() -> TestClient:
    """TestClient with all dependencies mocked (healthy LLM, matching context)."""
    import app.dependencies as deps

    deps.reset_cache()
    deps._cache["llm"] = MockLLMClient()
    deps._cache["embedder"] = MockEmbedder()
    deps._cache["store"] = MockVectorStore()
    deps._cache["pipeline"] = MockPipeline()
    deps._cache["ingestion"] = MockIngestionService()

    from app.main import create_app

    application = create_app()
    yield TestClient(application)
    deps.reset_cache()


@pytest.fixture()
def client_unhealthy_llm() -> TestClient:
    """TestClient where LLM health-check fails."""
    import app.dependencies as deps

    deps.reset_cache()
    deps._cache["llm"] = MockLLMClientUnhealthy()
    deps._cache["embedder"] = MockEmbedder()
    deps._cache["store"] = MockVectorStore()
    deps._cache["pipeline"] = MockPipelineNoContext()
    deps._cache["ingestion"] = MockIngestionService()

    from app.main import create_app

    application = create_app()
    yield TestClient(application)
    deps.reset_cache()
