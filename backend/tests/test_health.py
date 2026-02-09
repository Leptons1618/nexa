"""Tests for the /api/health endpoint and root /."""

import pytest


class TestRoot:
    def test_root_returns_message(self, client):
        resp = client.get("/")
        assert resp.status_code == 200
        body = resp.json()
        assert "Nexa Support" in body["message"]
        assert "version" in body


class TestHealth:
    def test_health_ok(self, client):
        resp = client.get("/api/health")
        assert resp.status_code == 200
        body = resp.json()
        assert body["status"] == "ok"
        assert body["llm_connected"] is True

    def test_health_degraded_when_llm_down(self, client_unhealthy_llm):
        resp = client_unhealthy_llm.get("/api/health")
        assert resp.status_code == 200
        body = resp.json()
        assert body["status"] == "degraded"
        assert body["llm_connected"] is False
