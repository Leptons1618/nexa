"""Tests for Pydantic schemas and the /api/config endpoint."""

import pytest

from app.models.schemas import (
    ChatRequest,
    ChatResponse,
    ConfigResponse,
    HealthResponse,
    IngestRequest,
    IngestResponse,
)


class TestSchemas:
    def test_chat_request_valid(self):
        req = ChatRequest(message="Hello")
        assert req.message == "Hello"

    def test_chat_request_with_session(self):
        req = ChatRequest(message="Hi", session_id="s1")
        assert req.session_id == "s1"

    def test_chat_response(self):
        resp = ChatResponse(answer="Yes", sources=["a.md"])
        assert resp.answer == "Yes"
        assert resp.sources == ["a.md"]

    def test_health_response(self):
        resp = HealthResponse(status="ok", llm_connected=True, detail="fine")
        assert resp.llm_connected is True

    def test_ingest_request_valid(self):
        req = IngestRequest(paths=["docs/"])
        assert req.paths == ["docs/"]
        assert req.tags is None

    def test_ingest_response(self):
        resp = IngestResponse(chunks_indexed=10)
        assert resp.chunks_indexed == 10

    def test_config_response(self):
        resp = ConfigResponse(
            llm_provider="ollama",
            model_name="mistral",
            vector_store="faiss",
            embedding_model="all-MiniLM-L6-v2",
        )
        assert resp.llm_provider == "ollama"


class TestConfigEndpoint:
    def test_config_returns_provider_info(self, client):
        resp = client.get("/api/config")
        assert resp.status_code == 200
        body = resp.json()
        assert "llm_provider" in body
        assert "model_name" in body
        assert "vector_store" in body
        assert "embedding_model" in body
