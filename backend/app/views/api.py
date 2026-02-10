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
    ApiKeysResponse,
    ApiKeysUpdateRequest,
    ChatRequest,
    ChatResponse,
    ConfigResponse,
    HealthResponse,
    IndexStatsResponse,
    IngestRequest,
    IngestResponse,
    LLMSettingsResponse,
    LLMSettingsUpdateRequest,
    OllamaModelEntry,
    OllamaModelsResponse,
    OllamaStatusResponse,
    PromptsResponse,
    PromptsUpdateRequest,
    SessionCreate,
    SessionDetail,
    SessionListResponse,
    SessionSummary,
    SwitchModelRequest,
    SwitchModelResponse,
    UploadResponse,
    UploadedFileInfo,
    UploadedFileEntry,
    UploadedFilesListResponse,
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


@router.delete("/sessions")
def clear_all_sessions():
    """Delete all chat sessions."""
    sessions = _history.list_sessions()
    count = 0
    for s in sessions:
        if _history.delete_session(s.id):
            count += 1
    return {"deleted": count}


# ── Prompts endpoints ────────────────────────────────────


@router.get("/prompts", response_model=PromptsResponse)
def get_prompts():
    """Read current system and RAG addon prompts."""
    settings = get_settings()
    system_prompt = ""
    rag_addon_prompt = ""
    try:
        with open(settings.system_prompt_path, "r", encoding="utf-8") as f:
            system_prompt = f.read()
    except FileNotFoundError:
        pass
    try:
        with open(settings.rag_prompt_path, "r", encoding="utf-8") as f:
            rag_addon_prompt = f.read()
    except FileNotFoundError:
        pass
    return PromptsResponse(system_prompt=system_prompt, rag_addon_prompt=rag_addon_prompt)


@router.put("/prompts", response_model=PromptsResponse)
def update_prompts(req: PromptsUpdateRequest):
    """Update system and/or RAG addon prompts."""
    settings = get_settings()
    if req.system_prompt is not None:
        with open(settings.system_prompt_path, "w", encoding="utf-8") as f:
            f.write(req.system_prompt)
        logger.info("Updated system prompt (%d chars)", len(req.system_prompt))
    if req.rag_addon_prompt is not None:
        with open(settings.rag_prompt_path, "w", encoding="utf-8") as f:
            f.write(req.rag_addon_prompt)
        logger.info("Updated RAG addon prompt (%d chars)", len(req.rag_addon_prompt))
    return get_prompts()


# ── Uploaded files management ────────────────────────────


@router.get("/uploads", response_model=UploadedFilesListResponse)
def list_uploaded_files():
    """List all uploaded files in the uploads directory."""
    UPLOAD_DIR.mkdir(parents=True, exist_ok=True)
    entries = []
    for p in sorted(UPLOAD_DIR.iterdir()):
        if p.is_file():
            stat = p.stat()
            from datetime import datetime, timezone
            entries.append(UploadedFileEntry(
                name=p.name,
                path=str(p),
                size=stat.st_size,
                modified_at=datetime.fromtimestamp(stat.st_mtime, tz=timezone.utc).isoformat(),
            ))
    return UploadedFilesListResponse(files=entries)


@router.delete("/uploads/{filename}")
def delete_uploaded_file(filename: str):
    """Delete an uploaded file."""
    safe_name = Path(filename).name  # prevent path traversal
    target = UPLOAD_DIR / safe_name
    if not target.exists():
        raise HTTPException(status_code=404, detail="File not found")
    target.unlink()
    logger.info("Deleted uploaded file: %s", target)
    return {"deleted": True, "file": safe_name}


# ── Index stats & rebuild ────────────────────────────────


@router.get("/index/stats", response_model=IndexStatsResponse)
def index_stats():
    """Return vector index statistics."""
    settings = get_settings()
    index_size = 0
    meta_size = 0
    total_vectors = 0
    total_metadata = 0

    idx_path = Path(settings.index_path)
    meta_path = Path(settings.metadata_path)

    if idx_path.exists():
        index_size = idx_path.stat().st_size
    if meta_path.exists():
        meta_size = meta_path.stat().st_size
        try:
            import json as _json
            with open(meta_path, "r", encoding="utf-8") as f:
                meta = _json.load(f)
                total_metadata = len(meta) if isinstance(meta, list) else 0
        except Exception:
            pass

    try:
        store = get_vector_store()
        if hasattr(store, 'index') and hasattr(store.index, 'ntotal'):
            total_vectors = store.index.ntotal
    except Exception:
        pass

    return IndexStatsResponse(
        total_vectors=total_vectors,
        total_metadata=total_metadata,
        index_path=settings.index_path,
        metadata_path=settings.metadata_path,
        index_size_bytes=index_size,
        metadata_size_bytes=meta_size,
    )


