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


class SwitchModelRequest(BaseModel):
    model: str = Field(..., min_length=1, description="Ollama model name to switch to")


class SwitchModelResponse(BaseModel):
    previous_model: str
    current_model: str


class ErrorResponse(BaseModel):
    detail: str


class UploadedFileInfo(BaseModel):
    original_name: str
    saved_path: str
    size: int


class UploadResponse(BaseModel):
    files: List[UploadedFileInfo]
    chunks_indexed: int


# ── Domain ──────────────────────────────────────────────


class DocumentChunk(BaseModel):
    id: str
    text: str
    metadata: dict


# ── Chat History ────────────────────────────────────────


class HistoryMessage(BaseModel):
    role: str  # "user" or "assistant"
    text: str
    sources: List[str] = []


class SessionCreate(BaseModel):
    id: str
    title: str
    messages: List[HistoryMessage] = []
    documents: List[str] = []


class SessionSummary(BaseModel):
    id: str
    title: str
    message_count: int = 0
    document_count: int = 0
    created_at: str = ""
    updated_at: str = ""


class SessionDetail(BaseModel):
    id: str
    title: str
    messages: List[HistoryMessage] = []
    documents: List[str] = []
    created_at: str = ""
    updated_at: str = ""


class SessionListResponse(BaseModel):
    sessions: List[SessionSummary]


# ── Prompts ─────────────────────────────────────────────


class PromptsResponse(BaseModel):
    system_prompt: str
    rag_addon_prompt: str


class PromptsUpdateRequest(BaseModel):
    system_prompt: Optional[str] = None
    rag_addon_prompt: Optional[str] = None


# ── Uploaded files ──────────────────────────────────────


class UploadedFileEntry(BaseModel):
    name: str
    path: str
    size: int
    modified_at: str


class UploadedFilesListResponse(BaseModel):
    files: List[UploadedFileEntry]


# ── Index stats ─────────────────────────────────────────


class IndexStatsResponse(BaseModel):
    total_vectors: int
    total_metadata: int
    index_path: str
    metadata_path: str
    index_size_bytes: int
    metadata_size_bytes: int


# ── LLM Settings ───────────────────────────────────────


class LLMSettingsResponse(BaseModel):
    temperature: float
    top_p: float
    max_tokens: int
    chunk_size: int
    chunk_overlap: int
    top_k: int
    similarity_threshold: float


class LLMSettingsUpdateRequest(BaseModel):
    temperature: Optional[float] = None
    top_p: Optional[float] = None
    max_tokens: Optional[int] = None
    chunk_size: Optional[int] = None
    chunk_overlap: Optional[int] = None
    top_k: Optional[int] = None
    similarity_threshold: Optional[float] = None
