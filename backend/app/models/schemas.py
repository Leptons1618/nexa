"""Pydantic request / response schemas (the 'M' in MVC)."""

from __future__ import annotations

from typing import List, Optional

from pydantic import BaseModel, Field


# ── Requests ────────────────────────────────────────────


class ChatRequest(BaseModel):
    message: str = Field(..., min_length=1, description="User question")
    session_id: Optional[str] = Field(None, description="Optional session identifier")


class IngestRequest(BaseModel):
    paths: List[str] = Field(..., min_length=1, description="Files or directories to ingest")
    tags: Optional[List[str]] = Field(default=None, description="Optional metadata tags")
    version: Optional[str] = Field(default=None, description="Documentation version string")


# ── Responses ───────────────────────────────────────────


class ChatResponse(BaseModel):
    answer: str
    sources: List[str]


class HealthResponse(BaseModel):
    status: str
    llm_connected: bool
    detail: str = ""


class IngestResponse(BaseModel):
    chunks_indexed: int


class ConfigResponse(BaseModel):
    llm_provider: str
    model_name: str
    vector_store: str
    embedding_model: str


class OllamaModelEntry(BaseModel):
    name: str
    size: Optional[int] = None
    digest: Optional[str] = None
    modified_at: Optional[str] = None


class OllamaModelsResponse(BaseModel):
    models: List[OllamaModelEntry]


class OllamaStatusResponse(BaseModel):
    running: bool
    base_url: str
    model: str
    models_available: int = 0


class ErrorResponse(BaseModel):
    detail: str


# ── Domain ──────────────────────────────────────────────


class DocumentChunk(BaseModel):
    id: str
    text: str
    metadata: dict