@router.post("/index/rebuild")
def rebuild_index():
    """Clear and rebuild the vector index (deletes existing index files)."""
    settings = get_settings()
    idx_path = Path(settings.index_path)
    meta_path = Path(settings.metadata_path)

    deleted_files = []
    if idx_path.exists():
        idx_path.unlink()
        deleted_files.append(str(idx_path))
    if meta_path.exists():
        meta_path.unlink()
        deleted_files.append(str(meta_path))

    # Clear cached vector store so it gets recreated
    from app.dependencies import _cache
    _cache.pop("store", None)
    _cache.pop("pipeline", None)
    _cache.pop("ingestion", None)

    logger.info("Index rebuilt (cleared files: %s)", deleted_files)
    return {"rebuilt": True, "deleted_files": deleted_files}


# ── LLM / RAG Settings ──────────────────────────────────

from app.dependencies import get_vector_store


# ── API Keys ─────────────────────────────────────────────


@router.get("/settings/api-keys", response_model=ApiKeysResponse)
def get_api_keys():
    """Read current cloud API key configuration (key is never exposed)."""
    settings = get_settings()
    return ApiKeysResponse(
        llm_provider=settings.llm_provider,
        cloud_api_key_set=bool(settings.cloud_api_key),
        cloud_base_url=settings.cloud_base_url,
        cloud_model=settings.cloud_model,
    )


@router.put("/settings/api-keys", response_model=ApiKeysResponse)
def update_api_keys(req: ApiKeysUpdateRequest):
    """Update cloud API keys and provider at runtime."""
    settings = get_settings()
    changed = False
    if req.llm_provider is not None and req.llm_provider != settings.llm_provider:
        settings.llm_provider = req.llm_provider
        changed = True
    if req.cloud_api_key is not None:
        settings.cloud_api_key = req.cloud_api_key
        changed = True
    if req.cloud_base_url is not None:
        settings.cloud_base_url = req.cloud_base_url
        changed = True
    if req.cloud_model is not None:
        settings.cloud_model = req.cloud_model
        changed = True
    if changed:
        _cache.pop("llm", None)
        _cache.pop("pipeline", None)
        logger.info("Updated API keys: provider=%s", settings.llm_provider)
    return get_api_keys()


@router.post("/index/clear")
def clear_index():
    """Completely clear the vector index and metadata files."""
    settings = get_settings()
    idx_path = Path(settings.index_path)
    meta_path = Path(settings.metadata_path)
    deleted_files = []
    if idx_path.exists():
        idx_path.unlink()
        deleted_files.append(str(idx_path))
    if meta_path.exists():
        meta_path.unlink()
        deleted_files.append(str(meta_path))
    _cache.pop("store", None)
    _cache.pop("pipeline", None)
    _cache.pop("ingestion", None)
    logger.info("Index cleared (deleted: %s)", deleted_files)
    return {"cleared": True, "deleted_files": deleted_files}


@router.get("/settings/llm", response_model=LLMSettingsResponse)
def get_llm_settings():
    """Read current LLM and RAG tuning parameters."""
    settings = get_settings()
    return LLMSettingsResponse(
        temperature=settings.temperature,
        top_p=settings.top_p,
        max_tokens=settings.max_tokens,
        chunk_size=settings.chunk_size,
        chunk_overlap=settings.chunk_overlap,
        top_k=settings.top_k,
        similarity_threshold=settings.similarity_threshold,
    )


@router.put("/settings/llm", response_model=LLMSettingsResponse)
def update_llm_settings(req: LLMSettingsUpdateRequest):
    """Update LLM/RAG tuning parameters at runtime."""
    settings = get_settings()
    llm = get_llm_client()

    if req.temperature is not None:
        settings.temperature = req.temperature
        if hasattr(llm, 'temperature'):
            llm.temperature = req.temperature
    if req.top_p is not None:
        settings.top_p = req.top_p
        if hasattr(llm, 'top_p'):
            llm.top_p = req.top_p
    if req.max_tokens is not None:
        settings.max_tokens = req.max_tokens
        if hasattr(llm, 'max_tokens'):
            llm.max_tokens = req.max_tokens
    if req.chunk_size is not None:
        settings.chunk_size = req.chunk_size
    if req.chunk_overlap is not None:
        settings.chunk_overlap = req.chunk_overlap
    if req.top_k is not None:
        settings.top_k = req.top_k
        pipeline = get_rag_pipeline()
        if hasattr(pipeline, 'top_k'):
            pipeline.top_k = req.top_k
    if req.similarity_threshold is not None:
        settings.similarity_threshold = req.similarity_threshold
        pipeline = get_rag_pipeline()
        if hasattr(pipeline, 'similarity_threshold'):
            pipeline.similarity_threshold = req.similarity_threshold

    logger.info("Updated LLM/RAG settings")
    return get_llm_settings()
