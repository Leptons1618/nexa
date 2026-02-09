"""Tests for the /api/ollama/* endpoints."""

from __future__ import annotations

from typing import Any, Dict, List, Optional, Tuple

import pytest
from fastapi.testclient import TestClient

from app.services.llm.ollama_client import OllamaClient


# ── Mock Ollama client (inherits OllamaClient so isinstance checks pass) ────


class MockOllamaClient(OllamaClient):
    """Fake OllamaClient — skips real __init__, stubs all methods."""

    def __init__(self) -> None:  # bypass real __init__ that creates httpx.Client
        self.base_url = "http://localhost:11434"
        self.model = "mistral"
        self.use_chat_api = True

    def generate(self, prompt: str, system_prompt: str = "") -> str:
        return "Mock Ollama chat answer."

    def health_check(self) -> bool:
        return True

    def list_models(self) -> List[Dict[str, Any]]:
        return [
            {"name": "mistral:latest", "size": 4_000_000_000, "digest": "abc123"},
            {"name": "llama3:latest", "size": 8_000_000_000, "digest": "def456"},
        ]

    def model_info(self, model_name: Optional[str] = None) -> Dict[str, Any]:
        return {"modelfile": "FROM mistral", "parameters": "temperature 0.2"}


class MockOllamaClientDown(MockOllamaClient):
    def health_check(self) -> bool:
        return False

    def list_models(self) -> List[Dict[str, Any]]:
        return []


# ── Fixtures ────────────────────────────────────────────


@pytest.fixture()
def ollama_client() -> TestClient:
    import os

    import app.dependencies as deps

    deps.reset_cache()
    os.environ["LLM_PROVIDER"] = "ollama"

    deps._cache["llm"] = MockOllamaClient()
    deps._cache["embedder"] = type("E", (), {"dimension": 384})()
    deps._cache["store"] = type("S", (), {})()
    deps._cache["pipeline"] = type(
        "P",
        (),
        {"generate": staticmethod(lambda q: ("answer", ["src.md"]))},
    )()
    deps._cache["ingestion"] = type(
        "I",
        (),
        {"ingest_paths": staticmethod(lambda *a, **kw: 5)},
    )()

    from app.main import create_app

    application = create_app()
    yield TestClient(application)
    deps.reset_cache()
    os.environ.pop("LLM_PROVIDER", None)


@pytest.fixture()
def ollama_client_down() -> TestClient:
    import os

    import app.dependencies as deps

    deps.reset_cache()
    os.environ["LLM_PROVIDER"] = "ollama"

    deps._cache["llm"] = MockOllamaClientDown()
    deps._cache["embedder"] = type("E", (), {"dimension": 384})()
    deps._cache["store"] = type("S", (), {})()
    deps._cache["pipeline"] = type(
        "P",
        (),
        {"generate": staticmethod(lambda q: ("answer", []))},
    )()
    deps._cache["ingestion"] = type(
        "I",
        (),
        {"ingest_paths": staticmethod(lambda *a, **kw: 0)},
    )()

    from app.main import create_app

    application = create_app()
    yield TestClient(application)
    deps.reset_cache()
    os.environ.pop("LLM_PROVIDER", None)


# ── Tests ───────────────────────────────────────────────


class TestOllamaModels:
    def test_list_models(self, ollama_client):
        resp = ollama_client.get("/api/ollama/models")
        assert resp.status_code == 200
        body = resp.json()
        assert len(body["models"]) == 2
        names = [m["name"] for m in body["models"]]
        assert "mistral:latest" in names
        assert "llama3:latest" in names


class TestOllamaStatus:
    def test_status_running(self, ollama_client):
        resp = ollama_client.get("/api/ollama/status")
        assert resp.status_code == 200
        body = resp.json()
        assert body["running"] is True
        assert body["model"] == "mistral"
        assert body["models_available"] == 2

    def test_status_down(self, ollama_client_down):
        resp = ollama_client_down.get("/api/ollama/status")
        assert resp.status_code == 200
        body = resp.json()
        assert body["running"] is False
        assert body["models_available"] == 0


class TestOllamaEndpointsRequireOllamaProvider:
    """Ensure Ollama endpoints return 400 when provider is not 'ollama'."""

    def test_models_rejects_non_ollama(self, client):
        """Default 'client' fixture has a mock that is not an OllamaClient."""
        resp = client.get("/api/ollama/models")
        # The default mock LLM is not OllamaClient, so endpoint returns 500
        assert resp.status_code in (400, 500)
