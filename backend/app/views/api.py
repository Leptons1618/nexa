"""API routes — the 'V' in MVC.  Thin layer that delegates to controllers."""

from __future__ import annotations

import json
import logging
import os
import uuid
from pathlib import Path
from typing import List, Optional

from fastapi import APIRouter, File, Form, HTTPException, UploadFile
from fastapi.responses import StreamingResponse

from app.controllers.chat_controller import ChatController
from app.controllers.health_controller import HealthController
from app.controllers.ingest_controller import IngestController
from app.dependencies import (
    get_ingestion_service,
    get_llm_client,
    get_rag_pipeline,
    get_settings,
)
from app.models.schemas import (
    ChatRequest,
    ChatResponse,
    ConfigResponse,
    HealthResponse,
    IngestRequest,
    IngestResponse,
    OllamaModelEntry,
    OllamaModelsResponse,
    OllamaStatusResponse,
    SessionCreate,
    SessionDetail,
    SessionListResponse,
    SessionSummary,
    SwitchModelRequest,
    SwitchModelResponse,
    UploadResponse,
    UploadedFileInfo,
)
from app.services.history import HistoryService

logger = logging.getLogger(__name__)
router = APIRouter(prefix="/api", tags=["api"])


@router.get("/health", response_model=HealthResponse)
def health():
    controller = HealthController(get_llm_client())
    return controller.check_health()


@router.post("/chat", response_model=ChatResponse)
def chat(req: ChatRequest):
    controller = ChatController(get_rag_pipeline())
    try:
        answer, sources = controller.handle_chat(req.message)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return ChatResponse(answer=answer, sources=sources)


@router.post("/chat/stream")
def chat_stream(req: ChatRequest):
    """Server-Sent Events stream for chat — tokens arrive incrementally."""
    cleaned = req.message.strip()
    if not cleaned:
        raise HTTPException(status_code=400, detail="Message cannot be empty")

    pipeline = get_rag_pipeline()

    def event_generator():
        try:
            for kind, payload in pipeline.generate_stream(cleaned):
                if kind == "sources":
                    yield f"event: sources\ndata: {json.dumps(payload)}\n\n"
                elif kind == "contexts":
                    yield f"event: contexts\ndata: {json.dumps(payload)}\n\n"
                elif kind == "token":
                    yield f"event: token\ndata: {json.dumps(payload)}\n\n"
                elif kind == "done":
                    yield f"event: done\ndata: \"\"\n\n"
        except Exception as exc:
            logger.error("Stream error: %s", exc, exc_info=True)
            yield f"event: error\ndata: {json.dumps(str(exc))}\n\n"

    return StreamingResponse(
        event_generator(),
        media_type="text/event-stream",
        headers={
            "Cache-Control": "no-cache",
            "Connection": "keep-alive",
            "X-Accel-Buffering": "no",
        },
    )


@router.post("/ingest", response_model=IngestResponse)
def ingest(req: IngestRequest):
    controller = IngestController(get_ingestion_service())
    try:
        count = controller.ingest(req.paths, tags=req.tags, version=req.version)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return IngestResponse(chunks_indexed=count)


UPLOAD_DIR = Path("data/uploads")


@router.post("/upload", response_model=UploadResponse)
async def upload_documents(
    files: List[UploadFile] = File(...),
    tags: Optional[str] = Form(None),
    version: Optional[str] = Form(None),
):
    """Upload files, save to disk, and auto-ingest into the vector store."""
    UPLOAD_DIR.mkdir(parents=True, exist_ok=True)

    ALLOWED_EXTENSIONS = {
        ".pdf", ".txt", ".md", ".doc", ".docx",
        ".html", ".htm", ".json", ".xml", ".csv", ".rst",
    }
    MAX_FILE_SIZE = 50 * 1024 * 1024  # 50 MB

    saved_paths: List[str] = []
    file_infos: List[UploadedFileInfo] = []

    for f in files:
        ext = Path(f.filename or "").suffix.lower()
        if ext not in ALLOWED_EXTENSIONS:
            raise HTTPException(
                status_code=400,
                detail=f"Unsupported file type: {ext} ({f.filename})",
            )

        content = await f.read()
        if len(content) > MAX_FILE_SIZE:
            raise HTTPException(
                status_code=400,
                detail=f"File too large (max 50 MB): {f.filename}",
            )

        # Use unique filename to avoid collisions
        safe_name = f"{uuid.uuid4().hex[:8]}_{Path(f.filename).name}"
        dest = UPLOAD_DIR / safe_name
        dest.write_bytes(content)
        saved_paths.append(str(dest))
        file_infos.append(UploadedFileInfo(
            original_name=f.filename or "unknown",
            saved_path=str(dest),
            size=len(content),
        ))
        logger.info("Uploaded file: %s → %s (%d bytes)", f.filename, dest, len(content))

    # Auto-ingest saved files
    tag_list = (
        [t.strip() for t in tags.split(",") if t.strip()] if tags else None
    )
    ver = version.strip() if version and version.strip() else None

    controller = IngestController(get_ingestion_service())
    try:
        chunks = controller.ingest(saved_paths, tags=tag_list, version=ver)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))

    return UploadResponse(
        files=file_infos,
        chunks_indexed=chunks,
    )


