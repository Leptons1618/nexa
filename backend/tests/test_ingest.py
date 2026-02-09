"""Tests for the /api/ingest endpoint."""

import pytest


class TestIngest:
    def test_ingest_success(self, client):
        resp = client.post(
            "/api/ingest",
            json={"paths": ["docs/"], "version": "1.0.0"},
        )
        assert resp.status_code == 200
        body = resp.json()
        assert body["chunks_indexed"] == 5

    def test_ingest_with_tags(self, client):
        resp = client.post(
            "/api/ingest",
            json={"paths": ["README.md"], "tags": ["release", "v2"], "version": "2.0.0"},
        )
        assert resp.status_code == 200
        assert resp.json()["chunks_indexed"] == 5

    def test_ingest_empty_paths_rejected(self, client):
        resp = client.post("/api/ingest", json={"paths": []})
        assert resp.status_code == 422  # pydantic min_length=1

    def test_ingest_missing_paths(self, client):
        resp = client.post("/api/ingest", json={})
        assert resp.status_code == 422
