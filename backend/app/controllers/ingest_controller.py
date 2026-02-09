"""Ingest controller — manages document ingestion into the vector store."""

from __future__ import annotations

import logging
from typing import List, Optional

from typing import TYPE_CHECKING

if TYPE_CHECKING:  # pragma: no cover
    from app.services.ingestion.ingest_service import IngestionService

logger = logging.getLogger(__name__)


class IngestController:
    def __init__(self, ingestion_service: IngestionService) -> None:
        self.service = ingestion_service

    def ingest(
        self,
        paths: List[str],
        tags: Optional[List[str]] = None,
        version: Optional[str] = None,
    ) -> int:
        """Validate inputs and run ingestion. Returns chunk count."""
        if not paths:
            raise ValueError("Paths list is empty")

        logger.info("Ingest request: paths=%s tags=%s version=%s", paths, tags, version)
        count = self.service.ingest_paths(paths, tags=tags, version=version)
        return count
