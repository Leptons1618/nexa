"""API routes — the 'V' in MVC.  Thin layer that delegates to controllers."""

from __future__ import annotations

import logging

from fastapi import APIRouter, HTTPException

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
)

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


@router.post("/ingest", response_model=IngestResponse)
def ingest(req: IngestRequest):
    controller = IngestController(get_ingestion_service())
    try:
        count = controller.ingest(req.paths, tags=req.tags, version=req.version)
    except ValueError as exc:
        raise HTTPException(status_code=400, detail=str(exc))
    return IngestResponse(chunks_indexed=count)


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