@router.get("/config", response_model=ConfigResponse)
def config():
    settings = get_settings()
    model_name = (
        settings.ollama_model
        if settings.llm_provider == "ollama"
        else settings.cloud_model
    )
    return ConfigResponse(
        llm_provider=settings.llm_provider,
        model_name=model_name,
        vector_store=settings.vector_store,
        embedding_model=settings.embedding_model_name,
    )


# ── Ollama-specific endpoints ───────────────────────────


@router.get("/ollama/models", response_model=OllamaModelsResponse)
def ollama_models():
    """List models available on the Ollama server."""
    settings = get_settings()
    if settings.llm_provider != "ollama":
        raise HTTPException(
            status_code=400,
            detail="Ollama endpoints are only available when llm_provider is 'ollama'.",
        )

    from app.services.llm.ollama_client import OllamaClient

    client = get_llm_client()
    if not isinstance(client, OllamaClient):
        raise HTTPException(status_code=500, detail="LLM client is not an OllamaClient")

    raw_models = client.list_models()
    entries = [
        OllamaModelEntry(
            name=m.get("name", "unknown"),
            size=m.get("size"),
            digest=m.get("digest"),
            modified_at=m.get("modified_at"),
        )
        for m in raw_models
    ]
    return OllamaModelsResponse(models=entries)


@router.get("/ollama/status", response_model=OllamaStatusResponse)
def ollama_status():
    """Check whether Ollama is running and report basic info."""
    settings = get_settings()
    if settings.llm_provider != "ollama":
        raise HTTPException(
            status_code=400,
            detail="Ollama endpoints are only available when llm_provider is 'ollama'.",
        )

    from app.services.llm.ollama_client import OllamaClient

    client = get_llm_client()
    if not isinstance(client, OllamaClient):
        raise HTTPException(status_code=500, detail="LLM client is not an OllamaClient")

    running = client.health_check()
    models = client.list_models() if running else []
    return OllamaStatusResponse(
        running=running,
        base_url=client.base_url,
        model=client.model,
        models_available=len(models),
    )


@router.put("/ollama/model", response_model=SwitchModelResponse)
def switch_ollama_model(req: SwitchModelRequest):
    """Switch the active Ollama model at runtime."""
    settings = get_settings()
    if settings.llm_provider != "ollama":
        raise HTTPException(
            status_code=400,
            detail="Model switching is only available when llm_provider is 'ollama'.",
        )

    from app.services.llm.ollama_client import OllamaClient

    client = get_llm_client()
    if not isinstance(client, OllamaClient):
        raise HTTPException(status_code=500, detail="LLM client is not an OllamaClient")

    previous = client.model
    client.model = req.model
    logger.info("Switched Ollama model: %s → %s", previous, req.model)
    return SwitchModelResponse(previous_model=previous, current_model=req.model)


# ── Session history endpoints ────────────────────────────

_history = HistoryService()


@router.get("/sessions", response_model=SessionListResponse)
def list_sessions():
    """List all saved chat sessions."""
    sessions = _history.list_sessions()
    return SessionListResponse(sessions=sessions)


@router.get("/sessions/{session_id}", response_model=SessionDetail)
def get_session(session_id: str):
    """Get full session details including messages."""
    session = _history.get_session(session_id)
    if session is None:
        raise HTTPException(status_code=404, detail="Session not found")
    return session


@router.post("/sessions", response_model=SessionDetail)
def save_session(req: SessionCreate):
    """Create or update a chat session."""
    return _history.save_session(req)


@router.delete("/sessions/{session_id}")
def delete_session(session_id: str):
    """Delete a chat session."""
    deleted = _history.delete_session(session_id)
    if not deleted:
        raise HTTPException(status_code=404, detail="Session not found")
    return {"deleted": True}
