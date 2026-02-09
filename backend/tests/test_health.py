"""Tests for the /api/health endpoint and root /."""

import pytest


class TestRoot:
    def test_root_returns_ok(self, client):
        resp = client.get("/")
        assert resp.status_code == 200
        # May return JSON (no static dir) or HTML (with static dir)
        ct = resp.headers.get("content-type", "")
        if "application/json" in ct:
            body = resp.json()
            assert "Nexa Support" in body["message"]
            assert "version" in body
        else:
            # Static frontend served — HTML response is fine
            assert len(resp.content) > 0


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
