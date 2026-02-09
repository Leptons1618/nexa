"""FastAPI application factory and ASGI entry point."""

from __future__ import annotations

import logging
from pathlib import Path

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware
from fastapi.staticfiles import StaticFiles

from app.config.settings import get_settings


def create_app() -> FastAPI:
    """Build and configure the FastAPI application."""
    settings = get_settings()
    logging.basicConfig(
        level=getattr(logging, settings.log_level.upper(), logging.INFO),
        format="%(asctime)s | %(levelname)-8s | %(name)s | %(message)s",
    )

    application = FastAPI(
        title=settings.app_name,
        version="2.0.0",
        description="Nexa Support — RAG-powered support chatbot with Ollama and cloud LLM support",
    )

    application.add_middleware(
        CORSMiddleware,
        allow_origins=["*"],
        allow_credentials=True,
        allow_methods=["*"],
        allow_headers=["*"],
    )

    from app.views.api import router  # noqa: E402

    application.include_router(router)

    # Ensure upload directory exists
    Path("data/uploads").mkdir(parents=True, exist_ok=True)

    # Serve static frontend (production)
    static_dir = Path(__file__).resolve().parent.parent / "static"
    if static_dir.is_dir():
        from fastapi.responses import FileResponse

        @application.get("/")
        def serve_index():
            index = static_dir / "index.html"
            if index.exists():
                return FileResponse(str(index))
            return {"message": "Nexa Support service is running", "version": "2.0.0"}

        application.mount(
            "/assets",
            StaticFiles(directory=str(static_dir / "assets")),
            name="static-assets",
        )

        # Serve WASM files
        wasm_dir = static_dir / "wasm"
        if wasm_dir.is_dir():
            application.mount(
                "/wasm",
                StaticFiles(directory=str(wasm_dir)),
                name="static-wasm",
            )
    else:

        @application.get("/")
        def root():
            return {"message": "Nexa Support service is running", "version": "2.0.0"}

    return application


app = create_app()
