"""Tests for the /api/chat endpoint."""

import pytest


class TestChat:
    def test_chat_success(self, client):
        resp = client.post("/api/chat", json={"message": "How do I deploy Nexa?"})
        assert resp.status_code == 200
        body = resp.json()
        assert "answer" in body
        assert isinstance(body["sources"], list)

    def test_chat_empty_message_rejected(self, client):
        resp = client.post("/api/chat", json={"message": ""})
        assert resp.status_code == 422  # pydantic validation (min_length=1)

    def test_chat_whitespace_only_rejected(self, client):
        resp = client.post("/api/chat", json={"message": "   "})
        # Passes pydantic min_length but controller strips & rejects
        assert resp.status_code == 400
        assert "empty" in resp.json()["detail"].lower()

    def test_chat_missing_body(self, client):
        resp = client.post("/api/chat")
        assert resp.status_code == 422

    def test_chat_with_session_id(self, client):
        resp = client.post(
            "/api/chat",
            json={"message": "Tell me about Nexa", "session_id": "sess-001"},
        )
        assert resp.status_code == 200
        body = resp.json()
        assert body["answer"]
