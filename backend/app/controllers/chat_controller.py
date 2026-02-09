"""Chat controller — orchestrates RAG pipeline for user queries."""

from __future__ import annotations

import logging
from typing import List, Tuple

from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from app.services.rag.pipeline import RAGPipeline

logger = logging.getLogger(__name__)


class ChatController:
    def __init__(self, pipeline: RAGPipeline) -> None:
        self.pipeline = pipeline

    def handle_chat(self, message: str) -> Tuple[str, List[str]]:
        """Validate the message and return (answer, sources)."""
        cleaned = message.strip()
        if not cleaned:
            raise ValueError("Message cannot be empty")

        logger.info("Chat request: %s", cleaned[:80])
        answer, sources = self.pipeline.generate(cleaned)
        return answer, sources
