"""FastAPI application factory and ASGI entry point."""

from __future__ import annotations

import logging

from fastapi import FastAPI
from fastapi.middleware.cors import CORSMiddleware

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

    @application.get("/")
    def root():
        return {"message": "Nexa Support service is running", "version": "2.0.0"}

    return application


app = create_app()
